use eframe::egui::{self, RichText};

use crate::theme::Palette;
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
        let palette = Palette::from_ui(ui);

        // Multi-Asset Comparison View (up to MAX_VISIBLE_ASSETS side-by-side)
        ui.horizontal(|ui| {
            ui.heading(RichText::new(format!("Painel Comparativo de Ativos B3 ({} na tela)", count)).size(18.0).color(palette.text));
            ui.separator();
            ui.label(RichText::new("Análise fundamentalista de múltiplos ativos simultaneamente").color(palette.muted));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if count < MAX_VISIBLE_ASSETS {
                    if ui.button(RichText::new("+ Adicionar para Comparar").strong().color(palette.accent)).clicked() {
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
                        .fill(palette.card)
                        .inner_margin(8.0)
                        .show(col, |ui| {
                            // Column Header
                            ui.horizontal(|ui| {
                                let price = self.shown_price(sym, ind.price);
                                ui.heading(RichText::new(sym).strong().color(palette.accent).size(18.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("R$ {price:.2}")).strong().color(palette.text).size(18.0));
                                });
                            });
                            ui.label(RichText::new(&profile.name).color(palette.subtle).size(12.0));
                            ui.label(RichText::new(price_caption(self.market_as_of.get(sym).copied())).color(palette.muted).size(11.0));
                            ui.label(RichText::new(fundamentals_caption(self.cvm_year.get(sym).copied())).color(palette.muted).size(11.0));
                            ui.label(RichText::new(&profile.subsector).color(palette.muted).size(11.0));

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
                                .fill(palette.card_alt)
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Preço Justo Graham").strong().size(11.0));
                                    ui.horizontal(|ui| {
                                        let price = self.shown_price(sym, ind.price);
                                        let graham_txt = self.metric(sym, MetricId::Graham, &format!("R$ {:.2}", ind.graham_fair_value));
                                        ui.label(RichText::new(graham_txt).strong().color(palette.accent));
                                        let fair = if self.cvm_year.contains_key(sym) {
                                            self.core_multiples.get(sym).and_then(|m| m.graham_fair_value)
                                        } else {
                                            Some(ind.graham_fair_value)
                                        };
                                        if let Some(fair) = fair.filter(|_| price > 0.0) {
                                            let upside = ((fair - price) / price) * 100.0;
                                            let up_col = if upside >= 0.0 { palette.positive } else { palette.negative };
                                            ui.label(RichText::new(format!("{upside:+.1}%")).color(up_col).size(11.0));
                                        }
                                    });
                                });

                            ui.add_space(4.0);

                            // Bazin Card
                            egui::Frame::group(ui.style())
                                .fill(palette.card_alt)
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Preço Teto Bazin (6%)").strong().size(11.0));
                                    ui.horizontal(|ui| {
                                        let price = self.shown_price(sym, ind.price);
                                        ui.label(RichText::new(self.metric(sym, MetricId::Bazin, &format!("R$ {:.2}", ind.bazin_ceiling_price))).strong().color(palette.link));
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
                                .fill(palette.card_alt)
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if self.cvm_year.contains_key(sym) {
                                            ui.label(RichText::new("Piotroski: indisponível").strong().size(11.0));
                                        } else {
                                            ui.label(RichText::new(format!("Piotroski: {}/9", ind.piotroski_f_score)).strong().color(palette.positive).size(11.0));
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
                                        .fill(palette.card)
                                        .inner_margin(6.0)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new(format!("FII: {}", fii.segment.display_name())).strong().color(palette.accent).size(11.0));
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
                                    ui.label("Div. Yield:"); ui.label(RichText::new(self.metric(sym, MetricId::DividendYield, &format!("{:.2}%", ind.dividend_yield))).color(palette.positive)); ui.end_row();
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
        let palette = Palette::from_ui(ui);
        let profile = self.company_profiles.get(sym).cloned().unwrap_or_else(|| CompanyProfile::default_for(sym));
        let ind = self.stock_indicators.get(sym).cloned().unwrap_or_else(|| StockIndicators::default_for(sym));
        let is_fii = AssetClass::from_symbol(sym) == AssetClass::Fii;

        egui::ScrollArea::vertical().show(ui, |ui| {
            let price = self.shown_price(sym, ind.price);
            section_card(ui, palette.card, |ui| {
                ui.horizontal_top(|ui| {
                    let price_width = 168.0;
                    let text_width = (ui.available_width() - price_width - 12.0).max(160.0);
                    ui.vertical(|ui| {
                        ui.set_width(text_width);
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.horizontal(|ui| {
                            ui.heading(RichText::new(&profile.symbol).strong().size(26.0).color(palette.text));
                            ui.label(RichText::new(&profile.segment).background_color(palette.badge_bg).size(11.0));
                        });
                        ui.label(RichText::new(&profile.name).color(palette.subtle).size(14.0));
                        ui.label(
                            RichText::new(format!(
                                "Setor: {}  |  Sub-setor: {}",
                                profile.sector.display_name(),
                                profile.subsector
                            ))
                            .color(palette.muted)
                            .size(12.0),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.set_width(price_width);
                        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                            ui.label(RichText::new(format!("R$ {price:.2}")).strong().size(28.0).color(palette.positive));
                            ui.label(RichText::new(price_caption(self.market_as_of.get(sym).copied())).color(palette.muted).size(11.0));
                            ui.label(RichText::new(fundamentals_caption(self.cvm_year.get(sym).copied())).color(palette.muted).size(11.0));
                        });
                    });
                });
            });

            ui.add_space(10.0);

            let ev_ebitda = if ind.ev_ebitda > 0.0 { format!("{:.2}", ind.ev_ebitda) } else { "-".to_string() };
            let roic = if ind.roic > 0.0 { format!("{:.2}%", ind.roic) } else { "-".to_string() };
            let kpis = [
                ("Dividend yield", self.metric(sym, MetricId::DividendYield, &format!("{:.2}%", ind.dividend_yield)), palette.accent),
                ("P/L", self.metric(sym, MetricId::Pl, &format!("{:.2}", ind.pl)), palette.warning),
                ("P/VP", self.metric(sym, MetricId::Pvp, &format!("{:.2}", ind.pvp)), palette.link),
                ("EV/EBITDA", self.metric(sym, MetricId::Outside, &ev_ebitda), palette.text),
                ("ROE", self.metric(sym, MetricId::Outside, &format!("{:.2}%", ind.roe)), palette.accent),
                ("ROIC", self.metric(sym, MetricId::Outside, &roic), palette.link),
            ];
            ui.columns(kpis.len(), |cols| {
                for (col, (label, value, color)) in cols.iter_mut().zip(kpis) {
                    section_card(col, palette.card_alt, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(label).color(palette.label).size(12.0));
                            ui.add_space(2.0);
                            ui.label(RichText::new(value).strong().color(color).size(18.0));
                        });
                    });
                }
            });

            ui.add_space(10.0);

            let graham_txt = self.metric(sym, MetricId::Graham, &format!("R$ {:.2}", ind.graham_fair_value));
            let graham_fair = if self.cvm_year.contains_key(sym) {
                self.core_multiples.get(sym).and_then(|m| m.graham_fair_value)
            } else {
                Some(ind.graham_fair_value)
            };
            let bazin_txt = self.metric(sym, MetricId::Bazin, &format!("R$ {:.2}", ind.bazin_ceiling_price));
            let bazin_fair = self.core_multiples.get(sym).and_then(|m| m.bazin_ceiling_price);
            let has_cvm = self.cvm_year.contains_key(sym);
            ui.columns(3, |cols| {
                section_card(&mut cols[0], palette.card, |ui| {
                    ui.label(RichText::new("Preço justo de Graham").strong().color(palette.text).size(14.0));
                    ui.label(RichText::new("sqrt(22,5 × LPA × VPA)").color(palette.muted).size(12.0));
                    ui.add_space(6.0);
                    ui.label(RichText::new(graham_txt).strong().color(palette.accent).size(22.0));
                    if let Some(fair) = graham_fair.filter(|_| price > 0.0) {
                        let upside = ((fair - price) / price) * 100.0;
                        let color = if upside >= 0.0 { palette.positive } else { palette.negative };
                        ui.label(RichText::new(format!("Upside {upside:+.1}%")).color(color).size(13.0));
                    }
                });

                section_card(&mut cols[1], palette.card, |ui| {
                    ui.label(RichText::new("Preço teto Bazin").strong().color(palette.text).size(14.0));
                    if has_cvm {
                        ui.label(RichText::new("DPA / 6%").color(palette.muted).size(12.0));
                        ui.add_space(6.0);
                        ui.label(RichText::new(bazin_txt).strong().color(palette.link).size(22.0));
                        if let Some(fair) = bazin_fair.filter(|fair| *fair > 0.0) {
                            let margin = ((fair - price) / fair) * 100.0;
                            ui.label(RichText::new(format!("Margem de segurança {margin:.1}%")).color(palette.text).size(13.0));
                        }
                    } else {
                        ui.label(RichText::new("Yield alvo").color(palette.muted).size(12.0));
                        ui.add(egui::Slider::new(&mut self.bazin_target_yield, 4.0..=12.0).suffix("%"));
                        let bazin_price = if self.bazin_target_yield > 0.0 { ind.dpa / (self.bazin_target_yield / 100.0) } else { 0.0 };
                        ui.add_space(6.0);
                        ui.label(RichText::new(format!("R$ {bazin_price:.2}")).strong().color(palette.link).size(22.0));
                        let margin = if bazin_price > 0.0 { ((bazin_price - ind.price) / bazin_price) * 100.0 } else { 0.0 };
                        ui.label(RichText::new(format!("Margem de segurança {margin:.1}%")).color(palette.text).size(13.0));
                    }
                });

                section_card(&mut cols[2], palette.card, |ui| {
                    ui.label(RichText::new("Piotroski e Altman Z").strong().color(palette.text).size(14.0));
                    ui.label(RichText::new("Saúde financeira").color(palette.muted).size(12.0));
                    ui.add_space(6.0);
                    ui.label(RichText::new(format!("F-Score {}/9", ind.piotroski_f_score)).strong().color(palette.positive).size(22.0));
                    let status = if ind.piotroski_f_score >= 7 { "Forte / saudável" } else { "Moderado" };
                    ui.label(RichText::new(status).color(palette.muted).size(13.0));
                    if ind.altman_z_score > 0.0 {
                        let zone = if ind.altman_z_score > 2.99 {
                            "Zona segura (> 2,99)"
                        } else if ind.altman_z_score > 1.81 {
                            "Zona cinzenta"
                        } else {
                            "Alerta"
                        };
                        ui.label(RichText::new(format!("Altman Z {altman:.2}", altman = ind.altman_z_score)).size(13.0));
                        ui.label(RichText::new(zone).color(palette.muted).size(12.0));
                    } else {
                        ui.label(RichText::new("Altman Z indisponível para instituições financeiras").color(palette.muted).size(12.0));
                    }
                });
            });

            ui.add_space(12.0);

            // Special FII Panel if applicable
            if is_fii {
                if let Some(fii) = self.fii_details.get(sym) {
                    egui::Frame::group(ui.style())
                        .fill(palette.card)
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.heading(RichText::new("Raio-X do Fundo Imobiliário (FII)").color(palette.accent).size(16.0));
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(format!("Segmento: {}", fii.segment.display_name()));
                                ui.label(format!("Vacância física: {:.1}%", fii.physical_vacancy_percent));
                                ui.label(format!("Vacância financeira: {:.1}%", fii.financial_vacancy_percent));
                                ui.label(format!("Último rendimento: R$ {:.2}/cota", fii.last_dividend));
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

            section_card(ui, palette.card, |ui| {
                ui.heading(RichText::new("Indicadores fundamentalistas").color(palette.text).size(16.0));
                ui.label(RichText::new(fundamentals_caption(self.cvm_year.get(sym).copied())).color(palette.muted).size(12.0));
                ui.add_space(8.0);
                ui.columns(4, |cols| {
                    metric_group(&mut cols[0], "Valuation", palette.link, palette, &[
                        ("P/L", self.metric(sym, MetricId::Pl, &format!("{:.2}", ind.pl))),
                        ("P/VP", self.metric(sym, MetricId::Pvp, &format!("{:.2}", ind.pvp))),
                        ("EV/EBITDA", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.ev_ebitda))),
                        ("EV/EBIT", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.ev_ebit))),
                        ("P/EBITDA", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_ebitda))),
                        ("P/SR", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_sr))),
                        ("P/Ativo", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_ativo))),
                        ("P/Cap. giro", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.p_cap_giro))),
                    ]);
                    metric_group(&mut cols[1], "Endividamento", palette.warning, palette, &[
                        ("Dív. líq./PL", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.divida_liq_pl))),
                        ("Dív. líq./EBITDA", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.divida_liq_ebitda))),
                        ("Dív. líq./EBIT", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.divida_liq_ebit))),
                        ("PL / ativos", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.pl_ativos))),
                        ("Passivos / ativos", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.passivos_ativos))),
                        ("Liq. corrente", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.liquidez_corrente))),
                        ("Liq. seca", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.liquidez_seca))),
                    ]);
                    metric_group(&mut cols[2], "Eficiência", palette.positive, palette, &[
                        ("Margem bruta", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_bruta))),
                        ("Margem EBITDA", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_ebitda))),
                        ("Margem EBIT", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_ebit))),
                        ("Margem líquida", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.margem_liquida))),
                        ("Giro dos ativos", self.metric(sym, MetricId::Outside, &format!("{:.2}", ind.giro_ativos))),
                    ]);
                    metric_group(&mut cols[3], "Rentabilidade", palette.violet, palette, &[
                        ("ROE", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roe))),
                        ("ROIC", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roic))),
                        ("ROA", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.roa))),
                        ("CAGR receitas", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.cagr_receitas_5a))),
                        ("CAGR lucros", self.metric(sym, MetricId::Outside, &format!("{:.1}%", ind.cagr_lucros_5a))),
                        ("LPA", self.metric(sym, MetricId::Lpa, &format!("R$ {:.2}", ind.lpa))),
                        ("VPA", self.metric(sym, MetricId::Vpa, &format!("R$ {:.2}", ind.vpa))),
                        ("DPA", self.metric(sym, MetricId::Dpa, &format!("R$ {:.2}", ind.dpa))),
                    ]);
                });
            });
        });
    }

    pub(crate) fn render_comparison_table(&self, ui: &mut egui::Ui) {
        let palette = Palette::from_ui(ui);
        egui::Frame::group(ui.style())
            .fill(palette.card)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading(RichText::new("Matriz Comparativa de Indicadores Fundamentalistas").color(palette.text).size(16.0));
                ui.label(RichText::new("P/L, P/VP, dividend yield, Graham e Bazin usam a CVM quando o período existe. O restante é cenário local ou fica indisponível.").color(palette.muted).size(12.0));
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
                            ui.label(RichText::new(t).strong().color(palette.accent));
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

fn section_card(ui: &mut egui::Ui, fill: egui::Color32, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(fill)
        .inner_margin(12.0)
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            body(ui);
        });
}

fn metric_group(ui: &mut egui::Ui, title: &str, color: egui::Color32, palette: Palette, rows: &[(&str, String)]) {
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    ui.label(RichText::new(title).strong().color(color).size(13.0));
    ui.add_space(6.0);
    for (label, value) in rows {
        metric_line(ui, label, value.clone(), palette);
    }
}

fn metric_line(ui: &mut egui::Ui, label: &str, value: String, palette: Palette) {
    ui.add_space(3.0);
    ui.label(RichText::new(label).color(palette.muted).size(11.0));
    ui.label(RichText::new(value).strong().color(palette.text).size(13.0));
}
