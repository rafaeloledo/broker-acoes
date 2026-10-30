use eframe::egui::{self, Color32, RichText};
use b3_core::indicators::StockIndicators;
use b3_core::ticker::{AssetClass, CompanyProfile};

use crate::{AppState, MAX_VISIBLE_ASSETS};
use crate::presentation::{fundamentals_caption, price_caption, MetricId};

impl AppState {
    pub(crate) fn render_status_invest_view(&mut self, ui: &mut egui::Ui) {
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
                                let price = self.shown_price(sym, ind.price);
                                ui.heading(RichText::new(sym).strong().color(Color32::from_rgb(0, 210, 160)).size(18.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("R$ {price:.2}")).strong().color(Color32::WHITE).size(18.0));
                                });
                            });
                            ui.label(RichText::new(&profile.name).color(Color32::from_rgb(180, 195, 210)).size(12.0));
                            ui.label(RichText::new(price_caption(self.market_as_of.get(sym).copied())).color(Color32::GRAY).size(11.0));
                            ui.label(RichText::new(fundamentals_caption(self.cvm_year.get(sym).copied())).color(Color32::GRAY).size(11.0));
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
                                        let price = self.shown_price(sym, ind.price);
                                        let graham_txt = self.metric(sym, MetricId::Graham, &format!("R$ {:.2}", ind.graham_fair_value));
                                        ui.label(RichText::new(graham_txt).strong().color(Color32::from_rgb(0, 210, 160)));
                                        let fair = if self.cvm_year.contains_key(sym) {
                                            self.core_multiples.get(sym).and_then(|m| m.graham_fair_value)
                                        } else {
                                            Some(ind.graham_fair_value)
                                        };
                                        if let Some(fair) = fair.filter(|_| price > 0.0) {
                                            let upside = ((fair - price) / price) * 100.0;
                                            let up_col = if upside >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                                            ui.label(RichText::new(format!("{upside:+.1}%")).color(up_col).size(11.0));
                                        }
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
                                        let price = self.shown_price(sym, ind.price);
                                        ui.label(RichText::new(self.metric(sym, MetricId::Bazin, &format!("R$ {:.2}", ind.bazin_ceiling_price))).strong().color(Color32::from_rgb(100, 180, 255)));
                                        let fair = if self.cvm_year.contains_key(sym) {
                                            self.core_multiples.get(sym).and_then(|m| m.bazin_ceiling_price)
                                        } else {
                                            Some(ind.bazin_ceiling_price)
                                        };
                                        if let Some(fair) = fair.filter(|fair| *fair > 0.0) {
                                            let margin = ((fair - price) / fair) * 100.0;
                                            ui.label(RichText::new(format!("Margem: {margin:.1}%")).size(11.0));
                                        }
                                    });
                                });

                            ui.add_space(4.0);

                            // Piotroski & Altman
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(26, 30, 42))
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if self.cvm_year.contains_key(sym) {
                                            ui.label(RichText::new("Piotroski: indisponível").strong().size(11.0));
                                        } else {
                                            ui.label(RichText::new(format!("Piotroski: {}/9", ind.piotroski_f_score)).strong().color(Color32::from_rgb(0, 220, 140)).size(11.0));
                                            if ind.altman_z_score > 0.0 {
                                                ui.label(RichText::new(format!("Z: {:.2}", ind.altman_z_score)).size(11.0));
                                            }
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
                                    ui.label("P/L:"); ui.label(RichText::new(self.metric(sym, MetricId::Pl, &format!("{:.2}", ind.pl))).strong()); ui.end_row();
                                    ui.label("P/VP:"); ui.label(RichText::new(self.metric(sym, MetricId::Pvp, &format!("{:.2}", ind.pvp))).strong()); ui.end_row();
                                    ui.label("EV/EBITDA:"); ui.label(self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.ev_ebitda))); ui.end_row();
                                    ui.label("Div. Yield:"); ui.label(RichText::new(self.metric(sym, MetricId::DividendYield, &format!("{:.2}%", ind.dividend_yield))).color(Color32::from_rgb(0, 220, 140))); ui.end_row();
                                    ui.label("ROE:"); ui.label(self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roe))); ui.end_row();
                                    ui.label("ROIC:"); ui.label(self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roic))); ui.end_row();
                                    ui.label("Mg. Líquida:"); ui.label(self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_liquida))); ui.end_row();
                                    ui.label("Dív.Líq/EBITDA:"); ui.label(self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.divida_liq_ebitda))); ui.end_row();
                                    ui.label("Liq. Corrente:"); ui.label(self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.liquidez_corrente))); ui.end_row();
                                    ui.label("LPA:"); ui.label(self.metric(sym, MetricId::Lpa, &format!("R$ {:.2}", ind.lpa))); ui.end_row();
                                    ui.label("VPA:"); ui.label(self.metric(sym, MetricId::Vpa, &format!("R$ {:.2}", ind.vpa))); ui.end_row();
                                    ui.label("DPA:"); ui.label(self.metric(sym, MetricId::Dpa, &format!("R$ {:.2}", ind.dpa))); ui.end_row();
                                });
                        });
                }
            });

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

            ui.add_space(12.0);

            // Detailed Side-by-Side Comparison Matrix Table
            self.render_comparison_table(ui);
        });
    }

    pub(crate) fn render_single_status_invest(&mut self, ui: &mut egui::Ui, sym: &str) {
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
                                let price = self.shown_price(sym, ind.price);
                                ui.label(RichText::new(format!("R$ {price:.2}")).strong().size(28.0).color(Color32::from_rgb(0, 220, 140)));
                                ui.label(RichText::new(price_caption(self.market_as_of.get(sym).copied())).color(Color32::GRAY).size(11.0));
                                ui.label(RichText::new(fundamentals_caption(self.cvm_year.get(sym).copied())).color(Color32::GRAY).size(11.0));
                            });
                        });
                    });
                });

            ui.add_space(8.0);

            // Key KPI Highlight Cards (Status Invest Style)
            ui.horizontal(|ui| {
                let card_w = (ui.available_width() - 40.0) / 6.0;
                let ev_ebitda = if ind.ev_ebitda > 0.0 { format!("{:.2}", ind.ev_ebitda) } else { "-".to_string() };
                let roic = if ind.roic > 0.0 { format!("{:.2}%", ind.roic) } else { "-".to_string() };
                let kpis = [
                    ("DIVIDEND YIELD", self.metric(sym, MetricId::DividendYield, &format!("{:.2}%", ind.dividend_yield)), Color32::from_rgb(0, 210, 160)),
                    ("P/L", self.metric(sym, MetricId::Pl, &format!("{:.2}", ind.pl)), Color32::from_rgb(255, 200, 80)),
                    ("P/VP", self.metric(sym, MetricId::Pvp, &format!("{:.2}", ind.pvp)), Color32::from_rgb(100, 180, 255)),
                    ("EV/EBITDA", self.metric(sym, MetricId::Outside, &ev_ebitda), Color32::WHITE),
                    ("ROE", self.metric(sym, MetricId::Outside, &format!("{:.2}%", ind.roe)), Color32::from_rgb(0, 210, 160)),
                    ("ROIC", self.metric(sym, MetricId::Outside, &roic), Color32::from_rgb(140, 220, 255)),
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
                        let price = self.shown_price(sym, ind.price);
                        let graham_txt = self.metric(sym, MetricId::Graham, &format!("R$ {:.2}", ind.graham_fair_value));
                        ui.label(RichText::new(graham_txt).strong().color(Color32::from_rgb(0, 210, 160)).size(20.0));
                        let fair = if self.cvm_year.contains_key(sym) {
                            self.core_multiples.get(sym).and_then(|m| m.graham_fair_value)
                        } else {
                            Some(ind.graham_fair_value)
                        };
                        if let Some(fair) = fair.filter(|_| price > 0.0) {
                            let upside = ((fair - price) / price) * 100.0;
                            let color = if upside >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                            ui.label(RichText::new(format!("Margem / Upside: {upside:+.1}%")).color(color).size(12.0));
                        }
                    });

                // Décio Bazin Card
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 28, 38))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(col_w);
                        ui.label(RichText::new("Preço Teto Bazin").strong().color(Color32::WHITE).size(14.0));
                        if self.cvm_year.contains_key(sym) {
                            ui.label(RichText::new("DPA / 6%").color(Color32::GRAY).size(11.0));
                            let price = self.shown_price(sym, ind.price);
                            ui.label(RichText::new(self.metric(sym, MetricId::Bazin, &format!("R$ {:.2}", ind.bazin_ceiling_price))).strong().color(Color32::from_rgb(100, 180, 255)).size(20.0));
                            if let Some(fair) = self.core_multiples.get(sym).and_then(|m| m.bazin_ceiling_price).filter(|fair| *fair > 0.0) {
                                let margin = ((fair - price) / fair) * 100.0;
                                ui.label(RichText::new(format!("Margem de Segurança: {margin:.1}%")).color(Color32::WHITE).size(12.0));
                            }
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("Yield Alvo:");
                                ui.add(egui::Slider::new(&mut self.bazin_target_yield, 4.0..=12.0).suffix("%"));
                            });
                            let bazin_price = if self.bazin_target_yield > 0.0 { ind.dpa / (self.bazin_target_yield / 100.0) } else { 0.0 };
                            ui.label(RichText::new(format!("R$ {bazin_price:.2}")).strong().color(Color32::from_rgb(100, 180, 255)).size(20.0));
                            let margin = if bazin_price > 0.0 { ((bazin_price - ind.price) / bazin_price) * 100.0 } else { 0.0 };
                            ui.label(RichText::new(format!("Margem de Segurança: {margin:.1}%")).color(Color32::WHITE).size(12.0));
                        }
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
                    ui.label(RichText::new(fundamentals_caption(self.cvm_year.get(sym).copied())).color(Color32::GRAY).size(12.0));
                    ui.add_space(6.0);

                    ui.columns(4, |cols| {
                        cols[0].label(RichText::new("VALUATION").strong().color(Color32::from_rgb(100, 180, 255)));
                        cols[0].label(format!("P/L: {}", self.metric(sym, MetricId::Pl, &format!("{:.2}", ind.pl))));
                        cols[0].label(format!("P/VP: {}", self.metric(sym, MetricId::Pvp, &format!("{:.2}", ind.pvp))));
                        cols[0].label(format!("EV/EBITDA: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.ev_ebitda))));
                        cols[0].label(format!("EV/EBIT: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.ev_ebit))));
                        cols[0].label(format!("P/EBITDA: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_ebitda))));
                        cols[0].label(format!("P/SR: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_sr))));
                        cols[0].label(format!("P/Ativo: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_ativo))));
                        cols[0].label(format!("P/Cap. Giro: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_cap_giro))));

                        cols[1].label(RichText::new("ENDIVIDAMENTO").strong().color(Color32::from_rgb(255, 180, 100)));
                        cols[1].label(format!("Dív. Líq./PL: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.divida_liq_pl))));
                        cols[1].label(format!("Dív. Líq./EBITDA: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.divida_liq_ebitda))));
                        cols[1].label(format!("Dív. Líq./EBIT: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.divida_liq_ebit))));
                        cols[1].label(format!("PL / Ativos: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.pl_ativos))));
                        cols[1].label(format!("Passivos / Ativos: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.passivos_ativos))));
                        cols[1].label(format!("Liq. Corrente: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.liquidez_corrente))));
                        cols[1].label(format!("Liq. Seca: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.liquidez_seca))));

                        cols[2].label(RichText::new("EFICIÊNCIA").strong().color(Color32::from_rgb(0, 220, 140)));
                        cols[2].label(format!("Margem Bruta: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_bruta))));
                        cols[2].label(format!("Margem EBITDA: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_ebitda))));
                        cols[2].label(format!("Margem EBIT: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_ebit))));
                        cols[2].label(format!("Margem Líquida: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_liquida))));
                        cols[2].label(format!("Giro Ativos: {}", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.giro_ativos))));

                        cols[3].label(RichText::new("RENTABILIDADE & 5A").strong().color(Color32::from_rgb(180, 140, 255)));
                        cols[3].label(format!("ROE: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roe))));
                        cols[3].label(format!("ROIC: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roic))));
                        cols[3].label(format!("ROA: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roa))));
                        cols[3].label(format!("CAGR Receitas: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.cagr_receitas_5a))));
                        cols[3].label(format!("CAGR Lucros: {}", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.cagr_lucros_5a))));
                        cols[3].label(format!("LPA: {}", self.metric(sym, MetricId::Lpa, &format!("R$ {:.2}", ind.lpa))));
                        cols[3].label(format!("VPA: {}", self.metric(sym, MetricId::Vpa, &format!("R$ {:.2}", ind.vpa))));
                        cols[3].label(format!("DPA: {}", self.metric(sym, MetricId::Dpa, &format!("R$ {:.2}", ind.dpa))));
                    });
                });
        });
    }

    pub(crate) fn render_comparison_table(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(22, 26, 36))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading(RichText::new("Matriz Comparativa de Indicadores Fundamentalistas").color(Color32::WHITE).size(16.0));
                ui.label(RichText::new("P/L, P/VP, dividend yield, Graham e Bazin usam a CVM quando o período existe. O restante é cenário local ou fica indisponível.").color(Color32::GRAY).size(12.0));
                ui.add_space(6.0);

                let indicators: Vec<(String, StockIndicators)> = self.visible_tickers
                    .iter()
                    .map(|t| {
                        let ind = self.stock_indicators.get(t).cloned().unwrap_or_else(|| StockIndicators::default_for(t));
                        (t.clone(), ind)
                    })
                    .collect();

                let rows: &[(&str, MetricId, fn(&StockIndicators) -> String)] = &[
                    ("Preço Atual", MetricId::Outside, |i| format!("R$ {:.2}", i.price)),
                    ("P/L", MetricId::Pl, |i| format!("{:.2}", i.pl)),
                    ("P/VP", MetricId::Pvp, |i| format!("{:.2}", i.pvp)),
                    ("EV/EBITDA", MetricId::Outside, |i| format!("{:.2}", i.ev_ebitda)),
                    ("EV/EBIT", MetricId::Outside, |i| format!("{:.2}", i.ev_ebit)),
                    ("P/SR", MetricId::Outside, |i| format!("{:.2}", i.p_sr)),
                    ("P/Ativo", MetricId::Outside, |i| format!("{:.2}", i.p_ativo)),
                    ("Dividend Yield", MetricId::DividendYield, |i| format!("{:.2}%", i.dividend_yield)),
                    ("ROE", MetricId::Outside, |i| format!("{:.2}%", i.roe)),
                    ("ROIC", MetricId::Outside, |i| format!("{:.2}%", i.roic)),
                    ("ROA", MetricId::Outside, |i| format!("{:.2}%", i.roa)),
                    ("Margem Bruta", MetricId::Outside, |i| format!("{:.1}%", i.margem_bruta)),
                    ("Margem EBITDA", MetricId::Outside, |i| format!("{:.1}%", i.margem_ebitda)),
                    ("Margem EBIT", MetricId::Outside, |i| format!("{:.1}%", i.margem_ebit)),
                    ("Margem Líquida", MetricId::Outside, |i| format!("{:.1}%", i.margem_liquida)),
                    ("Dívida Líq./PL", MetricId::Outside, |i| format!("{:.2}", i.divida_liq_pl)),
                    ("Dívida Líq./EBITDA", MetricId::Outside, |i| format!("{:.2}", i.divida_liq_ebitda)),
                    ("Liquidez Corrente", MetricId::Outside, |i| format!("{:.2}", i.liquidez_corrente)),
                    ("Liquidez Seca", MetricId::Outside, |i| format!("{:.2}", i.liquidez_seca)),
                    ("LPA", MetricId::Lpa, |i| format!("R$ {:.2}", i.lpa)),
                    ("VPA", MetricId::Vpa, |i| format!("R$ {:.2}", i.vpa)),
                    ("DPA", MetricId::Dpa, |i| format!("R$ {:.2}", i.dpa)),
                    ("Preço Justo Graham", MetricId::Graham, |i| format!("R$ {:.2}", i.graham_fair_value)),
                    ("Preço Teto Bazin", MetricId::Bazin, |i| format!("R$ {:.2}", i.bazin_ceiling_price)),
                    ("Score Piotroski", MetricId::Outside, |i| format!("{}/9", i.piotroski_f_score)),
                    ("Altman Z-Score", MetricId::Outside, |i| if i.altman_z_score > 0.0 { format!("{:.2}", i.altman_z_score) } else { "-".to_string() }),
                ];

                egui::Grid::new("comparison_matrix_grid")
                    .striped(true)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Indicador").strong());
                        for (t, _) in &indicators {
                            ui.label(RichText::new(t).strong().color(Color32::from_rgb(0, 210, 160)));
                        }
                        ui.end_row();

                        for (name, metric, local) in rows {
                            ui.label(*name);
                            for (sym, ind) in &indicators {
                                let text = if *name == "Preço Atual" {
                                    format!("R$ {:.2}", self.shown_price(sym, ind.price))
                                } else {
                                    self.metric(sym, *metric, &local(ind))
                                };
                                ui.label(text);
                            }
                            ui.end_row();
                        }
                    });
            });
    }
}
