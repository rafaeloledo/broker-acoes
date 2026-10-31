use eframe::egui::{self, Color32, RichText, Stroke};

use b3_analytics::{StockTrendAnalysis, TrendingEngine, TrendSignal};
use b3_core::b3_assets::B3_ASSET_CATALOG;
use b3_core::ticker::AssetClass;

use crate::theme::Palette;
use crate::{ActiveView, AppState, TrendingSort};

impl AppState {
    pub(crate) fn refresh_trending_cache(&mut self) {
        let now = chrono::Utc::now().timestamp_millis();
        if !self.trending_cache.is_empty() && now - self.trending_last_computed_ms < 800 {
            return;
        }

        let mut list = Vec::with_capacity(186);
        for asset in B3_ASSET_CATALOG {
            if self.trending_only_acoes && asset.asset_class != AssetClass::Acao {
                continue;
            }
            let sym = asset.ticker;
            let current_price = self.current_prices.get(sym).copied().unwrap_or(asset.base_price);
            let empty_candles = Vec::new();
            let candles = self.candles.get(sym).map(|v| v.as_slice()).unwrap_or(&empty_candles);
            let default_ind;
            let ind = if let Some(i) = self.stock_indicators.get(sym) {
                i
            } else {
                default_ind = b3_core::indicators::StockIndicators::default_for(sym);
                &default_ind
            };
            let core = self.core_multiples.get(sym);

            let analysis = TrendingEngine::analyze_stock(
                asset.ticker,
                asset.name,
                asset.sector,
                current_price,
                candles,
                ind,
                core,
            );
            list.push(analysis);
        }

        self.trending_cache = list;
        self.trending_last_computed_ms = now;
    }

