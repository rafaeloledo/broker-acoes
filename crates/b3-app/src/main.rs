use b3_analytics::{multiples_from_accounts, CoreMultiples, ScreenerCriteria};
use b3_core::fii::FiiDetails;
use b3_core::indicators::StockIndicators;
use b3_core::quote::{Candle, OrderBook, OrderBookLevel, Tick};
use b3_core::tax::{MonthlyTaxReport, TradeOperation};
use b3_core::ticker::{AssetClass, CompanyProfile};
use b3_gpu::candlestick::CandlestickPipeline;
use b3_gpu::telemetry::GpuTelemetry;
use b3_gpu::treemap::{GpuTreemapNode, TreemapPipeline};
use b3_ingest::live::LiveFeedClient;
use b3_ingest::sinacor::SinacorParser;
use b3_portfolio::position::PortfolioTracker;
use b3_portfolio::tax_calc::TaxCalculator;
use b3_storage::Database;

use chrono::Utc;
use eframe::egui::{self, Color32, RichText};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

mod market_feed;
mod presentation;
mod views;

use market_feed::{MarketEvent, QuoteTick};
use presentation::{format_metric, MetricId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveView {
    Decision,
    News,
    StatusInvest,
    GpuTerminal,
    MarketHeatmap,
    Screener,
    PortfolioTax,
}

pub const MAX_VISIBLE_ASSETS: usize = 4;

#[allow(dead_code)]
struct AppState {
    active_view: ActiveView,
    selected_ticker: String,
    visible_tickers: Vec<String>,
    focused_ticker_idx: usize,
    search_query: String,

    // B3 Universe Search Modal
    is_search_open: bool,
    search_target_slot: Option<usize>, // None = Add to screen, Some(idx) = Change slot idx
    b3_search_query: String,
    b3_filter_class: Option<AssetClass>,
    just_opened_search: bool,

    // Database & domain
    db: Arc<Database>,
    db_path: String,
    stock_indicators: HashMap<String, StockIndicators>,
    company_profiles: HashMap<String, CompanyProfile>,
    fii_details: HashMap<String, FiiDetails>,
    candles: HashMap<String, Vec<Candle>>,
    current_prices: HashMap<String, f64>,
    sim_prices: HashMap<String, f64>,
    market_as_of: HashMap<String, i64>,
    cvm_year: HashMap<String, i32>,
    cvm_accounts: HashMap<String, Vec<(String, f64)>>,
    core_multiples: HashMap<String, CoreMultiples>,
    market_rx: crossbeam_channel::Receiver<MarketEvent>,

    // Real-time market terminal
    order_book: OrderBook,
    recent_ticks: Vec<Tick>,
    is_live_sim_running: bool,
    tick_counter: u64,

    // GPU pipelines & telemetry
    candlestick_pipe: Arc<Mutex<CandlestickPipeline>>,
    treemap_pipe: Arc<Mutex<TreemapPipeline>>,
    telemetry: GpuTelemetry,
    chart_start_idx: f32,
    chart_candle_width: f32,

    // Heatmap nodes
    heatmap_nodes: Vec<GpuTreemapNode>,
    heatmap_tickers: Vec<String>,

    // Screener
    screener_criteria: ScreenerCriteria,
    screener_preset_name: String,

    // Portfolio & Tax
    portfolio_tracker: PortfolioTracker,
    tax_calc: TaxCalculator,
    trades: Vec<TradeOperation>,
    monthly_reports: Vec<MonthlyTaxReport>,
    bazin_target_yield: f64,
    news_text: String,
    news_ticker: String,
    news_period_days: Option<i64>,
    news_rows: Vec<b3_core::news::NewsItem>,
    news_attempted_at: Option<i64>,
    news_generation: u64,
    news_loaded_generation: u64,
    news_loaded_key: String,
    decision_news: Vec<b3_core::news::NewsItem>,
    decision_news_ticker: String,
    decision_news_generation: u64,
}

impl AppState {
    fn new(cc: &eframe::CreationContext) -> Self {
        let db_path = std::env::var("BROKER_DB_PATH").unwrap_or_else(|_| "broker.db".to_string());
        let db = Arc::new(Database::open(&db_path).unwrap_or_else(|e| {
            log::warn!("Failed to open SQLite database at '{}': {}. Falling back to in-memory database.", db_path, e);
            Database::in_memory().expect("Failed to initialize database")
        }));

        // Initialize GPU pipelines using the wgpu device from eframe
        let wgpu_render_state = cc.wgpu_render_state.as_ref().expect("wgpu is required");
        let device = &wgpu_render_state.device;
        let target_format = wgpu_render_state.target_format;

        let candlestick_pipe = Arc::new(Mutex::new(CandlestickPipeline::new(device, target_format)));
        let treemap_pipe = Arc::new(Mutex::new(TreemapPipeline::new(device, target_format)));

        // Load user-defined visible tickers from SQLite database persistence
        let mut user_tickers = db.get_user_tickers().unwrap_or_default();
        if user_tickers.is_empty() {
            let defaults = vec![
                "PETR4".to_string(),
                "VALE3".to_string(),
                "WEGE3".to_string(),
                "ITUB4".to_string(),
            ];
            if let Err(e) = db.set_user_tickers(&defaults) {
                log::warn!("Failed to seed initial user tickers in database: {}", e);
            }
            user_tickers = defaults;
        }

        let visible_tickers: Vec<String> = user_tickers.into_iter().take(MAX_VISIBLE_ASSETS).collect();
        let selected_ticker = visible_tickers.first().cloned().unwrap_or_else(|| "PETR4".to_string());

        let mut tickers = vec![
            "PETR4".to_string(), "VALE3".to_string(), "WEGE3".to_string(), "ITUB4".to_string(),
            "BBAS3".to_string(), "TAEE11".to_string(), "EGIE3".to_string(), "HGLG11".to_string(),
            "KNCR11".to_string(), "MXRF11".to_string(), "XPML11".to_string(), "BTLG11".to_string(),
        ];
        for t in &visible_tickers {
            if !tickers.contains(t) {
                tickers.push(t.clone());
            }
        }

        let mut stock_indicators = HashMap::new();
        let mut company_profiles = HashMap::new();
        let mut fii_details = HashMap::new();
        let mut candles = HashMap::new();
        let mut current_prices = HashMap::new();
        let mut sim_prices = HashMap::new();
        let mut market_as_of = HashMap::new();
        let mut cvm_year = HashMap::new();
        let mut cvm_accounts = HashMap::new();
        let mut core_multiples = HashMap::new();

        for t in &tickers {
            let ind = StockIndicators::default_for(t);
            let base_price = ind.price;
            current_prices.insert(t.clone(), base_price);
            stock_indicators.insert(t.clone(), ind);
            company_profiles.insert(t.clone(), CompanyProfile::default_for(t));

            if AssetClass::from_symbol(t) == AssetClass::Fii {
                fii_details.insert(t.clone(), FiiDetails::default_for(t));
            }

            let existing_quotes = db.get_quotes(t, 180).unwrap_or_default();
            let sym_candles = if !existing_quotes.is_empty() {
                existing_quotes
            } else {
                synthetic_candles(base_price)
            };
            if let Ok(Some((candle, fetched_at))) = db.latest_market_quote(t) {
                current_prices.insert(t.clone(), candle.close);
                market_as_of.insert(t.clone(), fetched_at);
                if let Some(ind) = stock_indicators.get_mut(t) {
                    ind.price = candle.close;
                }
            }
            if AssetClass::from_symbol(t) != AssetClass::Fii {
                if let Ok(Some((year, accounts))) = db.load_cvm_accounts(t) {
                    let price = *current_prices.get(t).unwrap_or(&base_price);
                    let multiples = multiples_from_accounts(price, &accounts);
                    if let Some(ind) = stock_indicators.get_mut(t) {
                        copy_core_onto(ind, &multiples);
                    }
                    cvm_year.insert(t.clone(), year);
                    core_multiples.insert(t.clone(), multiples);
                    cvm_accounts.insert(t.clone(), accounts);
                }
            }
            sim_prices.insert(t.clone(), *current_prices.get(t).unwrap_or(&base_price));
            candles.insert(t.clone(), sym_candles);
        }

        // Initialize realistic portfolio with SINACOR trades (load from DB if present)
        let sample_note = r#"
            1-BOVESPA C VISTA PETR4 PETROBRAS PN 200 34,20 6.840,00 C
            1-BOVESPA C VISTA VALE3 VALE ON 100 56,50 5.650,00 C
            1-BOVESPA C VISTA WEGE3 WEG ON 150 48,00 7.200,00 C
            1-BOVESPA C VISTA HGLG11 CSHG LOGISTICA 50 151,00 7.550,00 C
            1-BOVESPA C VISTA BBAS3 BANCO DO BRASIL 200 26,80 5.360,00 C
            1-BOVESPA V VISTA PETR4 PETROBRAS PN 50 37,10 1.855,00 D
        "#;
        let mut trades = db.get_all_trades().unwrap_or_default();
        if trades.is_empty() {
            trades = SinacorParser::parse_trade_lines(sample_note, "XP Investimentos");
            for tr in &trades {
                let _ = db.insert_trade(tr);
            }
        }
        let mut portfolio_tracker = PortfolioTracker::new();
        let mut tax_calc = TaxCalculator::new();
        let monthly_reports = tax_calc.compute_monthly_taxes(&trades, &mut portfolio_tracker);

        // Precompute Treemap nodes for major B3 sectors
        let (heatmap_nodes, heatmap_tickers) = Self::layout_heatmap(&stock_indicators);

        // Initialize Level 2 orderbook for selected ticker
        let base_p = current_prices.get(&selected_ticker).copied().unwrap_or(36.40);
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        for i in 1..=8 {
            bids.push(OrderBookLevel {
                price: base_p - (i as f64 * 0.01),
                quantity: (10 - i) as f64 * 1200.0,
                orders_count: (12 - i) as u32,
            });
            asks.push(OrderBookLevel {
                price: base_p + (i as f64 * 0.01),
                quantity: (i + 2) as f64 * 1100.0,
                orders_count: (i + 3) as u32,
            });
        }
        let order_book = OrderBook {
            symbol: selected_ticker.clone(),
            timestamp_ms: Utc::now().timestamp_millis(),
            bids,
            asks,
        };
        let market_rx = market_feed::spawn(db.clone());
        let news_ticker = selected_ticker.clone();
        let news_attempted_at = db.latest_news_attempt().ok().flatten();

        Self {
            active_view: ActiveView::Decision,
            selected_ticker,
            visible_tickers,
            focused_ticker_idx: 0,
            search_query: String::new(),
            is_search_open: false,
            search_target_slot: None,
            b3_search_query: String::new(),
            b3_filter_class: None,
            just_opened_search: false,
            db,
            db_path,
            stock_indicators,
            company_profiles,
            fii_details,
            candles,
            current_prices,
            sim_prices,
            market_as_of,
            cvm_year,
            cvm_accounts,
            core_multiples,
            market_rx,
            order_book,
            recent_ticks: Vec::new(),
            is_live_sim_running: true,
            tick_counter: 0,
            candlestick_pipe,
            treemap_pipe,
            telemetry: GpuTelemetry::new(),
            chart_start_idx: 80.0,
            chart_candle_width: 7.0,
            heatmap_nodes,
            heatmap_tickers,
            screener_criteria: ScreenerCriteria::default(),
            screener_preset_name: "Todos os Ativos".to_string(),
            portfolio_tracker,
            tax_calc,
            trades,
            monthly_reports,
            bazin_target_yield: 6.0,
            news_text: String::new(),
            news_ticker,
            news_period_days: Some(30),
            news_rows: Vec::new(),
            news_attempted_at,
            news_generation: 0,
            news_loaded_generation: u64::MAX,
            news_loaded_key: String::new(),
            decision_news: Vec::new(),
            decision_news_ticker: String::new(),
            decision_news_generation: u64::MAX,
        }
    }

    fn persist_visible_tickers(&self) {
        if let Err(e) = self.db.set_user_tickers(&self.visible_tickers) {
            log::error!("Failed to persist user tickers to SQLite database: {}", e);
        }
    }

    fn ensure_asset_loaded(&mut self, ticker: &str) {
        let sym = ticker.to_uppercase();
        if self.stock_indicators.contains_key(&sym) {
            return;
        }

        let ind = StockIndicators::default_for(&sym);
        let base_price = ind.price;
        self.current_prices.insert(sym.clone(), base_price);
        self.sim_prices.insert(sym.clone(), base_price);
        self.stock_indicators.insert(sym.clone(), ind);
        self.company_profiles.insert(sym.clone(), CompanyProfile::default_for(&sym));

        if AssetClass::from_symbol(&sym) == AssetClass::Fii {
            self.fii_details.insert(sym.clone(), FiiDetails::default_for(&sym));
        }

        let existing_quotes = self.db.get_quotes(&sym, 180).unwrap_or_default();
        let sym_candles = if !existing_quotes.is_empty() {
            existing_quotes
        } else {
            synthetic_candles(base_price)
        };
        if let Ok(Some((candle, fetched_at))) = self.db.latest_market_quote(&sym) {
            self.current_prices.insert(sym.clone(), candle.close);
            self.market_as_of.insert(sym.clone(), fetched_at);
            if let Some(ind) = self.stock_indicators.get_mut(&sym) {
                ind.price = candle.close;
            }
        }
        self.candles.insert(sym.clone(), sym_candles);
        if AssetClass::from_symbol(&sym) != AssetClass::Fii {
            if let Ok(Some((year, accounts))) = self.db.load_cvm_accounts(&sym) {
                self.store_cvm(&sym, year, accounts);
            }
        }
    }

    fn apply_selected_ticker(&mut self, ticker: &str) {
        let sym = ticker.to_uppercase();
        self.ensure_asset_loaded(&sym);
        self.selected_ticker = sym.clone();

        if let Some(slot) = self.search_target_slot {
            if slot < self.visible_tickers.len() {
                self.visible_tickers[slot] = sym.clone();
                self.focused_ticker_idx = slot;
            }
        } else {
            if !self.visible_tickers.contains(&sym) {
                if self.visible_tickers.len() < MAX_VISIBLE_ASSETS {
                    self.visible_tickers.push(sym.clone());
                    self.focused_ticker_idx = self.visible_tickers.len() - 1;
                } else if !self.visible_tickers.is_empty() {
                    self.visible_tickers[self.focused_ticker_idx] = sym.clone();
                }
            } else if let Some(pos) = self.visible_tickers.iter().position(|t| t == &sym) {
                self.focused_ticker_idx = pos;
            }
        }
        self.persist_visible_tickers();
    }

    fn layout_heatmap(_indicators: &HashMap<String, StockIndicators>) -> (Vec<GpuTreemapNode>, Vec<String>) {
        let tickers = vec![
            ("PETR4", 2.45), ("VALE3", -1.80), ("ITUB4", 1.15), ("BBDC4", 0.65),
            ("BBAS3", 1.95), ("WEGE3", 3.20), ("TAEE11", 0.40), ("EGIE3", -0.50),
            ("HGLG11", 0.25), ("KNCR11", 0.10), ("MXRF11", -0.30), ("XPML11", 0.85),
        ];

        let mut nodes = Vec::new();
        let mut names = Vec::new();

        // 4 columns x 3 rows grid normalized [0, 1]
        let cols = 4;
        let rows = 3;
        let w = 1.0 / cols as f32;
        let h = 1.0 / rows as f32;

        for (idx, (sym, chg)) in tickers.iter().enumerate() {
            let col = (idx % cols) as f32;
            let row = (idx / cols) as f32;
            let x = col * w;
            let y = row * h;

            nodes.push(GpuTreemapNode {
                x,
                y,
                w,
                h,
                change_pct: *chg,
                flash_intensity: 0.0,
                _pad1: 0.0,
                _pad2: 0.0,
            });
            names.push(sym.to_string());
        }

        (nodes, names)
    }

    fn on_tick(&mut self) {
        self.tick_counter += 1;
        let tickers = self.visible_tickers.clone();
        for sym in tickers {
            let last_p = *self.sim_prices.get(&sym).unwrap_or(&35.0);
            let tick = LiveFeedClient::simulate_tick(&sym, last_p);
            self.sim_prices.insert(sym.clone(), tick.price);

            if let Some(list) = self.candles.get_mut(&sym) {
                if let Some(last_candle) = list.last_mut() {
                    last_candle.close = tick.price;
                    last_candle.high = last_candle.high.max(tick.price);
                    last_candle.low = last_candle.low.min(tick.price);
                    last_candle.volume += tick.quantity * tick.price;
                }
            }

            if sym == self.selected_ticker {
                if tick.is_buyer_aggressor {
                    if let Some(ask) = self.order_book.asks.first_mut() {
                        ask.price = tick.price;
                    }
                } else if let Some(bid) = self.order_book.bids.first_mut() {
                    bid.price = tick.price;
                }

                self.recent_ticks.insert(0, tick);
                if self.recent_ticks.len() > 25 {
                    self.recent_ticks.pop();
                }
            }
        }

        if !self.heatmap_nodes.is_empty() {
            let node_idx = (self.tick_counter as usize) % self.heatmap_nodes.len();
            self.heatmap_nodes[node_idx].flash_intensity = 1.0;
        }
    }

    fn drain_market_events(&mut self) {
        while let Ok(event) = self.market_rx.try_recv() {
            match event {
                MarketEvent::Quotes(quotes) => {
                    for quote in quotes {
                        self.apply_quote(quote);
                    }
                }
                MarketEvent::Cvm(rows) => {
                    for row in rows {
                        self.store_cvm(&row.ticker, row.year, row.accounts);
                    }
                }
                MarketEvent::News => {
                    self.news_generation = self.news_generation.wrapping_add(1);
                    self.news_attempted_at = self.db.latest_news_attempt().ok().flatten();
                }
            }
        }
    }

    fn apply_quote(&mut self, quote: QuoteTick) {
        self.current_prices.insert(quote.symbol.clone(), quote.close);
        self.market_as_of.insert(quote.symbol.clone(), quote.fetched_at_ms);
        self.sim_prices.entry(quote.symbol.clone()).or_insert(quote.close);
        if let Some(ind) = self.stock_indicators.get_mut(&quote.symbol) {
            ind.price = quote.close;
        }
        if let Some(accounts) = self.cvm_accounts.get(&quote.symbol).cloned() {
            self.recompute_core(&quote.symbol, quote.close, &accounts);
        }
    }

    fn store_cvm(&mut self, ticker: &str, year: i32, accounts: Vec<(String, f64)>) {
        if AssetClass::from_symbol(ticker) == AssetClass::Fii {
            return;
        }
        let price = self
            .current_prices
            .get(ticker)
            .copied()
            .unwrap_or_else(|| {
                self.stock_indicators
                    .get(ticker)
                    .map(|ind| ind.price)
                    .unwrap_or(0.0)
            });
        self.cvm_year.insert(ticker.to_string(), year);
        self.cvm_accounts.insert(ticker.to_string(), accounts.clone());
        self.recompute_core(ticker, price, &accounts);
    }

    fn recompute_core(&mut self, ticker: &str, price: f64, accounts: &[(String, f64)]) {
        let multiples = multiples_from_accounts(price, accounts);
        if let Some(ind) = self.stock_indicators.get_mut(ticker) {
            copy_core_onto(ind, &multiples);
            ind.price = price;
        }
        self.core_multiples.insert(ticker.to_string(), multiples);
    }

    fn shown_price(&self, sym: &str, fallback: f64) -> f64 {
        if self.market_as_of.contains_key(sym) {
            self.current_prices.get(sym).copied().unwrap_or(fallback)
        } else {
            fallback
        }
    }

    fn metric(&self, sym: &str, metric: MetricId, local_display: &str) -> String {
        if self.cvm_year.contains_key(sym) {
            format_metric(self.core_multiples.get(sym), metric)
        } else {
            local_display.to_string()
        }
    }
}

impl eframe::App for AppState {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.telemetry.on_frame();
        self.drain_market_events();
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));

        // High-frequency refresh simulation
        if self.is_live_sim_running {
            self.on_tick();
            // Decay heatmap flashes smoothly
            for n in &mut self.heatmap_nodes {
                n.flash_intensity = (n.flash_intensity * 0.88).max(0.0);
            }
            ui.ctx().request_repaint(); // Request next frame for 120Hz smooth animations
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new("BROKER-AÇÕES").strong().color(Color32::from_rgb(0, 210, 160)).size(17.0));
            ui.separator();

            let on_decision = self.active_view == ActiveView::Decision;
            if ui.selectable_label(on_decision, RichText::new("Decisão").strong()).clicked() {
                self.active_view = ActiveView::Decision;
            }

            let tools = [
                (ActiveView::News, "Notícias"),
                (ActiveView::GpuTerminal, "Terminal"),
                (ActiveView::MarketHeatmap, "Mapa"),
                (ActiveView::Screener, "Busca de ativos"),
                (ActiveView::PortfolioTax, "Carteira"),
                (ActiveView::StatusInvest, "Ficha"),
            ];
            ui.menu_button(RichText::new("Ferramentas").strong(), |ui| {
                for (view, label) in tools {
                    if ui.button(label).clicked() {
                        self.active_view = view;
                        ui.close();
                    }
                }
            });

            if !on_decision {
                if ui.button("Voltar à Decisão").clicked() {
                    self.active_view = ActiveView::Decision;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.visible_tickers.len() < MAX_VISIBLE_ASSETS {
                    if ui.button(RichText::new("+ Adicionar").strong().color(Color32::from_rgb(0, 210, 160))).clicked() {
                        self.search_target_slot = None;
                        self.b3_search_query.clear();
                        self.is_search_open = true;
                        self.just_opened_search = true;
                    }
                }

                let mut slot_to_remove: Option<usize> = None;
                let mut slot_to_change: Option<usize> = None;

                for i in (0..self.visible_tickers.len()).rev() {
                    let sym = self.visible_tickers[i].clone();
                    let is_focused = self.focused_ticker_idx == i;
                    let chip_color = if is_focused {
                        Color32::from_rgb(0, 210, 160)
                    } else {
                        Color32::WHITE
                    };

                    if ui.button(RichText::new(&sym).strong().color(chip_color)).clicked() {
                        self.focused_ticker_idx = i;
                        self.news_ticker = sym.clone();
                        self.selected_ticker = sym;
                    }
                    if ui.small_button("trocar").clicked() {
                        slot_to_change = Some(i);
                    }

                    if self.visible_tickers.len() > 1 && ui.small_button("x").clicked() {
                        slot_to_remove = Some(i);
                    }
                }

                if let Some(r) = slot_to_remove {
                    self.visible_tickers.remove(r);
                    if self.focused_ticker_idx >= self.visible_tickers.len() {
                        self.focused_ticker_idx = self.visible_tickers.len().saturating_sub(1);
                    }
                    if !self.visible_tickers.is_empty() {
                        self.selected_ticker = self.visible_tickers[self.focused_ticker_idx].clone();
                    }
                    self.persist_visible_tickers();
                }

                if let Some(s) = slot_to_change {
                    self.search_target_slot = Some(s);
                    self.b3_search_query.clear();
                    self.is_search_open = true;
                    self.just_opened_search = true;
                }

                ui.label(RichText::new("Ativos").color(Color32::GRAY));
            });
        });

        ui.separator();

        // Render search modal if open
        self.render_search_modal(ui.ctx());

        // Main View Content
        match self.active_view {
            ActiveView::Decision => self.render_decision_view(ui),
            ActiveView::News => self.render_news_view(ui),
            ActiveView::StatusInvest => self.render_status_invest_view(ui),
            ActiveView::GpuTerminal => self.render_gpu_terminal_view(ui),
            ActiveView::MarketHeatmap => self.render_heatmap_view(ui),
            ActiveView::Screener => self.render_screener_view(ui),
            ActiveView::PortfolioTax => self.render_portfolio_tax_view(ui),
        }
    }
}

