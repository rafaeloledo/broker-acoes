use b3_analytics::{ScreenerCriteria, ScreenerEngine};
use b3_core::fii::FiiDetails;
use b3_core::indicators::StockIndicators;
use b3_core::quote::{Candle, GpuCandle, OrderBook, OrderBookLevel, Tick};
use b3_core::tax::{MonthlyTaxReport, TradeOperation};
use b3_core::ticker::{AssetClass, CompanyProfile};
use b3_gpu::candlestick::{CandlestickPipeline, ChartUniforms};
use b3_gpu::egui_callback::{GpuChartCallback, GpuTreemapCallback};
use b3_gpu::telemetry::GpuTelemetry;
use b3_gpu::treemap::{GpuTreemapNode, TreemapPipeline, TreemapUniforms};
use b3_ingest::live::LiveFeedClient;
use b3_ingest::sinacor::SinacorParser;
use b3_portfolio::position::PortfolioTracker;
use b3_portfolio::tax_calc::TaxCalculator;
use b3_storage::Database;

use chrono::Utc;
use eframe::egui::{self, Color32, RichText, Sense, Vec2};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveView {
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
    stock_indicators: HashMap<String, StockIndicators>,
    company_profiles: HashMap<String, CompanyProfile>,
    fii_details: HashMap<String, FiiDetails>,
    candles: HashMap<String, Vec<Candle>>,
    current_prices: HashMap<String, f64>,

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
}

impl AppState {
    fn new(cc: &eframe::CreationContext) -> Self {
        let db = Arc::new(Database::in_memory().expect("Failed to initialize database"));

        // Initialize GPU pipelines using the wgpu device from eframe
        let wgpu_render_state = cc.wgpu_render_state.as_ref().expect("wgpu is required");
        let device = &wgpu_render_state.device;
        let target_format = wgpu_render_state.target_format;

        let candlestick_pipe = Arc::new(Mutex::new(CandlestickPipeline::new(device, target_format)));
        let treemap_pipe = Arc::new(Mutex::new(TreemapPipeline::new(device, target_format)));

        let tickers = vec![
            "PETR4", "VALE3", "WEGE3", "ITUB4", "BBAS3",
            "TAEE11", "EGIE3", "HGLG11", "KNCR11", "MXRF11", "XPML11", "BTLG11"
        ];

        let mut stock_indicators = HashMap::new();
        let mut company_profiles = HashMap::new();
        let mut fii_details = HashMap::new();
        let mut candles = HashMap::new();
        let mut current_prices = HashMap::new();

        for t in &tickers {
            let ind = StockIndicators::default_for(t);
            current_prices.insert(t.to_string(), ind.price);
            stock_indicators.insert(t.to_string(), ind);
            company_profiles.insert(t.to_string(), CompanyProfile::default_for(t));

            if AssetClass::from_symbol(t) == AssetClass::Fii {
                fii_details.insert(t.to_string(), FiiDetails::default_for(t));
            }

            // Generate realistic historical daily candles for GPU rendering
            let base_price = stock_indicators.get(*t).unwrap().price;
            let mut sym_candles = Vec::new();
            let mut p = base_price * 0.85;
            let now = Utc::now().timestamp_millis();
            for i in (0..180).rev() {
                let day_ms = now - (i * 86_400_000);
                let delta = ((i as f64 * 7.3).sin() * 0.035) * p;
                let open = p;
                let close = (p + delta).max(1.0);
                let high = open.max(close) + (p * 0.015);
                let low = (open.min(close) - (p * 0.012)).max(0.5);
                let vol = 15_000_000.0 + ((i as f64 * 3.1).cos().abs() * 30_000_000.0);
                p = close;
                sym_candles.push(Candle {
                    timestamp_ms: day_ms,
                    open,
                    high,
                    low,
                    close,
                    volume: vol,
                    trades: 1200,
                    adjusted_close: close,
                });
            }
            let _ = db.insert_quotes(t, &sym_candles);
            candles.insert(t.to_string(), sym_candles);
        }

        // Initialize realistic portfolio with SINACOR trades
        let sample_note = r#"
            1-BOVESPA C VISTA PETR4 PETROBRAS PN 200 34,20 6.840,00 C
            1-BOVESPA C VISTA VALE3 VALE ON 100 56,50 5.650,00 C
            1-BOVESPA C VISTA WEGE3 WEG ON 150 48,00 7.200,00 C
            1-BOVESPA C VISTA HGLG11 CSHG LOGISTICA 50 151,00 7.550,00 C
            1-BOVESPA C VISTA BBAS3 BANCO DO BRASIL 200 26,80 5.360,00 C
            1-BOVESPA V VISTA PETR4 PETROBRAS PN 50 37,10 1.855,00 D
        "#;
        let trades = SinacorParser::parse_trade_lines(sample_note, "XP Investimentos");
        let mut portfolio_tracker = PortfolioTracker::new();
        let mut tax_calc = TaxCalculator::new();
        let monthly_reports = tax_calc.compute_monthly_taxes(&trades, &mut portfolio_tracker);

        // Precompute Treemap nodes for major B3 sectors
        let (heatmap_nodes, heatmap_tickers) = Self::layout_heatmap(&stock_indicators);

        // Initialize Level 2 orderbook for PETR4
        let base_p = 36.40;
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
            symbol: "PETR4".to_string(),
            timestamp_ms: Utc::now().timestamp_millis(),
            bids,
            asks,
        };

        Self {
            active_view: ActiveView::StatusInvest,
            selected_ticker: "PETR4".to_string(),
            visible_tickers: vec![
                "PETR4".to_string(),
                "VALE3".to_string(),
                "WEGE3".to_string(),
                "ITUB4".to_string(),
            ],
            focused_ticker_idx: 0,
            search_query: String::new(),
            is_search_open: false,
            search_target_slot: None,
            b3_search_query: String::new(),
            b3_filter_class: None,
            just_opened_search: false,
            db,
            stock_indicators,
            company_profiles,
            fii_details,
            candles,
            current_prices,
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
        }
    }

    fn ensure_asset_loaded(&mut self, ticker: &str) {
        let sym = ticker.to_uppercase();
        if self.stock_indicators.contains_key(&sym) {
            return;
        }

        let ind = StockIndicators::default_for(&sym);
        self.current_prices.insert(sym.clone(), ind.price);
        self.stock_indicators.insert(sym.clone(), ind);
        self.company_profiles.insert(sym.clone(), CompanyProfile::default_for(&sym));

        if AssetClass::from_symbol(&sym) == AssetClass::Fii {
            self.fii_details.insert(sym.clone(), FiiDetails::default_for(&sym));
        }

        let base_price = self.stock_indicators.get(&sym).unwrap().price;
        let mut sym_candles = Vec::new();
        let mut p = base_price * 0.85;
        let now = Utc::now().timestamp_millis();
        for i in (0..180).rev() {
            let day_ms = now - (i * 86_400_000);
            let delta = ((i as f64 * 7.3).sin() * 0.035) * p;
            let open = p;
            let close = (p + delta).max(1.0);
            let high = open.max(close) + (p * 0.015);
            let low = (open.min(close) - (p * 0.012)).max(0.5);
            let vol = 15_000_000.0 + ((i as f64 * 3.1).cos().abs() * 30_000_000.0);
            p = close;
            sym_candles.push(Candle {
                timestamp_ms: day_ms,
                open,
                high,
                low,
                close,
                volume: vol,
                trades: 1200,
                adjusted_close: close,
            });
        }
        let _ = self.db.insert_quotes(&sym, &sym_candles);
        self.candles.insert(sym.clone(), sym_candles);
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
            let last_p = *self.current_prices.get(&sym).unwrap_or(&35.0);
            let tick = LiveFeedClient::simulate_tick(&sym, last_p);
            self.current_prices.insert(sym.clone(), tick.price);

            // Update indicators price & valuation
            if let Some(ind) = self.stock_indicators.get_mut(&sym) {
                ind.price = tick.price;
                if ind.lpa > 0.0 { ind.pl = tick.price / ind.lpa; }
                if ind.vpa > 0.0 { ind.pvp = tick.price / ind.vpa; }
                if tick.price > 0.0 { ind.dividend_yield = (ind.dpa / tick.price) * 100.0; }
            }

            // Update candle list
            if let Some(list) = self.candles.get_mut(&sym) {
                if let Some(last_candle) = list.last_mut() {
                    last_candle.close = tick.price;
                    last_candle.high = last_candle.high.max(tick.price);
                    last_candle.low = last_candle.low.min(tick.price);
                    last_candle.volume += tick.quantity * tick.price;
                }
            }

            if sym == self.selected_ticker {
                // Update orderbook top level
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

        // Random pulse in heatmap
        if !self.heatmap_nodes.is_empty() {
            let node_idx = (self.tick_counter as usize) % self.heatmap_nodes.len();
            self.heatmap_nodes[node_idx].flash_intensity = 1.0;
        }
    }
}

