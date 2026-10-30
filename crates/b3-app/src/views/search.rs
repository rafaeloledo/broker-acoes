use eframe::egui::{self, RichText};

use crate::theme::Palette;
use b3_core::ticker::AssetClass;

use crate::{AppState, MAX_VISIBLE_ASSETS};

impl AppState {
    pub(crate) fn render_search_modal(&mut self, ctx: &egui::Context) {
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
                let palette = Palette::from_ui(ui);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Pesquisa no Universo B3").strong().size(16.0).color(palette.text));
                    if let Some(slot) = self.search_target_slot {
                        ui.label(RichText::new(format!("(Alterando Slot {})", slot + 1)).color(palette.accent));
                    } else {
                        ui.label(RichText::new(format!("(Adicionando ativo {}/{})", self.visible_tickers.len() + 1, MAX_VISIBLE_ASSETS)).color(palette.accent));
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
                ui.label(RichText::new(format!("Encontrados {} ativos na B3", results.len())).color(palette.muted).size(12.0));

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
                                ui.label(RichText::new(asset.ticker).strong().color(palette.accent));
                                ui.label(asset.name);
                                ui.label(RichText::new(asset.sector).color(palette.muted));
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
}
