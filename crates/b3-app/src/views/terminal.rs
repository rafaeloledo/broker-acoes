use eframe::egui::{self, RichText, Sense, Vec2};

use crate::theme::Palette;
use b3_core::quote::GpuCandle;
use b3_gpu::candlestick::ChartUniforms;
use b3_gpu::egui_callback::GpuChartCallback;

use crate::{AppState, MAX_VISIBLE_ASSETS};

impl AppState {
    pub(crate) fn render_gpu_terminal_view(&mut self, ui: &mut egui::Ui) {
        let palette = Palette::from_ui(ui);
        let count = self.visible_tickers.len();

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Terminal · {count} gráficos")).strong().size(15.0));
            ui.label(
                RichText::new(format!("{:.0} FPS ({:.1} ms)", self.telemetry.fps, self.telemetry.frame_time_ms))
                    .monospace()
                    .color(palette.positive)
                    .size(12.0),
            );
            ui.separator();
            ui.label("Largura do Candle:");
            ui.add(egui::Slider::new(&mut self.chart_candle_width, 3.0..=20.0));
            ui.separator();
            ui.checkbox(&mut self.is_live_sim_running, "Simulação de ticks (book e tape)");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if count < MAX_VISIBLE_ASSETS {
                    if ui.button(RichText::new("+ Adicionar Gráfico").strong().color(palette.accent)).clicked() {
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
                                    .fill(palette.card_inset)
                                    .inner_margin(6.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("{}: {}", idx + 1, sym)).strong().color(palette.accent).size(15.0));
                                            ui.label(RichText::new(format!("R$ {:.2}", price)).strong().color(palette.text).size(15.0));

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
                self.persist_visible_tickers();
            }

            if let Some(s) = slot_to_change {
                self.search_target_slot = Some(s);
                self.b3_search_query.clear();
                self.is_search_open = true;
                self.just_opened_search = true;
            }
        }
    }

    pub(crate) fn render_single_chart_terminal(&mut self, ui: &mut egui::Ui, sym: &str) {
        let palette = Palette::from_ui(ui);
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
                ui.heading(RichText::new("Livro de Ofertas (simulação)").size(15.0));

                egui::Frame::group(ui.style())
                    .fill(palette.card_inset)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.columns(2, |cols| {
                            cols[0].label(RichText::new("COMPRA (Bids)").color(palette.positive).size(11.0));
                            for b in &self.order_book.bids {
                                cols[0].label(RichText::new(format!("{:.2} | {:.0}", b.price, b.quantity)).size(11.0));
                            }

                            cols[1].label(RichText::new("VENDA (Asks)").color(palette.negative).size(11.0));
                            for a in &self.order_book.asks {
                                cols[1].label(RichText::new(format!("{:.2} | {:.0}", a.price, a.quantity)).size(11.0));
                            }
                        });
                    });

                ui.add_space(8.0);
                ui.heading(RichText::new("Times & Trades (simulação)").size(15.0));

                egui::Frame::group(ui.style())
                    .fill(palette.card_inset)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                            for tick in &self.recent_ticks {
                                let (col, aggressor) = if tick.is_buyer_aggressor {
                                    (palette.positive, "C")
                                } else {
                                    (palette.negative, "V")
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
}