impl eframe::App for AppState {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.telemetry.on_frame();

        // High-frequency refresh simulation
        if self.is_live_sim_running {
            self.on_tick();
            // Decay heatmap flashes smoothly
            for n in &mut self.heatmap_nodes {
                n.flash_intensity = (n.flash_intensity * 0.88).max(0.0);
            }
            ui.ctx().request_repaint(); // Request next frame for 120Hz smooth animations
        }

        // Top Navigation & System Status Bar
        ui.horizontal(|ui| {
            ui.label(RichText::new("BROKER-AÇÕES").strong().color(Color32::from_rgb(0, 210, 160)).size(17.0));
            ui.label(RichText::new("| B3 HARDWARE TERMINAL").color(Color32::from_rgb(160, 175, 190)).size(13.0));

            ui.separator();

            // View Selector Tabs
            let views = [
                (ActiveView::StatusInvest, "Status Invest"),
                (ActiveView::GpuTerminal, "Terminal GPU"),
                (ActiveView::MarketHeatmap, "Mapa B3"),
                (ActiveView::Screener, "Screener SIMD"),
                (ActiveView::PortfolioTax, "Carteira & DARF"),
            ];

            for (view, label) in views {
                let is_active = self.active_view == view;
                if ui.selectable_label(is_active, RichText::new(label).strong()).clicked() {
                    self.active_view = view;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // GPU Telemetry Pill
                let telemetry_text = format!(
                    "Vulkan RADV | {:.0} FPS ({:.2}ms) | Telas: {}",
                    self.telemetry.fps, self.telemetry.frame_time_ms, self.visible_tickers.len()
                );
                ui.label(RichText::new(telemetry_text).monospace().color(Color32::from_rgb(0, 220, 140)).size(12.0));

                ui.separator();

                // Button to add another asset if below max
                if self.visible_tickers.len() < MAX_VISIBLE_ASSETS {
                    if ui.button(RichText::new("+ Adicionar").strong().color(Color32::from_rgb(0, 210, 160))).clicked() {
                        self.search_target_slot = None;
                        self.b3_search_query.clear();
                        self.is_search_open = true;
                        self.just_opened_search = true;
                    }
                }

                // Render slots in reverse order because right_to_left layout
                let mut slot_to_remove: Option<usize> = None;
                let mut slot_to_change: Option<usize> = None;

                for i in (0..self.visible_tickers.len()).rev() {
                    let sym = self.visible_tickers[i].clone();
                    let is_focused = self.focused_ticker_idx == i;
                    let text = format!("{}: {}", i + 1, sym);
                    let chip_color = if is_focused {
                        Color32::from_rgb(0, 210, 160)
                    } else {
                        Color32::WHITE
                    };

                    if ui.button(RichText::new(text).strong().color(chip_color)).clicked() {
                        self.focused_ticker_idx = i;
                        self.selected_ticker = sym.clone();
                        slot_to_change = Some(i);
                    }

                    if self.visible_tickers.len() > 1 {
                        if ui.small_button("x").clicked() {
                            slot_to_remove = Some(i);
                        }
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
                }

                if let Some(s) = slot_to_change {
                    self.search_target_slot = Some(s);
                    self.b3_search_query.clear();
                    self.is_search_open = true;
                    self.just_opened_search = true;
                }

                ui.label(RichText::new(format!("Ativos na Tela ({}/{}):", self.visible_tickers.len(), MAX_VISIBLE_ASSETS)).color(Color32::GRAY));
            });
        });

        ui.separator();

        // Render search modal if open
        self.render_search_modal(ui.ctx());

        // Main View Content
        match self.active_view {
            ActiveView::StatusInvest => self.render_status_invest_view(ui),
            ActiveView::GpuTerminal => self.render_gpu_terminal_view(ui),
            ActiveView::MarketHeatmap => self.render_heatmap_view(ui),
            ActiveView::Screener => self.render_screener_view(ui),
            ActiveView::PortfolioTax => self.render_portfolio_tax_view(ui),
        }
    }
}