    pub(crate) fn render_trending_view(&mut self, ui: &mut egui::Ui) {
        self.refresh_trending_cache();
        let palette = Palette::from_ui(ui);

        // Calculate summary counts across the entire universe
        let total_count = self.trending_cache.len();
        let strong_buy_count = self.trending_cache.iter().filter(|a| a.signal == TrendSignal::StrongBuy).count();
        let buy_count = self.trending_cache.iter().filter(|a| a.signal == TrendSignal::Buy).count();
        let neutral_count = self.trending_cache.iter().filter(|a| a.signal == TrendSignal::Neutral).count();
        let sell_count = self.trending_cache.iter().filter(|a| a.signal == TrendSignal::Sell || a.signal == TrendSignal::StrongSell).count();
        let avg_score = if total_count > 0 {
            self.trending_cache.iter().map(|a| a.overall_score).sum::<f64>() / total_count as f64
        } else {
            0.0
        };

        // Header Title & Context
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Trending B3 - Tendências em Tempo Real").strong().size(22.0).color(palette.text));
            ui.add_space(8.0);
            let live_bg = Color32::from_rgb(0, 80, 45);
            let live_fg = Color32::from_rgb(46, 230, 110);
            egui::Frame::new()
                .fill(live_bg)
                .corner_radius(4)
                .inner_margin(egui::Margin::symmetric(8, 3))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("● TEMPO REAL ATIVO ({total_count} ATIVOS)")).size(11.0).strong().color(live_fg));
                });
        });

        ui.label(
            RichText::new("Algoritmo quantitativo multifatorial em tempo real combinando IFR (RSI 14), Médias Móveis (SMA20/50), Bandas de Bollinger, Preço Justo de Graham e Preço Teto de Bazin.")
                .color(palette.muted)
                .size(12.0)
        );

        ui.add_space(8.0);

        // Top Summary Metric Cards
        ui.horizontal(|ui| {
            let card_w = (ui.available_width() - 32.0) / 5.0;

            render_summary_card(ui, "Forte Compra", &format!("{strong_buy_count} ativos"), "Assimetria máxima", Color32::from_rgb(46, 180, 80), palette.card_alt, card_w);
            render_summary_card(ui, "Compra", &format!("{buy_count} ativos"), "Tendência altista", Color32::from_rgb(80, 200, 120), palette.card_alt, card_w);
            render_summary_card(ui, "Neutro", &format!("{neutral_count} ativos"), "Consolidação", palette.muted, palette.card_alt, card_w);
            render_summary_card(ui, "Venda / Cautela", &format!("{sell_count} ativos"), "Pressão vendedora", palette.negative, palette.card_alt, card_w);
            render_summary_card(ui, "Score Médio B3", &format!("{avg_score:.1} / 100"), "Sentimento geral", palette.accent, palette.card_alt, card_w);
        });

        ui.add_space(10.0);

        // Search & Filter Toolbar
        egui::Frame::group(ui.style())
            .fill(palette.card)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Buscar:").strong().color(palette.label));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.trending_search)
                            .hint_text("Código ou nome (ex: PETR4, Vale, WEG, BBAS3)...")
                            .desired_width(260.0)
                    );

                    ui.separator();

                    ui.label(RichText::new("Sinal:").strong().color(palette.label));
                    if ui.selectable_label(self.trending_filter_signal.is_none(), "Todos").clicked() {
                        self.trending_filter_signal = None;
                    }
                    if ui.selectable_label(self.trending_filter_signal == Some(TrendSignal::StrongBuy), "Forte Compra").clicked() {
                        self.trending_filter_signal = Some(TrendSignal::StrongBuy);
                    }
                    if ui.selectable_label(self.trending_filter_signal == Some(TrendSignal::Buy), "Compra").clicked() {
                        self.trending_filter_signal = Some(TrendSignal::Buy);
                    }
                    if ui.selectable_label(self.trending_filter_signal == Some(TrendSignal::Neutral), "Neutro").clicked() {
                        self.trending_filter_signal = Some(TrendSignal::Neutral);
                    }
                    if ui.selectable_label(self.trending_filter_signal == Some(TrendSignal::Sell), "Venda").clicked() {
                        self.trending_filter_signal = Some(TrendSignal::Sell);
                    }
                    if ui.selectable_label(self.trending_filter_signal == Some(TrendSignal::StrongSell), "Forte Venda").clicked() {
                        self.trending_filter_signal = Some(TrendSignal::StrongSell);
                    }

                    ui.separator();

                    if ui.checkbox(&mut self.trending_only_acoes, "Somente Ações").changed() {
                        self.trending_last_computed_ms = 0;
                        self.refresh_trending_cache();
                    }
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Ordenar por:").color(palette.muted));
                    if ui.selectable_label(self.trending_sort == TrendingSort::ScoreDesc, "Maior Score").clicked() {
                        self.trending_sort = TrendingSort::ScoreDesc;
                    }
                    if ui.selectable_label(self.trending_sort == TrendingSort::GrahamUpsideDesc, "Potencial Graham").clicked() {
                        self.trending_sort = TrendingSort::GrahamUpsideDesc;
                    }
                    if ui.selectable_label(self.trending_sort == TrendingSort::BazinUpsideDesc, "Preço Teto Bazin").clicked() {
                        self.trending_sort = TrendingSort::BazinUpsideDesc;
                    }
                    if ui.selectable_label(self.trending_sort == TrendingSort::DividendYieldDesc, "Maior DY").clicked() {
                        self.trending_sort = TrendingSort::DividendYieldDesc;
                    }
                    if ui.selectable_label(self.trending_sort == TrendingSort::RsiAsc, "IFR (Sobrevenda)").clicked() {
                        self.trending_sort = TrendingSort::RsiAsc;
                    }
                    if ui.selectable_label(self.trending_sort == TrendingSort::TickerAsc, "Ticker A-Z").clicked() {
                        self.trending_sort = TrendingSort::TickerAsc;
                    }
                });
            });

        ui.add_space(8.0);

        // Filter and sort items
        let query = self.trending_search.trim().to_uppercase();
        let mut filtered: Vec<StockTrendAnalysis> = self
            .trending_cache
            .iter()
            .filter(|item| {
                if let Some(signal) = self.trending_filter_signal {
                    if item.signal != signal {
                        return false;
                    }
                }
                if query.is_empty() {
                    return true;
                }
                item.ticker.contains(&query) || item.name.to_uppercase().contains(&query) || item.sector.to_uppercase().contains(&query)
            })
            .cloned()
            .collect();

        match self.trending_sort {
            TrendingSort::ScoreDesc => filtered.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal)),
            TrendingSort::GrahamUpsideDesc => filtered.sort_by(|a, b| {
                let u_a = a.graham_upside_pct.unwrap_or(-999.0);
                let u_b = b.graham_upside_pct.unwrap_or(-999.0);
                u_b.partial_cmp(&u_a).unwrap_or(std::cmp::Ordering::Equal)
            }),
            TrendingSort::BazinUpsideDesc => filtered.sort_by(|a, b| {
                let u_a = a.bazin_upside_pct.unwrap_or(-999.0);
                let u_b = b.bazin_upside_pct.unwrap_or(-999.0);
                u_b.partial_cmp(&u_a).unwrap_or(std::cmp::Ordering::Equal)
            }),
            TrendingSort::DividendYieldDesc => filtered.sort_by(|a, b| b.dividend_yield.partial_cmp(&a.dividend_yield).unwrap_or(std::cmp::Ordering::Equal)),
            TrendingSort::RsiAsc => filtered.sort_by(|a, b| {
                let r_a = a.rsi_14.unwrap_or(50.0);
                let r_b = b.rsi_14.unwrap_or(50.0);
                r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
            }),
            TrendingSort::TickerAsc => filtered.sort_by(|a, b| a.ticker.cmp(&b.ticker)),
        }

        if self.trending_selected_ticker.is_none() && !filtered.is_empty() {
            self.trending_selected_ticker = Some(filtered[0].ticker.clone());
        }

        // Split Layout: Table (Left) + Detailed Number Analysis Panel (Right)
        let total_w = ui.available_width();
        let left_w = if total_w > 900.0 { (total_w * 0.58).max(480.0) } else { total_w };

        ui.horizontal_top(|ui| {
            // Left Column: Table of Stocks
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(RichText::new(format!("Exibindo {} ativos ordenados", filtered.len())).color(palette.muted).size(12.0));
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical().id_salt("trending_table_scroll").max_height(600.0).show(ui, |ui| {
                        egui::Grid::new("trending_stocks_grid")
                            .striped(true)
                            .spacing([10.0, 7.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Ticker").strong());
                                ui.label(RichText::new("Preço").strong());
                                ui.label(RichText::new("Var 1d").strong());
                                ui.label(RichText::new("Sinal").strong());
                                ui.label(RichText::new("Score").strong());
                                ui.label(RichText::new("IFR").strong());
                                ui.label(RichText::new("Graham").strong());
                                ui.label(RichText::new("DY").strong());
                                ui.label(RichText::new("Ação").strong());
                                ui.end_row();

                                for item in &filtered {
                                    let is_selected = self.trending_selected_ticker.as_deref() == Some(&item.ticker);

                                    // Ticker
                                    let tick_color = if is_selected { palette.accent } else { palette.text };
                                    if ui.link(RichText::new(&item.ticker).strong().color(tick_color)).clicked() {
                                        self.trending_selected_ticker = Some(item.ticker.clone());
                                    }

                                    // Price
                                    ui.label(RichText::new(format!("R$ {:.2}", item.current_price)));

                                    // Change
                                    if let Some(ch) = item.price_change_pct {
                                        let c = if ch >= 0.0 { palette.positive } else { palette.negative };
                                        ui.label(RichText::new(format!("{ch:+.2}%")).color(c));
                                    } else {
                                        ui.label(RichText::new("-").color(palette.muted));
                                    }

                                    // Signal badge
                                    render_signal_pill(ui, item.signal);

                                    // Score with mini bar
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{:.0}", item.overall_score)).strong());
                                        render_mini_score_bar(ui, item.overall_score, palette);
                                    });

                                    // RSI
                                    if let Some(rsi) = item.rsi_14 {
                                        let c = if rsi < 35.0 { palette.positive } else if rsi > 70.0 { palette.negative } else { palette.text };
                                        ui.label(RichText::new(format!("{rsi:.1}")).color(c));
                                    } else {
                                        ui.label("-");
                                    }

                                    // Graham upside
                                    if let Some(upside) = item.graham_upside_pct {
                                        let c = if upside >= 0.0 { palette.positive } else { palette.muted };
                                        ui.label(RichText::new(format!("{upside:+.1}%")).color(c));
                                    } else {
                                        ui.label("-");
                                    }

                                    // DY
                                    ui.label(RichText::new(format!("{:.1}%", item.dividend_yield)));

                                    // Action
                                    let btn_txt = if is_selected { "● Vendo" } else { "Analisar" };
                                    if ui.small_button(btn_txt).clicked() {
                                        self.trending_selected_ticker = Some(item.ticker.clone());
                                    }

                                    ui.end_row();
                                }
                            });
                    });
                },
            );

            // Right Column: Detailed Numerical Analysis Panel
            if total_w > 900.0 {
                ui.separator();
                let selected_opt = self
                    .trending_selected_ticker
                    .as_deref()
                    .and_then(|tick| self.trending_cache.iter().find(|a| a.ticker == tick))
                    .cloned()
                    .or_else(|| filtered.first().cloned());

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        if let Some(analysis) = selected_opt {
                            self.render_detailed_numbers_panel(ui, &analysis, palette);
                        } else {
                            ui.label(RichText::new("Selecione um ativo para visualizar a análise detalhada dos números.").color(palette.muted));
                        }
                    },
                );
            }
        });
    }

    fn render_detailed_numbers_panel(&mut self, ui: &mut egui::Ui, item: &StockTrendAnalysis, palette: Palette) {
        egui::ScrollArea::vertical().id_salt("trending_detail_scroll").show(ui, |ui| {
            // Header: Ticker, Name, Sector, Current Price & Signal
            ui.horizontal(|ui| {
                ui.heading(RichText::new(&item.ticker).strong().size(26.0).color(palette.accent));
                ui.add_space(6.0);
                render_signal_pill(ui, item.signal);
            });
            ui.label(RichText::new(format!("{} • {}", item.name, item.sector)).color(palette.muted).size(13.0));

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("R$ {:.2}", item.current_price)).strong().size(22.0).color(palette.text));
                if let Some(ch) = item.price_change_pct {
                    let c = if ch >= 0.0 { palette.positive } else { palette.negative };
                    ui.label(RichText::new(format!("{ch:+.2}% vs candle anterior")).strong().size(13.0).color(c));
                }
            });

            ui.add_space(8.0);

            // Big Composite Score Gauge Card
            egui::Frame::group(ui.style())
                .fill(palette.card)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Score Quantitativo Geral:").strong().size(14.0).color(palette.label));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{:.0} / 100", item.overall_score)).strong().size(20.0).color(palette.accent));
                        });
                    });
                    ui.add_space(4.0);

                    // Large Score Gauge Bar
                    let (r, g, b) = item.signal.color_code();
                    let bar_color = Color32::from_rgb(r, g, b);
                    let progress = (item.overall_score / 100.0).clamp(0.0, 1.0) as f32;
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 12.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 6.0, palette.card_inset);
                    let filled_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * progress, 12.0));
                    ui.painter().rect_filled(filled_rect, 6.0, bar_color);

                    ui.add_space(8.0);

                    // Split Scores: Technical vs Fundamental
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Score Técnico:").size(12.0).color(palette.muted));
                        ui.label(RichText::new(format!("{:.0}/100", item.technical_score)).strong().size(13.0).color(palette.text));
                        ui.separator();
                        ui.label(RichText::new("Score Fundamentalista:").size(12.0).color(palette.muted));
                        ui.label(RichText::new(format!("{:.0}/100", item.fundamental_score)).strong().size(13.0).color(palette.text));
                    });
                });

            ui.add_space(8.0);

            // Rationale Callout
            egui::Frame::group(ui.style())
                .fill(palette.card_alt)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Diagnóstico do Algoritmo").strong().size(13.0).color(palette.text));
                    ui.add_space(2.0);
                    ui.label(RichText::new(&item.summary_rationale).size(12.0).color(palette.subtle));
                });

            ui.add_space(10.0);

            // Detailed Technical Numbers Card
            egui::Frame::group(ui.style())
                .fill(palette.card)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("1. Análise Técnica dos Números").strong().size(14.0).color(palette.text));
                    ui.add_space(6.0);

                    egui::Grid::new("tech_numbers_grid")
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("IFR (RSI 14):").color(palette.label));
                            ui.horizontal(|ui| {
                                if let Some(rsi) = item.rsi_14 {
                                    let c = if rsi < 35.0 { palette.positive } else if rsi > 70.0 { palette.negative } else { palette.text };
                                    ui.label(RichText::new(format!("{rsi:.1}")).strong().color(c));
                                    ui.label(RichText::new(&item.rsi_status).size(11.0).color(palette.muted));
                                } else {
                                    ui.label("Indisponível");
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("Média Móvel 20 (SMA20):").color(palette.label));
                            ui.horizontal(|ui| {
                                if let Some(s20) = item.sma_20 {
                                    let diff = item.price_vs_sma20_pct.unwrap_or(0.0);
                                    let c = if diff >= 0.0 { palette.positive } else { palette.negative };
                                    ui.label(RichText::new(format!("R$ {s20:.2}")).strong());
                                    ui.label(RichText::new(format!("({diff:+.1}% vs preço)")).size(11.0).color(c));
                                } else {
                                    ui.label("Indisponível");
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("Média Móvel 50 (SMA50):").color(palette.label));
                            ui.horizontal(|ui| {
                                if let Some(s50) = item.sma_50 {
                                    let diff = item.price_vs_sma50_pct.unwrap_or(0.0);
                                    let c = if diff >= 0.0 { palette.positive } else { palette.negative };
                                    ui.label(RichText::new(format!("R$ {s50:.2}")).strong());
                                    ui.label(RichText::new(format!("({diff:+.1}% vs preço)")).size(11.0).color(c));
                                } else {
                                    ui.label("Indisponível");
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("Tendência de Médias:").color(palette.label));
                            ui.label(RichText::new(&item.moving_average_trend).strong().color(palette.accent));
                            ui.end_row();

                            ui.label(RichText::new("Bandas de Bollinger:").color(palette.label));
                            ui.horizontal(|ui| {
                                if let (Some(u), Some(m), Some(l)) = (item.bollinger_upper, item.bollinger_middle, item.bollinger_lower) {
                                    ui.label(RichText::new(format!("Sup: R$ {u:.2} | Média: R$ {m:.2} | Inf: R$ {l:.2}")).size(11.0));
                                    if let Some(pct) = item.bollinger_pct_b {
                                        ui.label(RichText::new(format!("(%B: {:.0}%)", pct * 100.0)).size(11.0).color(palette.muted));
                                    }
                                } else {
                                    ui.label("Indisponível");
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("Momento 20 dias:").color(palette.label));
                            if let Some(mom) = item.momentum_20d_pct {
                                let c = if mom >= 0.0 { palette.positive } else { palette.negative };
                                ui.label(RichText::new(format!("{mom:+.2}%")).strong().color(c));
                            } else {
                                ui.label("Indisponível");
                            }
                            ui.end_row();
                        });
                });

            ui.add_space(10.0);

            // Detailed Fundamental Numbers Card
            egui::Frame::group(ui.style())
                .fill(palette.card)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("2. Análise Fundamentalista & Valuation").strong().size(14.0).color(palette.text));
                    ui.add_space(6.0);

                    egui::Grid::new("fund_numbers_grid")
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Benjamin Graham:").color(palette.label));
                            ui.horizontal(|ui| {
                                if let Some(g) = item.graham_fair_value {
                                    let upside = item.graham_upside_pct.unwrap_or(0.0);
                                    let c = if upside >= 0.0 { palette.positive } else { palette.negative };
                                    ui.label(RichText::new(format!("Preço Justo R$ {g:.2}")).strong());
                                    ui.label(RichText::new(format!("(Margem: {upside:+.1}%)")).size(11.0).color(c));
                                } else {
                                    ui.label("Indisponível");
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("Décio Bazin:").color(palette.label));
                            ui.horizontal(|ui| {
                                if let Some(b) = item.bazin_ceiling_price {
                                    let upside = item.bazin_upside_pct.unwrap_or(0.0);
                                    let c = if upside >= 0.0 { palette.positive } else { palette.negative };
                                    ui.label(RichText::new(format!("Preço Teto R$ {b:.2}")).strong());
                                    ui.label(RichText::new(format!("(Margem: {upside:+.1}%)")).size(11.0).color(c));
                                } else {
                                    ui.label("Indisponível");
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("P/L e P/VP:").color(palette.label));
                            ui.label(RichText::new(format!("P/L: {:.2}x  |  P/VP: {:.2}x", item.pl, item.pvp)).strong());
                            ui.end_row();

                            ui.label(RichText::new("Dividend Yield (LTM):").color(palette.label));
                            ui.label(RichText::new(format!("{:.2}%", item.dividend_yield)).strong().color(palette.positive));
                            ui.end_row();

                            ui.label(RichText::new("Rentabilidade (ROE / Margem):").color(palette.label));
                            ui.label(RichText::new(format!("ROE: {:.1}%  |  Margem Líq.: {:.1}%", item.roe, item.margem_liquida)).strong());
                            ui.end_row();

                            ui.label(RichText::new("Saúde Contábil & Solvência:").color(palette.label));
                            ui.label(RichText::new(format!("Piotroski: {}/9  |  Altman Z: {:.2}", item.piotroski_f_score, item.altman_z_score)).size(12.0));
                            ui.end_row();
                        });
                });

            ui.add_space(10.0);

            // Catalysts & Risk Factors
            egui::Frame::group(ui.style())
                .fill(palette.card)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("3. Fatores Críticos do Algoritmo").strong().size(14.0).color(palette.text));
                    ui.add_space(6.0);

                    ui.label(RichText::new("Catalisadores de Compra (Prós):").strong().size(12.0).color(palette.positive));
                    for cat in &item.buy_catalysts {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("✔").color(palette.positive));
                            ui.label(RichText::new(cat).size(12.0));
                        });
                    }

                    ui.add_space(6.0);

                    ui.label(RichText::new("Fatores de Risco e Cautela (Contras):").strong().size(12.0).color(palette.negative));
                    for risk in &item.risk_factors {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠").color(palette.negative));
                            ui.label(RichText::new(risk).size(12.0));
                        });
                    }
                });

            ui.add_space(12.0);

            // Quick Actions
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Abrir no Terminal").strong()).clicked() {
                    self.apply_selected_ticker(&item.ticker);
                    self.active_view = ActiveView::GpuTerminal;
                }
                if ui.button("Adicionar à Tela").clicked() {
                    self.apply_selected_ticker(&item.ticker);
                }
                if ui.button("Ver na Ficha").clicked() {
                    self.apply_selected_ticker(&item.ticker);
                    self.active_view = ActiveView::StatusInvest;
                }
                if ui.button("Ver Decisão").clicked() {
                    self.apply_selected_ticker(&item.ticker);
                    self.active_view = ActiveView::Decision;
                }
            });

            ui.add_space(16.0);
        });
    }
}