fn copy_core_onto(ind: &mut StockIndicators, multiples: &CoreMultiples) {
    if let Some(value) = multiples.lpa { ind.lpa = value; }
    if let Some(value) = multiples.vpa { ind.vpa = value; }
    if let Some(value) = multiples.dpa { ind.dpa = value; }
    if let Some(value) = multiples.pl { ind.pl = value; }
    if let Some(value) = multiples.pvp { ind.pvp = value; }
    if let Some(value) = multiples.dividend_yield { ind.dividend_yield = value; }
    if let Some(value) = multiples.graham_fair_value { ind.graham_fair_value = value; }
    if let Some(value) = multiples.bazin_ceiling_price { ind.bazin_ceiling_price = value; }
}

fn synthetic_candles(base_price: f64) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(180);
    let mut price = base_price * 0.85;
    let now = Utc::now().timestamp_millis();
    for i in (0..180).rev() {
        let day_ms = now - (i * 86_400_000);
        let delta = ((i as f64 * 7.3).sin() * 0.035) * price;
        let open = price;
        let close = (price + delta).max(1.0);
        let high = open.max(close) + (price * 0.015);
        let low = (open.min(close) - (price * 0.012)).max(0.5);
        let volume = 15_000_000.0 + ((i as f64 * 3.1).cos().abs() * 30_000_000.0);
        price = close;
        candles.push(Candle {
            timestamp_ms: day_ms,
            open,
            high,
            low,
            close,
            volume,
            trades: 1200,
            adjusted_close: close,
        });
    }
    candles
}

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Broker Ações")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "broker-acoes",
        native_options,
        Box::new(|cc| Ok(Box::new(AppState::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_persistence_default_and_custom() {
        let db = Database::in_memory().unwrap();

        // Initial state is empty
        let initial = db.get_user_tickers().unwrap();
        assert!(initial.is_empty());

        // Seeding defaults when empty
        let defaults = vec![
            "PETR4".to_string(),
            "VALE3".to_string(),
            "WEGE3".to_string(),
            "ITUB4".to_string(),
        ];
        db.set_user_tickers(&defaults).unwrap();
        assert_eq!(db.get_user_tickers().unwrap(), defaults);

        // User customizes their own stocks (user-defined persistence)
        let custom_tickers = vec![
            "BBAS3".to_string(),
            "TAEE11".to_string(),
            "HGLG11".to_string(),
            "XPML11".to_string(),
        ];
        db.set_user_tickers(&custom_tickers).unwrap();
        let loaded = db.get_user_tickers().unwrap();
        assert_eq!(loaded, custom_tickers);

        // Respects MAX_VISIBLE_ASSETS limit when taking
        let visible: Vec<String> = loaded.into_iter().take(MAX_VISIBLE_ASSETS).collect();
        assert_eq!(visible.len(), 4);
        assert_eq!(visible[0], "BBAS3");
        assert_eq!(visible[3], "XPML11");
    }
}