impl AppState {
    // -------------------------------------------------------------
    // -------------------------------------------------------------
    // B3 UNIVERSE SEARCH MODAL
    // -------------------------------------------------------------
    fn render_search_modal(&mut self, ctx: &egui::Context) {
        if !self.is_search_open {
            return;
        }

        let mut is_open = true;
        let mut chosen: Option<String> = None;

        egui::Window::new("Buscar Ativo B3")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([650.0, 480.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Pesquisa no Universo B3").strong().size(16.0).color(Color32::WHITE));
                    if let Some(slot) = self.search_target_slot {
                        ui.label(RichText::new(format!("(Alterando Slot {})", slot + 1)).color(Color32::from_rgb(0, 210, 160)));
                    } else {
                        ui.label(RichText::new(format!("(Adicionando ativo {}/{})", self.visible_tickers.len() + 1, MAX_VISIBLE_ASSETS)).color(Color32::from_rgb(0, 210, 160)));
                    }
                });

                ui.add_space(4.0);

                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.b3_search_query)
                        .hint_text("Digite código ou nome (ex: PETR4, Vale, HGLG11, Itau, Apple)...")
                        .desired_width(f32::INFINITY)
                );
                if self.just_opened_search {
                    response.request_focus();
                    self.just_opened_search = false;
                }

                ui.add_space(4.0);

                // Category filter chips
                ui.horizontal(|ui| {
                    ui.label("Tipo:");
                    if ui.selectable_label(self.b3_filter_class.is_none(), "Todos").clicked() {
                        self.b3_filter_class = None;
                    }
                    if ui.selectable_label(self.b3_filter_class == Some(AssetClass::Acao), "Ações").clicked() {
                        self.b3_filter_class = Some(AssetClass::Acao);
                    }
                    if ui.selectable_label(self.b3_filter_class == Some(AssetClass::Fii), "FIIs").clicked() {
                        self.b3_filter_class = Some(AssetClass::Fii);
                    }
                    if ui.selectable_label(self.b3_filter_class == Some(AssetClass::Etf), "ETFs").clicked() {
                        self.b3_filter_class = Some(AssetClass::Etf);
                    }
                    if ui.selectable_label(self.b3_filter_class == Some(AssetClass::Bdr), "BDRs").clicked() {
                        self.b3_filter_class = Some(AssetClass::Bdr);
                    }
                });

                ui.separator();

                let results = b3_core::b3_assets::search_b3_assets(&self.b3_search_query, self.b3_filter_class);
                ui.label(RichText::new(format!("Encontrados {} ativos na B3", results.len())).color(Color32::GRAY).size(12.0));

                ui.add_space(4.0);

                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    egui::Grid::new("b3_search_results_grid")
                        .striped(true)
                        .spacing([14.0, 6.0])
                        .min_col_width(70.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Código").strong());
                            ui.label(RichText::new("Nome do Ativo").strong());
                            ui.label(RichText::new("Setor").strong());
                            ui.label(RichText::new("Tipo").strong());
                            ui.label(RichText::new("Preço Ref.").strong());
                            ui.label(RichText::new("Ação").strong());
                            ui.end_row();

                            for asset in results.iter().take(60) {
                                ui.label(RichText::new(asset.ticker).strong().color(Color32::from_rgb(0, 210, 160)));
                                ui.label(asset.name);
                                ui.label(RichText::new(asset.sector).color(Color32::GRAY));
                                let class_str = match asset.asset_class {
                                    AssetClass::Acao => "Ação",
                                    AssetClass::Fii => "FII",
                                    AssetClass::Etf => "ETF",
                                    AssetClass::Bdr => "BDR",
                                    _ => "Outro",
                                };
                                ui.label(class_str);
                                ui.label(format!("R$ {:.2}", asset.base_price));

                                let is_on_screen = self.visible_tickers.iter().any(|t| t == asset.ticker);
                                let btn_text = if is_on_screen { "Na Tela" } else { "Selecionar" };
                                if ui.button(btn_text).clicked() {
                                    chosen = Some(asset.ticker.to_string());
                                }
                                ui.end_row();
                            }
                        });
                });
            });

        if let Some(sym) = chosen {
            self.apply_selected_ticker(&sym);
            self.is_search_open = false;
        } else {
            self.is_search_open = is_open;
        }
    }

    // -------------------------------------------------------------
    // VIEW 1: STATUS INVEST FUNDAMENTAL DASHBOARD
    // -------------------------------------------------------------
    fn render_status_invest_view(&mut self, ui: &mut egui::Ui) {
        let count = self.visible_tickers.len();
        if count == 1 {
            let sym = self.visible_tickers[0].clone();
            self.render_single_status_invest(ui, &sym);
            return;
        }

        // Multi-Asset Comparison View (up to MAX_VISIBLE_ASSETS side-by-side)
        ui.horizontal(|ui| {
            ui.heading(RichText::new(format!("Painel Comparativo de Ativos B3 ({} na tela)", count)).size(18.0).color(Color32::WHITE));
            ui.separator();
            ui.label(RichText::new("Análise fundamentalista de múltiplos ativos simultaneamente").color(Color32::GRAY));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if count < MAX_VISIBLE_ASSETS {
                    if ui.button(RichText::new("+ Adicionar para Comparar").strong().color(Color32::from_rgb(0, 210, 160))).clicked() {
                        self.search_target_slot = None;
                        self.b3_search_query.clear();
                        self.is_search_open = true;
                        self.just_opened_search = true;
                    }
                }
            });
        });

        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut ticker_to_focus: Option<String> = None;
            let mut slot_to_change: Option<usize> = None;
            let mut slot_to_remove: Option<usize> = None;

            ui.columns(count, |cols| {
                for (idx, sym) in self.visible_tickers.iter().enumerate() {
                    let col = &mut cols[idx];
                    let ind = self.stock_indicators.get(sym).cloned().unwrap_or_else(|| StockIndicators::default_for(sym));
                    let profile = self.company_profiles.get(sym).cloned().unwrap_or_else(|| CompanyProfile::default_for(sym));
                    let is_fii = AssetClass::from_symbol(sym) == AssetClass::Fii;

                    egui::Frame::group(col.style())
                        .fill(Color32::from_rgb(22, 26, 36))
                        .inner_margin(8.0)
                        .show(col, |ui| {
                            // Column Header
                            ui.horizontal(|ui| {
                                ui.heading(RichText::new(sym).strong().color(Color32::from_rgb(0, 210, 160)).size(18.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("R$ {:.2}", ind.price)).strong().color(Color32::WHITE).size(18.0));
                                });
                            });
                            ui.label(RichText::new(&profile.name).color(Color32::from_rgb(180, 195, 210)).size(12.0));
                            ui.label(RichText::new(&profile.subsector).color(Color32::GRAY).size(11.0));

                            ui.add_space(4.0);

                            // Slot Action Bar
                            ui.horizontal(|ui| {
                                if ui.small_button("Focar").clicked() {
                                    ticker_to_focus = Some(sym.clone());
                                }
                                if ui.small_button("Trocar").clicked() {
                                    slot_to_change = Some(idx);
                                }
                                if self.visible_tickers.len() > 1 {
                                    if ui.small_button("Remover").clicked() {
                                        slot_to_remove = Some(idx);
                                    }
                                }
                            });

                            ui.separator();

                            // Graham Card
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(26, 30, 42))
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Preço Justo Graham").strong().size(11.0));
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("R$ {:.2}", ind.graham_fair_value)).strong().color(Color32::from_rgb(0, 210, 160)));
                                        let upside = if ind.price > 0.0 { ((ind.graham_fair_value - ind.price) / ind.price) * 100.0 } else { 0.0 };
                                        let up_col = if upside >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                                        ui.label(RichText::new(format!("{:+.1}%", upside)).color(up_col).size(11.0));
                                    });
                                });

                            ui.add_space(4.0);

                            // Bazin Card
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(26, 30, 42))
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Preço Teto Bazin (6%)").strong().size(11.0));
                                    ui.horizontal(|ui| {
                                        let bazin_p = ind.dpa / 0.06;
                                        ui.label(RichText::new(format!("R$ {:.2}", bazin_p)).strong().color(Color32::from_rgb(100, 180, 255)));
                                        let margin = if bazin_p > 0.0 { ((bazin_p - ind.price) / bazin_p) * 100.0 } else { 0.0 };
                                        ui.label(RichText::new(format!("Margem: {:.1}%", margin)).size(11.0));
                                    });
                                });

                            ui.add_space(4.0);

                            // Piotroski & Altman
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(26, 30, 42))
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("Piotroski: {}/9", ind.piotroski_f_score)).strong().color(Color32::from_rgb(0, 220, 140)).size(11.0));
                                        if ind.altman_z_score > 0.0 {
                                            ui.label(RichText::new(format!("Z: {:.2}", ind.altman_z_score)).size(11.0));
                                        }
                                    });
                                });

                            ui.add_space(4.0);

                            // FII specific card if applicable
                            if is_fii {
                                if let Some(fii) = self.fii_details.get(sym) {
                                    egui::Frame::group(ui.style())
                                        .fill(Color32::from_rgb(24, 34, 46))
                                        .inner_margin(6.0)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new(format!("FII: {}", fii.segment.display_name())).strong().color(Color32::from_rgb(0, 210, 160)).size(11.0));
                                            ui.label(format!("Vac. Física: {:.1}% | Fin.: {:.1}%", fii.physical_vacancy_percent, fii.financial_vacancy_percent));
                                            ui.label(format!("Rend.: R$ {:.2} | Cotistas: {}", fii.last_dividend, fii.shareholders_count));
                                        });
                                    ui.add_space(4.0);
                                }
                            }

                            // Key indicators mini-grid
                            ui.label(RichText::new("Métricas Rápidas:").strong().size(11.0));
                            egui::Grid::new(format!("mini_grid_{}", sym))
                                .spacing([10.0, 3.0])
                                .show(ui, |ui| {
                                    ui.label("P/L:"); ui.label(RichText::new(format!("{:.2}", ind.pl)).strong()); ui.end_row();
                                    ui.label("P/VP:"); ui.label(RichText::new(format!("{:.2}", ind.pvp)).strong()); ui.end_row();
                                    ui.label("EV/EBITDA:"); ui.label(format!("{:.2}", ind.ev_ebitda)); ui.end_row();
                                    ui.label("Div. Yield:"); ui.label(RichText::new(format!("{:.2}%", ind.dividend_yield)).color(Color32::from_rgb(0, 220, 140))); ui.end_row();
                                    ui.label("ROE:"); ui.label(format!("{:.1}%", ind.roe)); ui.end_row();
                                    ui.label("ROIC:"); ui.label(format!("{:.1}%", ind.roic)); ui.end_row();
                                    ui.label("Mg. Líquida:"); ui.label(format!("{:.1}%", ind.margem_liquida)); ui.end_row();
                                    ui.label("Dív.Líq/EBITDA:"); ui.label(format!("{:.2}", ind.divida_liq_ebitda)); ui.end_row();
                                    ui.label("Liq. Corrente:"); ui.label(format!("{:.2}", ind.liquidez_corrente)); ui.end_row();
                                    ui.label("LPA:"); ui.label(format!("R$ {:.2}", ind.lpa)); ui.end_row();
                                    ui.label("VPA:"); ui.label(format!("R$ {:.2}", ind.vpa)); ui.end_row();
                                    ui.label("DPA:"); ui.label(format!("R$ {:.2}", ind.dpa)); ui.end_row();
                                });
                        });
                }
            });

            if let Some(t) = ticker_to_focus {
                self.visible_tickers = vec![t.clone()];
                self.selected_ticker = t;
                self.focused_ticker_idx = 0;
            }

            if let Some(s) = slot_to_change {
                self.search_target_slot = Some(s);
                self.b3_search_query.clear();
                self.is_search_open = true;
                self.just_opened_search = true;
            }

            if let Some(r) = slot_to_remove {
                self.visible_tickers.remove(r);
                if self.focused_ticker_idx >= self.visible_tickers.len() {
                    self.focused_ticker_idx = self.visible_tickers.len().saturating_sub(1);
                }
                if !self.visible_tickers.is_empty() {
                    self.selected_ticker = self.visible_tickers[self.focused_ticker_idx].clone();
                }
            }

            ui.add_space(12.0);

            // Detailed Side-by-Side Comparison Matrix Table
            self.render_comparison_table(ui);
        });
    }

    fn render_single_status_invest(&mut self, ui: &mut egui::Ui, sym: &str) {
        let profile = self.company_profiles.get(sym).cloned().unwrap_or_else(|| CompanyProfile::default_for(sym));
        let ind = self.stock_indicators.get(sym).cloned().unwrap_or_else(|| StockIndicators::default_for(sym));
        let is_fii = AssetClass::from_symbol(sym) == AssetClass::Fii;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Company Header Banner
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(22, 26, 35))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading(RichText::new(&profile.symbol).strong().size(26.0).color(Color32::WHITE));
                                ui.label(RichText::new(&profile.segment).background_color(Color32::from_rgb(40, 50, 70)).size(11.0));
                            });
                            ui.label(RichText::new(&profile.name).color(Color32::from_rgb(180, 195, 210)).size(14.0));
                            ui.label(RichText::new(format!("Setor: {}  |  Sub-setor: {}", profile.sector.display_name(), profile.subsector)).color(Color32::GRAY).size(12.0));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(format!("R$ {:.2}", ind.price)).strong().size(28.0).color(Color32::from_rgb(0, 220, 140)));
                                ui.label(RichText::new("Cotação Atual (B3)").color(Color32::GRAY).size(11.0));
                            });
                        });
                    });
                });

            ui.add_space(8.0);

            // Key KPI Highlight Cards (Status Invest Style)
            ui.horizontal(|ui| {
                let card_w = (ui.available_width() - 40.0) / 6.0;
                let kpis = [
                    ("DIVIDEND YIELD", format!("{:.2}%", ind.dividend_yield), Color32::from_rgb(0, 210, 160)),
                    ("P/L", format!("{:.2}", ind.pl), Color32::from_rgb(255, 200, 80)),
                    ("P/VP", format!("{:.2}", ind.pvp), Color32::from_rgb(100, 180, 255)),
                    ("EV/EBITDA", if ind.ev_ebitda > 0.0 { format!("{:.2}", ind.ev_ebitda) } else { "-".to_string() }, Color32::WHITE),
                    ("ROE", format!("{:.2}%", ind.roe), Color32::from_rgb(0, 210, 160)),
                    ("ROIC", if ind.roic > 0.0 { format!("{:.2}%", ind.roic) } else { "-".to_string() }, Color32::from_rgb(140, 220, 255)),
                ];

                for (label, val, color) in kpis {
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(26, 30, 42))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_width(card_w);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new(label).color(Color32::from_rgb(140, 155, 175)).size(11.0));
                                ui.label(RichText::new(val).strong().color(color).size(18.0));
                            });
                        });
                }
            });

            ui.add_space(10.0);

            // Valuation & Health Models Grid (Graham, Bazin, Piotroski, Altman)
            ui.horizontal(|ui| {
                let col_w = (ui.available_width() - 24.0) / 3.0;

                // Benjamin Graham Card
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 28, 38))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(col_w);
                        ui.label(RichText::new("Preço Justo de Graham").strong().color(Color32::WHITE).size(14.0));
                        ui.label(RichText::new("Fórmula: sqrt(22.5 * LPA * VPA)").color(Color32::GRAY).size(11.0));
                        ui.add_space(4.0);
                        ui.label(RichText::new(format!("R$ {:.2}", ind.graham_fair_value)).strong().color(Color32::from_rgb(0, 210, 160)).size(20.0));
                        let upside = if ind.price > 0.0 { ((ind.graham_fair_value - ind.price) / ind.price) * 100.0 } else { 0.0 };
                        let color = if upside >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                        ui.label(RichText::new(format!("Margem / Upside: {:+.1}%", upside)).color(color).size(12.0));
                    });

                // Décio Bazin Card
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 28, 38))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(col_w);
                        ui.label(RichText::new("Preço Teto Bazin").strong().color(Color32::WHITE).size(14.0));
                        ui.horizontal(|ui| {
                            ui.label("Yield Alvo:");
                            ui.add(egui::Slider::new(&mut self.bazin_target_yield, 4.0..=12.0).suffix("%"));
                        });
                        let bazin_price = if self.bazin_target_yield > 0.0 { ind.dpa / (self.bazin_target_yield / 100.0) } else { 0.0 };
                        ui.label(RichText::new(format!("R$ {:.2}", bazin_price)).strong().color(Color32::from_rgb(100, 180, 255)).size(20.0));
                        let margin = if bazin_price > 0.0 { ((bazin_price - ind.price) / bazin_price) * 100.0 } else { 0.0 };
                        ui.label(RichText::new(format!("Margem de Segurança: {:.1}%", margin)).color(Color32::WHITE).size(12.0));
                    });

                // Piotroski & Altman Score Card
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 28, 38))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(col_w);
                        ui.label(RichText::new("Score Piotroski & Altman Z").strong().color(Color32::WHITE).size(14.0));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("F-Score: {}/9", ind.piotroski_f_score)).strong().color(Color32::from_rgb(0, 220, 140)).size(16.0));
                            let status = if ind.piotroski_f_score >= 7 { "Forte / Saudável" } else { "Moderado" };
                            ui.label(RichText::new(status).color(Color32::GRAY).size(12.0));
                        });
                        ui.add_space(4.0);
                        if ind.altman_z_score > 0.0 {
                            let zone = if ind.altman_z_score > 2.99 { "Zona Segura (> 2.99)" } else if ind.altman_z_score > 1.81 { "Zona Cinzenta" } else { "Alerta" };
                            ui.label(RichText::new(format!("Altman Z: {:.2} ({})", ind.altman_z_score, zone)).size(12.0));
                        } else {
                            ui.label(RichText::new("Altman Z: N/A para instituições financeiras").color(Color32::GRAY).size(11.0));
                        }
                    });
            });

            ui.add_space(12.0);

            // Special FII Panel if applicable
            if is_fii {
                if let Some(fii) = self.fii_details.get(sym) {
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(24, 32, 44))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.heading(RichText::new("Raio-X do Fundo Imobiliário (FII)").color(Color32::from_rgb(0, 210, 160)).size(16.0));
                            ui.horizontal(|ui| {
                                ui.label(format!("Segmento: {}", fii.segment.display_name()));
                                ui.separator();
                                ui.label(format!("Vacância Física: {:.1}%", fii.physical_vacancy_percent));
                                ui.separator();
                                ui.label(format!("Vacância Financeira: {:.1}%", fii.financial_vacancy_percent));
                                ui.separator();
                                ui.label(format!("Último Rendimento: R$ {:.2}/cota", fii.last_dividend));
                                ui.separator();
                                ui.label(format!("Cotistas: {}", fii.shareholders_count));
                            });
                            ui.add_space(6.0);
                            ui.label(RichText::new("Imóveis do Portfólio:").strong());
                            for p in &fii.properties {
                                ui.label(format!("- {} - {}, {} (Área: {:.0} m2, Vacância: {:.1}%)", p.name, p.city, p.state, p.area_m2, p.vacancy_percent));
                            }
                        });
                    ui.add_space(10.0);
                }
            }

            // Status Invest Comprehensive 30+ Indicators Table
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(22, 26, 36))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.heading(RichText::new("Matriz Completa de Indicadores Fundamentalistas").color(Color32::WHITE).size(16.0));
                    ui.add_space(6.0);

                    ui.columns(4, |cols| {
                        // Col 1: Valuation
                        cols[0].label(RichText::new("VALUATION").strong().color(Color32::from_rgb(100, 180, 255)));
                        cols[0].label(format!("P/L: {:.2}", ind.pl));
                        cols[0].label(format!("P/VP: {:.2}", ind.pvp));
                        cols[0].label(format!("EV/EBITDA: {:.2}", ind.ev_ebitda));
                        cols[0].label(format!("EV/EBIT: {:.2}", ind.ev_ebit));
                        cols[0].label(format!("P/EBITDA: {:.2}", ind.p_ebitda));
                        cols[0].label(format!("P/SR: {:.2}", ind.p_sr));
                        cols[0].label(format!("P/Ativo: {:.2}", ind.p_ativo));
                        cols[0].label(format!("P/Cap. Giro: {:.2}", ind.p_cap_giro));

                        // Col 2: Endividamento & Liquidez
                        cols[1].label(RichText::new("ENDIVIDAMENTO").strong().color(Color32::from_rgb(255, 180, 100)));
                        cols[1].label(format!("Dív. Líq./PL: {:.2}", ind.divida_liq_pl));
                        cols[1].label(format!("Dív. Líq./EBITDA: {:.2}", ind.divida_liq_ebitda));
                        cols[1].label(format!("Dív. Líq./EBIT: {:.2}", ind.divida_liq_ebit));
                        cols[1].label(format!("PL / Ativos: {:.2}", ind.pl_ativos));
                        cols[1].label(format!("Passivos / Ativos: {:.2}", ind.passivos_ativos));
                        cols[1].label(format!("Liq. Corrente: {:.2}", ind.liquidez_corrente));
                        cols[1].label(format!("Liq. Seca: {:.2}", ind.liquidez_seca));

                        // Col 3: Eficiência Operacional
                        cols[2].label(RichText::new("EFICIÊNCIA").strong().color(Color32::from_rgb(0, 220, 140)));
                        cols[2].label(format!("Margem Bruta: {:.1}%", ind.margem_bruta));
                        cols[2].label(format!("Margem EBITDA: {:.1}%", ind.margem_ebitda));
                        cols[2].label(format!("Margem EBIT: {:.1}%", ind.margem_ebit));
                        cols[2].label(format!("Margem Líquida: {:.1}%", ind.margem_liquida));
                        cols[2].label(format!("Giro Ativos: {:.2}", ind.giro_ativos));

                        // Col 4: Rentabilidade & Crescimento
                        cols[3].label(RichText::new("RENTABILIDADE & 5A").strong().color(Color32::from_rgb(180, 140, 255)));
                        cols[3].label(format!("ROE: {:.1}%", ind.roe));
                        cols[3].label(format!("ROIC: {:.1}%", ind.roic));
                        cols[3].label(format!("ROA: {:.1}%", ind.roa));
                        cols[3].label(format!("CAGR Receitas: {:.1}%", ind.cagr_receitas_5a));
                        cols[3].label(format!("CAGR Lucros: {:.1}%", ind.cagr_lucros_5a));
                        cols[3].label(format!("LPA: R$ {:.2}", ind.lpa));
                        cols[3].label(format!("VPA: R$ {:.2}", ind.vpa));
                        cols[3].label(format!("DPA: R$ {:.2}", ind.dpa));
                    });
                });
        });
    }

    fn render_comparison_table(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(22, 26, 36))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading(RichText::new("Matriz Comparativa de Indicadores Fundamentalistas").color(Color32::WHITE).size(16.0));
                ui.add_space(6.0);

                let indicators: Vec<StockIndicators> = self.visible_tickers
                    .iter()
                    .map(|t| self.stock_indicators.get(t).cloned().unwrap_or_else(|| StockIndicators::default_for(t)))
                    .collect();

                let metrics: &[(&str, fn(&StockIndicators) -> String)] = &[
                    ("Preço Atual", |i| format!("R$ {:.2}", i.price)),
                    ("P/L", |i| format!("{:.2}", i.pl)),
                    ("P/VP", |i| format!("{:.2}", i.pvp)),
                    ("EV/EBITDA", |i| format!("{:.2}", i.ev_ebitda)),
                    ("EV/EBIT", |i| format!("{:.2}", i.ev_ebit)),
                    ("P/SR", |i| format!("{:.2}", i.p_sr)),
                    ("P/Ativo", |i| format!("{:.2}", i.p_ativo)),
                    ("Dividend Yield", |i| format!("{:.2}%", i.dividend_yield)),
                    ("ROE", |i| format!("{:.2}%", i.roe)),
                    ("ROIC", |i| format!("{:.2}%", i.roic)),
                    ("ROA", |i| format!("{:.2}%", i.roa)),
                    ("Margem Bruta", |i| format!("{:.1}%", i.margem_bruta)),
                    ("Margem EBITDA", |i| format!("{:.1}%", i.margem_ebitda)),
                    ("Margem EBIT", |i| format!("{:.1}%", i.margem_ebit)),
                    ("Margem Líquida", |i| format!("{:.1}%", i.margem_liquida)),
                    ("Dívida Líq./PL", |i| format!("{:.2}", i.divida_liq_pl)),
                    ("Dívida Líq./EBITDA", |i| format!("{:.2}", i.divida_liq_ebitda)),
                    ("Liquidez Corrente", |i| format!("{:.2}", i.liquidez_corrente)),
                    ("Liquidez Seca", |i| format!("{:.2}", i.liquidez_seca)),
                    ("LPA", |i| format!("R$ {:.2}", i.lpa)),
                    ("VPA", |i| format!("R$ {:.2}", i.vpa)),
                    ("DPA", |i| format!("R$ {:.2}", i.dpa)),
                    ("Preço Justo Graham", |i| format!("R$ {:.2}", i.graham_fair_value)),
                    ("Preço Teto Bazin", |i| format!("R$ {:.2}", i.bazin_ceiling_price)),
                    ("Score Piotroski", |i| format!("{}/9", i.piotroski_f_score)),
                    ("Altman Z-Score", |i| if i.altman_z_score > 0.0 { format!("{:.2}", i.altman_z_score) } else { "-".to_string() }),
                ];

                egui::Grid::new("comparison_matrix_grid")
                    .striped(true)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Indicador").strong());
                        for t in &self.visible_tickers {
                            ui.label(RichText::new(t).strong().color(Color32::from_rgb(0, 210, 160)));
                        }
                        ui.end_row();

                        for (name, extractor) in metrics {
                            ui.label(*name);
                            for ind in &indicators {
                                ui.label(extractor(ind));
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    // -------------------------------------------------------------
    // VIEW 2: HARDWARE-ACCELERATED GPU TERMINAL
    // -------------------------------------------------------------
    fn render_gpu_terminal_view(&mut self, ui: &mut egui::Ui) {
        let count = self.visible_tickers.len();

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Terminal GPU ({} gráficos na tela)", count)).strong().size(15.0));
            ui.separator();
            ui.label("Largura do Candle:");
            ui.add(egui::Slider::new(&mut self.chart_candle_width, 3.0..=20.0));
            ui.separator();
            ui.checkbox(&mut self.is_live_sim_running, "Feed Real-Time (Ticks ao Vivo)");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if count < MAX_VISIBLE_ASSETS {
                    if ui.button(RichText::new("+ Adicionar Gráfico").strong().color(Color32::from_rgb(0, 210, 160))).clicked() {
                        self.search_target_slot = None;
                        self.b3_search_query.clear();
                        self.is_search_open = true;
                        self.just_opened_search = true;
                    }
                }
            });
        });

        ui.add_space(4.0);

        if count == 1 {
            let sym = self.visible_tickers[0].clone();
            self.render_single_chart_terminal(ui, &sym);
        } else {
            // Multi-chart Grid Layout (2 columns for 2 assets, 2x2 for 3 or 4 assets)
            let available_w = ui.available_width();
            let available_h = ui.available_height().max(500.0);

            let (cols, rows) = if count == 2 {
                (2, 1)
            } else {
                (2, 2)
            };

            let cell_w = (available_w - ((cols as f32 - 1.0) * 8.0)) / cols as f32;
            let cell_h = (available_h - ((rows as f32 - 1.0) * 8.0)) / rows as f32;

            let mut slot_to_change: Option<usize> = None;
            let mut ticker_to_focus: Option<String> = None;

            for r in 0..rows {
                ui.horizontal(|ui| {
                    for c in 0..cols {
                        let idx = r * cols + c;
                        if idx < count {
                            let sym = self.visible_tickers[idx].clone();
                            let price = *self.current_prices.get(&sym).unwrap_or(&0.0);

                            ui.vertical(|ui| {
                                ui.set_width(cell_w);
                                ui.set_height(cell_h);

                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(18, 22, 30))
                                    .inner_margin(6.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("{}: {}", idx + 1, sym)).strong().color(Color32::from_rgb(0, 210, 160)).size(15.0));
                                            ui.label(RichText::new(format!("R$ {:.2}", price)).strong().color(Color32::WHITE).size(15.0));

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("Trocar").clicked() {
                                                    slot_to_change = Some(idx);
                                                }
                                                if ui.small_button("Focar").clicked() {
                                                    ticker_to_focus = Some(sym.clone());
                                                }
                                            });
                                        });

                                        let chart_rect_h = (cell_h - 38.0).max(120.0);
                                        let (rect, _response) = ui.allocate_exact_size(Vec2::new(cell_w - 14.0, chart_rect_h), Sense::hover());

                                        if let Some(list) = self.candles.get(&sym) {
                                            let start = (list.len() as f32 - 60.0).max(0.0) as usize;
                                            let slice = &list[start..];

                                            let mut min_p = f64::MAX;
                                            let mut max_p = f64::MIN;
                                            for candle in slice {
                                                min_p = min_p.min(candle.low);
                                                max_p = max_p.max(candle.high);
                                            }
                                            min_p *= 0.98;
                                            max_p *= 1.02;

                                            let gpu_candles: Vec<GpuCandle> = slice.iter().map(|c| c.to_gpu_candle(0.0)).collect();

                                            let uniforms = ChartUniforms {
                                                view_width: rect.width(),
                                                view_height: rect.height(),
                                                min_price: min_p as f32,
                                                max_price: max_p as f32,
                                                start_idx: 0.0,
                                                end_idx: gpu_candles.len() as f32,
                                                candle_width: self.chart_candle_width.min(10.0),
                                                wick_width: 1.5,
                                            };

                                            let callback = egui_wgpu::Callback::new_paint_callback(
                                                rect,
                                                GpuChartCallback {
                                                    pipeline: self.candlestick_pipe.clone(),
                                                    candles: gpu_candles,
                                                    uniforms,
                                                },
                                            );
                                            ui.painter().add(callback);
                                        }
                                    });
                            });
                        }
                    }
                });
                ui.add_space(4.0);
            }

            if let Some(t) = ticker_to_focus {
                self.visible_tickers = vec![t.clone()];
                self.selected_ticker = t;
                self.focused_ticker_idx = 0;
            }

            if let Some(s) = slot_to_change {
                self.search_target_slot = Some(s);
                self.b3_search_query.clear();
                self.is_search_open = true;
                self.just_opened_search = true;
            }
        }
    }

    fn render_single_chart_terminal(&mut self, ui: &mut egui::Ui, sym: &str) {
        ui.horizontal(|ui| {
            // Left 75%: GPU Candlestick Canvas
            let chart_w = ui.available_width() * 0.72;
            let chart_h = ui.available_height().max(400.0);

            let (rect, _response) = ui.allocate_exact_size(Vec2::new(chart_w, chart_h), Sense::hover());

            // Prepare GPU candle slice
            if let Some(list) = self.candles.get(sym) {
                let start = (list.len() as f32 - 70.0).max(0.0) as usize;
                let slice = &list[start..];

                let mut min_p = f64::MAX;
                let mut max_p = f64::MIN;
                for c in slice {
                    min_p = min_p.min(c.low);
                    max_p = max_p.max(c.high);
                }
                min_p *= 0.98;
                max_p *= 1.02;

                let gpu_candles: Vec<GpuCandle> = slice.iter().map(|c| c.to_gpu_candle(0.0)).collect();

                let uniforms = ChartUniforms {
                    view_width: rect.width(),
                    view_height: rect.height(),
                    min_price: min_p as f32,
                    max_price: max_p as f32,
                    start_idx: 0.0,
                    end_idx: gpu_candles.len() as f32,
                    candle_width: self.chart_candle_width,
                    wick_width: 1.5,
                };

                let callback = egui_wgpu::Callback::new_paint_callback(
                    rect,
                    GpuChartCallback {
                        pipeline: self.candlestick_pipe.clone(),
                        candles: gpu_candles,
                        uniforms,
                    },
                );

                ui.painter().add(callback);
            }

            ui.separator();

            // Right 25%: Order Book & Times and Trades
            ui.vertical(|ui| {
                ui.heading(RichText::new("Livro de Ofertas (Level 2)").size(15.0));

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(20, 24, 32))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.columns(2, |cols| {
                            cols[0].label(RichText::new("COMPRA (Bids)").color(Color32::from_rgb(0, 220, 140)).size(11.0));
                            for b in &self.order_book.bids {
                                cols[0].label(RichText::new(format!("{:.2} | {:.0}", b.price, b.quantity)).size(11.0));
                            }

                            cols[1].label(RichText::new("VENDA (Asks)").color(Color32::from_rgb(255, 80, 90)).size(11.0));
                            for a in &self.order_book.asks {
                                cols[1].label(RichText::new(format!("{:.2} | {:.0}", a.price, a.quantity)).size(11.0));
                            }
                        });
                    });

                ui.add_space(8.0);
                ui.heading(RichText::new("Times & Trades (Últimos Ticks)").size(15.0));

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(20, 24, 32))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                            for tick in &self.recent_ticks {
                                let (col, aggressor) = if tick.is_buyer_aggressor {
                                    (Color32::from_rgb(0, 220, 140), "C")
                                } else {
                                    (Color32::from_rgb(255, 80, 90), "V")
                                };
                                ui.label(
                                    RichText::new(format!("R$ {:.2} | {:.0} un | {} [C:{} V:{}]", tick.price, tick.quantity, aggressor, tick.buyer_broker, tick.seller_broker))
                                        .color(col)
                                        .size(11.0),
                                );
                            }
                        });
                    });
            });
        });
    }

    // -------------------------------------------------------------
    // VIEW 3: GPU MARKET MAP (SECTOR TREEMAP)
    // -------------------------------------------------------------
    fn render_heatmap_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Mapa de Mercado B3 (Heatmap GPU)").color(Color32::WHITE).size(18.0));
            ui.separator();
            ui.label(RichText::new("Verde = Alta | Vermelho = Baixa | Pulso Branco = Ticks em Tempo Real").color(Color32::GRAY));
        });

        ui.add_space(6.0);

        let area_w = ui.available_width();
        let area_h = ui.available_height().max(420.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(area_w, area_h), Sense::click());

        // Check clicks on treemap cells to switch active ticker
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let norm_x = (pos.x - rect.left()) / rect.width();
                let norm_y = (pos.y - rect.top()) / rect.height();

                let mut clicked_ticker = None;
                for (idx, node) in self.heatmap_nodes.iter().enumerate() {
                    if norm_x >= node.x && norm_x <= (node.x + node.w) && norm_y >= node.y && norm_y <= (node.y + node.h) {
                        if let Some(t) = self.heatmap_tickers.get(idx) {
                            clicked_ticker = Some(t.clone());
                            break;
                        }
                    }
                }
                if let Some(t) = clicked_ticker {
                    self.apply_selected_ticker(&t);
                    self.active_view = ActiveView::StatusInvest;
                }
            }
        }

        // Render with GPU Treemap pipeline
        let uniforms = TreemapUniforms {
            view_width: rect.width(),
            view_height: rect.height(),
            time_sec: self.telemetry.last_frame_time.elapsed().as_secs_f32(),
            _pad: 0.0,
        };

        let callback = egui_wgpu::Callback::new_paint_callback(
            rect,
            GpuTreemapCallback {
                pipeline: self.treemap_pipe.clone(),
                nodes: self.heatmap_nodes.clone(),
                uniforms,
            },
        );
        ui.painter().add(callback);

        // Overlay crisp text labels on top of GPU cells
        for (idx, node) in self.heatmap_nodes.iter().enumerate() {
            let px = rect.left() + node.x * rect.width() + 10.0;
            let py = rect.top() + node.y * rect.height() + 10.0;
            if let Some(t) = self.heatmap_tickers.get(idx) {
                let sign = if node.change_pct >= 0.0 { "+" } else { "" };
                ui.painter().text(
                    egui::pos2(px, py),
                    egui::Align2::LEFT_TOP,
                    format!("{}\n{}{:.2}%", t, sign, node.change_pct),
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
            }
        }
    }

    // -------------------------------------------------------------
    // VIEW 4: MULTI-FACTOR SIMD SCREENER
    // -------------------------------------------------------------
    fn render_screener_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Screener Fundamentalista Multi-Fator").size(18.0));
            ui.separator();
            ui.label(RichText::new(format!("Estratégia: {}", self.screener_preset_name)).strong().color(Color32::from_rgb(0, 210, 160)));
        });

        ui.add_space(6.0);

        // Preset Strategy Buttons
        ui.horizontal(|ui| {
            if ui.button("Fórmula Mágica (Greenblatt)").clicked() {
                self.screener_criteria = ScreenerCriteria::default();
                self.screener_preset_name = "Fórmula Mágica (Greenblatt)".to_string();
            }
            if ui.button("Método Bazin (Preço Teto)").clicked() {
                self.screener_criteria = ScreenerEngine::bazin_preset();
                self.screener_preset_name = "Método Décio Bazin (DY >= 6%)".to_string();
            }
            if ui.button("Graham Deep Value").clicked() {
                self.screener_criteria = ScreenerEngine::graham_preset();
                self.screener_preset_name = "Benjamin Graham (P/L <= 15, P/VP <= 1.5)".to_string();
            }
            if ui.button("Limpar Filtros").clicked() {
                self.screener_criteria = ScreenerCriteria::default();
                self.screener_preset_name = "Todos os Ativos".to_string();
            }
        });

        ui.add_space(6.0);

        // Filter sliders
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(22, 26, 36))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let mut dy_val = self.screener_criteria.min_dy.unwrap_or(0.0);
                    if ui.add(egui::Slider::new(&mut dy_val, 0.0..=20.0).prefix("Min DY: ").suffix("%")).changed() {
                        self.screener_criteria.min_dy = if dy_val > 0.0 { Some(dy_val) } else { None };
                    }

                    let mut pl_val = self.screener_criteria.max_pl.unwrap_or(50.0);
                    if ui.add(egui::Slider::new(&mut pl_val, 2.0..=50.0).prefix("Max P/L: ")).changed() {
                        self.screener_criteria.max_pl = if pl_val < 50.0 { Some(pl_val) } else { None };
                    }

                    let mut pvp_val = self.screener_criteria.max_pvp.unwrap_or(10.0);
                    if ui.add(egui::Slider::new(&mut pvp_val, 0.5..=10.0).prefix("Max P/VP: ")).changed() {
                        self.screener_criteria.max_pvp = if pvp_val < 10.0 { Some(pvp_val) } else { None };
                    }

                    let mut roe_val = self.screener_criteria.min_roe.unwrap_or(0.0);
                    if ui.add(egui::Slider::new(&mut roe_val, 0.0..=40.0).prefix("Min ROE: ").suffix("%")).changed() {
                        self.screener_criteria.min_roe = if roe_val > 0.0 { Some(roe_val) } else { None };
                    }
                });
            });

        ui.add_space(8.0);

        // Filter all loaded indicators
        let all_items: Vec<StockIndicators> = self.stock_indicators.values().cloned().collect();
        let filtered: Vec<&StockIndicators> = if self.screener_preset_name.starts_with("Fórmula Mágica") {
            ScreenerEngine::magic_formula(&all_items)
        } else {
            ScreenerEngine::filter(&all_items, &self.screener_criteria)
        };

        // Results Table
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("screener_grid")
                .striped(true)
                .spacing([14.0, 8.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Ticker").strong());
                    ui.label(RichText::new("Preço").strong());
                    ui.label(RichText::new("DY").strong());
                    ui.label(RichText::new("P/L").strong());
                    ui.label(RichText::new("P/VP").strong());
                    ui.label(RichText::new("EV/EBITDA").strong());
                    ui.label(RichText::new("ROE").strong());
                    ui.label(RichText::new("Mg. Líquida").strong());
                    ui.label(RichText::new("Graham P.J.").strong());
                    ui.label(RichText::new("Bazin Teto").strong());
                    ui.label(RichText::new("Ação").strong());
                    ui.end_row();

                    for item in filtered {
                        ui.label(RichText::new(&item.symbol).strong().color(Color32::WHITE));
                        ui.label(format!("R$ {:.2}", item.price));
                        ui.label(RichText::new(format!("{:.2}%", item.dividend_yield)).color(Color32::from_rgb(0, 210, 160)));
                        ui.label(format!("{:.2}", item.pl));
                        ui.label(format!("{:.2}", item.pvp));
                        ui.label(if item.ev_ebitda > 0.0 { format!("{:.2}", item.ev_ebitda) } else { "-".to_string() });
                        ui.label(format!("{:.1}%", item.roe));
                        ui.label(format!("{:.1}%", item.margem_liquida));
                        ui.label(format!("R$ {:.2}", item.graham_fair_value));
                        ui.label(format!("R$ {:.2}", item.bazin_ceiling_price));

                        if ui.button("Ver Detalhes").clicked() {
                            self.apply_selected_ticker(&item.symbol);
                            self.active_view = ActiveView::StatusInvest;
                        }
                        ui.end_row();
                    }
                });
        });
    }

    // -------------------------------------------------------------
    // VIEW 5: PORTFOLIO & BRAZILIAN TAX ENGINE (DARF / IRPF)
    // -------------------------------------------------------------
    fn render_portfolio_tax_view(&mut self, ui: &mut egui::Ui) {
        let positions = self.portfolio_tracker.get_positions(&self.current_prices);

        let total_invested: f64 = positions.iter().map(|p| p.total_invested).sum();
        let total_value: f64 = positions.iter().map(|p| p.current_value).sum();
        let total_pnl = total_value - total_invested;
        let total_pnl_pct = if total_invested > 0.0 { (total_pnl / total_invested) * 100.0 } else { 0.0 };

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Portfolio Summary Banner
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(22, 26, 36))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Patrimônio em Custódia B3").color(Color32::GRAY).size(12.0));
                            ui.heading(RichText::new(format!("R$ {:.2}", total_value)).size(26.0).color(Color32::WHITE));
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            ui.label(RichText::new("Total Aplicado (Preço Médio)").color(Color32::GRAY).size(12.0));
                            ui.label(RichText::new(format!("R$ {:.2}", total_invested)).size(18.0).color(Color32::from_rgb(180, 195, 210)));
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            let col = if total_pnl >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                            ui.label(RichText::new("Lucro / Prejuízo Não Realizado").color(Color32::GRAY).size(12.0));
                            ui.label(RichText::new(format!("R$ {:+.2} ({:+.2}%)", total_pnl, total_pnl_pct)).strong().size(18.0).color(col));
                        });
                    });
                });

            ui.add_space(10.0);

            // Open Positions Table
            ui.heading(RichText::new("Posições em Aberto").size(16.0));
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(24, 28, 38))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    egui::Grid::new("portfolio_positions_grid")
                        .striped(true)
                        .spacing([18.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Ticker").strong());
                            ui.label(RichText::new("Quantidade").strong());
                            ui.label(RichText::new("Preço Médio (PM)").strong());
                            ui.label(RichText::new("Cotação Atual").strong());
                            ui.label(RichText::new("Valor Aplicado").strong());
                            ui.label(RichText::new("Saldo Atual").strong());
                            ui.label(RichText::new("Retorno (R$)").strong());
                            ui.label(RichText::new("Retorno (%)").strong());
                            ui.end_row();

                            for p in &positions {
                                let col = if p.unrealized_pnl >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                                ui.label(RichText::new(&p.symbol).strong().color(Color32::WHITE));
                                ui.label(format!("{:.0}", p.quantity));
                                ui.label(format!("R$ {:.2}", p.average_price));
                                ui.label(format!("R$ {:.2}", p.current_price));
                                ui.label(format!("R$ {:.2}", p.total_invested));
                                ui.label(format!("R$ {:.2}", p.current_value));
                                ui.label(RichText::new(format!("R$ {:+.2}", p.unrealized_pnl)).color(col));
                                ui.label(RichText::new(format!("{:+.2}%", p.unrealized_pnl_percent)).color(col));
                                ui.end_row();
                            }
                        });
                });

            ui.add_space(12.0);

            // Monthly Tax & DARF (Code 6015) Engine
            ui.heading(RichText::new("Apuração Mensal de Imposto de Renda (DARF 6015)").color(Color32::from_rgb(0, 210, 160)).size(16.0));
            ui.label(RichText::new("Conforme normas da Receita Federal: Isenção de R$ 20.000/mês para Ações Swing Trade | Compensação de Prejuízos Acumulados").color(Color32::GRAY).size(11.0));

            ui.add_space(6.0);

            for report in &self.monthly_reports {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 30, 42))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(RichText::new(format!("Competência: {:02}/{}", report.month, report.year)).size(16.0).color(Color32::WHITE));
                            ui.separator();
                            let exempt_label = if report.is_acoes_exempt {
                                RichText::new("ISENTO (Vendas <= R$ 20.000)").color(Color32::from_rgb(0, 220, 140))
                            } else {
                                RichText::new("TRIBUTÁVEL (Vendas > R$ 20.000)").color(Color32::from_rgb(255, 180, 80))
                            };
                            ui.label(exempt_label);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let darf_col = if report.total_darf_due > 0.0 { Color32::from_rgb(255, 80, 90) } else { Color32::from_rgb(0, 220, 140) };
                                ui.label(RichText::new(format!("DARF 6015: R$ {:.2}", report.total_darf_due)).strong().size(18.0).color(darf_col));
                            });
                        });

                        ui.separator();

                        ui.columns(4, |cols| {
                            cols[0].label(format!("Total Vendas Ações: R$ {:.2}", report.acoes_swing_sales));
                            cols[0].label(format!("Resultado Ações: R$ {:+.2}", report.net_result_acoes_swing));

                            cols[1].label(format!("Day Trade (20%): R$ {:+.2}", report.net_result_day_trade));
                            cols[1].label(format!("FIIs (20%): R$ {:+.2}", report.net_result_fii));

                            cols[2].label(format!("Compensação Prejuízo: R$ {:.2}", report.loss_offset_acoes_swing));
                            cols[2].label(format!("IRRF Retido na Fonte: R$ {:.2}", report.irrf_deducted));

                            cols[3].label(RichText::new(format!("Vencimento: {}", report.due_date.format("%d/%m/%Y"))).color(Color32::from_rgb(100, 180, 255)));
                            cols[3].label(format!("Prejuízo Carregado: R$ {:.2}", report.carryover_loss_acoes_swing));
                        });
                    });
            }
        });
    }
}

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Broker Ações - Brazilian Stocks GPU Terminal & Fundamental Platform")
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