fn render_summary_card(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    subtext: &str,
    accent: Color32,
    bg: Color32,
    width: f32,
) {
    egui::Frame::group(ui.style())
        .fill(bg)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_min_width(width.max(120.0));
            ui.label(RichText::new(label).size(11.0).color(Color32::from_rgb(150, 160, 175)));
            ui.label(RichText::new(value).strong().size(18.0).color(accent));
            ui.label(RichText::new(subtext).size(10.0).color(Color32::from_rgb(130, 140, 150)));
        });
}

fn render_signal_pill(ui: &mut egui::Ui, signal: TrendSignal) {
    let (r, g, b) = signal.color_code();
    let bg = Color32::from_rgb((r as f32 * 0.2) as u8, (g as f32 * 0.2) as u8, (b as f32 * 0.2) as u8);
    let fg = Color32::from_rgb(r, g, b);

    egui::Frame::new()
        .fill(bg)
        .corner_radius(4)
        .stroke(Stroke::new(1.0, fg))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(signal.label()).size(11.0).strong().color(fg));
        });
}

fn render_mini_score_bar(ui: &mut egui::Ui, score: f64, palette: Palette) {
    let (r, g, b) = if score >= 70.0 {
        (46, 180, 80)
    } else if score >= 50.0 {
        (100, 180, 120)
    } else if score >= 35.0 {
        (220, 150, 50)
    } else {
        (240, 70, 70)
    };
    let bar_color = Color32::from_rgb(r, g, b);
    let progress = (score / 100.0).clamp(0.0, 1.0) as f32;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 6.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 3.0, palette.card_inset);
    let filled_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * progress, 6.0));
    ui.painter().rect_filled(filled_rect, 3.0, bar_color);
}
