use eframe::egui::{self, Color32, RichText};
use b3_analytics::{ScreenerCriteria, ScreenerEngine};
use b3_core::indicators::StockIndicators;

use crate::{ActiveView, AppState};

impl AppState {
    pub(crate) fn render_screener_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Busca de ativos").size(18.0));
            ui.label(RichText::new("Fora de P/L, P/VP, DY, Graham e Bazin, os números desta tela são cenário local.").color(Color32::GRAY));
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

                        if ui.button("Ver decisão").clicked() {
                            self.apply_selected_ticker(&item.symbol);
                            self.active_view = ActiveView::Decision;
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
