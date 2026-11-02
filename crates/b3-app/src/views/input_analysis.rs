use eframe::egui::{self, RichText, Stroke};

use crate::theme::Palette;
use b3_core::indicators::StockIndicators;
use b3_core::ticker::{AssetClass, CompanyProfile};
use b3_analytics::TrendingEngine;

use crate::presentation::{change_percent, fundamentals_caption, price_caption, MetricId};
use crate::{ActiveView, AppState, InputAnalysisSort, InputAnalysisViewMode, MAX_VISIBLE_ASSETS};

/// Parses a comma-separated (or semicolon/newline) list of asset tickers.
/// Trims whitespace, uppercases, filters empty entries, and deduplicates preserving input order.
pub fn parse_input_tickers(input: &str) -> Vec<String> {
    let mut tickers = Vec::new();
    for part in input.split(&[',', ';', '\n', '\r'][..]) {
        let trimmed = part.trim().trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']');
        let clean = trimmed.trim().to_uppercase();
        if !clean.is_empty() && !tickers.contains(&clean) {
            tickers.push(clean);
        }
    }
    tickers
}

pub fn asset_class_name(class: AssetClass) -> &'static str {
    match class {
        AssetClass::Acao => "Ação",
        AssetClass::Fii => "Fundo Imobiliário (FII)",
        AssetClass::Etf => "ETF",
        AssetClass::Bdr => "BDR",
        AssetClass::Opcao => "Opção",
        AssetClass::Tesouro => "Tesouro",
        AssetClass::Indice => "Índice",
    }
}

pub fn asset_class_badge_label(class: AssetClass) -> &'static str {
    match class {
        AssetClass::Acao => "AÇÃO",
        AssetClass::Fii => "FII",
        AssetClass::Etf => "ETF",
        AssetClass::Bdr => "BDR",
        AssetClass::Opcao => "OPÇÃO",
        AssetClass::Tesouro => "TESOURO",
        AssetClass::Indice => "ÍNDICE",
    }
}

/// Safely retrieves text from the system clipboard using arboard.
pub fn get_clipboard_text() -> Option<String> {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        if let Ok(text) = cb.get_text() {
            let clean = text.trim();
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }
    None
}

impl AppState {
    pub(crate) fn render_input_analysis_view(&mut self, ui: &mut egui::Ui) {
        let palette = Palette::from_ui(ui);

        // Header / Intro
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Análise por Input de Ativos").size(22.0).color(palette.text));
            ui.separator();
            ui.label(
                RichText::new("Recebe N ativos da Bovespa (Ações, FIIs, ETFs, BDRs) separados por vírgula e exibe análise completa de cada um.")
                    .color(palette.muted)
                    .size(13.0),
            );
        });

        ui.add_space(8.0);

        // Input Box Container
        egui::Frame::group(ui.style())
            .fill(palette.card)
            .stroke(Stroke::new(1.0, palette.nav_border))
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Ativos a analisar:")
                            .strong()
                            .color(palette.text)
                            .size(14.0),
                    );
                    ui.label(
                        RichText::new("(separe os códigos por vírgula, ex: PETR4, VALE3, HGLG11, MXRF11)")
                            .color(palette.muted)
                            .size(12.0),
                    );
                });

                ui.add_space(4.0);

                // Text Edit field with Paste button
                let mut pasted_from_button = false;
                ui.horizontal(|ui| {
                    let mut text = self.input_analysis_text.clone();
                    let paste_btn_w = 90.0;
                    let text_edit_response = ui.add(
                        egui::TextEdit::singleline(&mut text)
                            .hint_text("Digite ou cole tickers separados por vírgula... (ex: PETR4, VALE3, HGLG11, MXRF11)")
                            .desired_width((ui.available_width() - paste_btn_w - 8.0).max(180.0))
                            .margin(egui::Margin::symmetric(10, 8))
                    );
                    if text_edit_response.changed() {
                        self.input_analysis_text = text;
                    }

                    // Check for Ctrl+V shortcut if not consumed by TextEdit
                    let shortcut_paste = ui.input(|i| {
                        (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::V)
                    });

                    if ui.button(RichText::new("📋 Colar").strong().color(palette.accent).size(13.0))
                        .on_hover_text("Colar texto da área de transferência (Clipboard) - Ctrl+V")
                        .clicked() || (shortcut_paste && !text_edit_response.has_focus())
                    {
                        if let Some(clip) = get_clipboard_text() {
                            let clean = clip.replace(['\r', '\n', ';'], ",");
                            if self.input_analysis_text.trim().is_empty() {
                                self.input_analysis_text = clean;
                            } else {
                                let trimmed = self.input_analysis_text.trim_end();
                                if trimmed.ends_with(',') {
                                    self.input_analysis_text = format!("{} {}", trimmed, clean);
                                } else {
                                    self.input_analysis_text = format!("{}, {}", trimmed, clean);
                                }
                            }
                            pasted_from_button = true;
                        }
                    }
                });

                ui.add_space(6.0);

                // Action buttons and Presets
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Exemplos rápidos:").color(palette.muted).size(12.0));

                    if ui.button(RichText::new("Ações & FIIs").size(12.0)).clicked() {
                        self.input_analysis_text = "PETR4, VALE3, WEGE3, HGLG11, MXRF11, XPML11".to_string();
                    }
                    if ui.button(RichText::new("Top Dividendos").size(12.0)).clicked() {
                        self.input_analysis_text = "BBAS3, TAEE11, EGIE3, PETR4, MXRF11, KNCR11".to_string();
                    }
                    if ui.button(RichText::new("Fundos Imobiliários").size(12.0)).clicked() {
                        self.input_analysis_text = "HGLG11, BTLG11, XPML11, VISC11, KNCR11, MXRF11".to_string();
                    }
                    if ui.button(RichText::new("Tech, ETFs & BDRs").size(12.0)).clicked() {
                        self.input_analysis_text = "WEGE3, TOTS3, IVVB11, BOVA11, AAPL34, NVDC34".to_string();
                    }
                    if ui.button(RichText::new("Small Caps & Energia").size(12.0)).clicked() {
                        self.input_analysis_text = "CPLE6, CMIG4, VBBR3, AZZA3, RANI3, KEPL3".to_string();
                    }

                    if !self.input_analysis_text.is_empty() {
                        if ui.button(RichText::new("📋 Substituir por Clipboard").size(12.0).color(palette.accent)).clicked() {
                            if let Some(clip) = get_clipboard_text() {
                                self.input_analysis_text = clip.replace(['\r', '\n', ';'], ",");
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("Limpar").color(palette.negative).size(12.0)).clicked() {
                            self.input_analysis_text.clear();
                        }
                        if pasted_from_button {
                            ui.label(RichText::new("✓ Colado!").color(palette.positive).size(12.0));
                        }
                    });
                });
            });

        ui.add_space(8.0);

        // Parse input tickers
        let parsed_tickers = parse_input_tickers(&self.input_analysis_text);

        // Ensure all input tickers are dynamically loaded and query real StatusInvest data
        for sym in &parsed_tickers {
            self.ensure_asset_loaded(sym);
            self.query_statusinvest(sym, false);
        }

        if parsed_tickers.is_empty() {
            // Empty state guide
            self.render_empty_state(ui, palette);
            return;
        }

        // Count categories
        let mut count_acoes = 0;
        let mut count_fiis = 0;
        let mut count_outros = 0;
        for sym in &parsed_tickers {
            match AssetClass::from_symbol(sym) {
                AssetClass::Acao => count_acoes += 1,
                AssetClass::Fii => count_fiis += 1,
                _ => count_outros += 1,
            }
        }

        // Filter and Control Bar
        ui.horizontal(|ui| {
            // StatusInvest Status & Refresh All Button
            let loading_count = parsed_tickers.iter().filter(|s| self.statusinvest_loading.contains(*s)).count();
            if loading_count > 0 {
                ui.label(
                    RichText::new(format!("🔄 Consultando StatusInvest ({} ativo(s))...", loading_count))
                        .color(palette.accent)
                        .strong()
                        .size(12.0),
                );
            }

            if ui.button(RichText::new("🔄 Atualizar StatusInvest").strong().color(palette.accent).size(12.0))
                .on_hover_text("Consulta dados em tempo real no StatusInvest para todos os ativos informados")
                .clicked()
            {
                for s in &parsed_tickers {
                    self.query_statusinvest(s, true);
                }
            }

            ui.separator();

            // Category badges
            ui.label(RichText::new(format!("{} ativos detectados:", parsed_tickers.len())).strong().color(palette.text));

            let filter_all_selected = self.input_analysis_filter_class.is_none();
            if ui.selectable_label(filter_all_selected, format!("Todos ({})", parsed_tickers.len())).clicked() {
                self.input_analysis_filter_class = None;
            }

            if count_acoes > 0 {
                let filter_acao_selected = self.input_analysis_filter_class == Some(AssetClass::Acao);
                if ui.selectable_label(filter_acao_selected, format!("Ações ({})", count_acoes)).clicked() {
                    self.input_analysis_filter_class = Some(AssetClass::Acao);
                }
            }

            if count_fiis > 0 {
                let filter_fii_selected = self.input_analysis_filter_class == Some(AssetClass::Fii);
                if ui.selectable_label(filter_fii_selected, format!("FIIs ({})", count_fiis)).clicked() {
                    self.input_analysis_filter_class = Some(AssetClass::Fii);
                }
            }

            if count_outros > 0 {
                let filter_outros_selected = self.input_analysis_filter_class == Some(AssetClass::Etf);
                if ui.selectable_label(filter_outros_selected, format!("ETFs/BDRs ({})", count_outros)).clicked() {
                    self.input_analysis_filter_class = Some(AssetClass::Etf);
                }
            }

            ui.separator();

            // Search filter within results
            ui.label(RichText::new("Filtrar:").color(palette.muted).size(12.0));
            ui.add(
                egui::TextEdit::singleline(&mut self.input_analysis_search)
                    .hint_text("Buscar ativo...")
                    .desired_width(120.0),
            );

            // Right-aligned view modes & sorting
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // View Mode buttons
                let is_table = self.input_analysis_view_mode == InputAnalysisViewMode::Table;
                if ui.selectable_label(is_table, "Tabela Comparativa").clicked() {
                    self.input_analysis_view_mode = InputAnalysisViewMode::Table;
                }
                let is_cards = self.input_analysis_view_mode == InputAnalysisViewMode::Cards;
                if ui.selectable_label(is_cards, "Cartões Detalhados").clicked() {
                    self.input_analysis_view_mode = InputAnalysisViewMode::Cards;
                }

                ui.separator();

                // Sort selector
                egui::ComboBox::from_id_salt("input_analysis_sort_combo")
                    .selected_text(match self.input_analysis_sort {
                        InputAnalysisSort::InputOrder => "Ordem de entrada",
                        InputAnalysisSort::TickerAsc => "Ticker (A-Z)",
                        InputAnalysisSort::PriceDesc => "Maior Preço",
                        InputAnalysisSort::DyDesc => "Maior Dividend Yield",
                        InputAnalysisSort::ChangeDesc => "Maior Variação Dia",
                        InputAnalysisSort::GrahamUpsideDesc => "Maior Upside Graham",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.input_analysis_sort, InputAnalysisSort::InputOrder, "Ordem de entrada");
                        ui.selectable_value(&mut self.input_analysis_sort, InputAnalysisSort::TickerAsc, "Ticker (A-Z)");
                        ui.selectable_value(&mut self.input_analysis_sort, InputAnalysisSort::PriceDesc, "Maior Preço");
                        ui.selectable_value(&mut self.input_analysis_sort, InputAnalysisSort::DyDesc, "Maior Dividend Yield");
                        ui.selectable_value(&mut self.input_analysis_sort, InputAnalysisSort::ChangeDesc, "Maior Variação Dia");
                        ui.selectable_value(&mut self.input_analysis_sort, InputAnalysisSort::GrahamUpsideDesc, "Maior Upside Graham");
                    });
                ui.label(RichText::new("Ordenar:").color(palette.muted).size(12.0));
            });
        });

        ui.add_space(8.0);

        // Filter tickers according to class and search text
        let search_norm = b3_core::b3_assets::normalize(self.input_analysis_search.trim());
        let mut display_tickers: Vec<String> = parsed_tickers
            .into_iter()
            .filter(|sym| {
                let class = AssetClass::from_symbol(sym);
                if let Some(target_class) = self.input_analysis_filter_class {
                    if target_class == AssetClass::Etf {
                        if class != AssetClass::Etf && class != AssetClass::Bdr {
                            return false;
                        }
                    } else if class != target_class {
                        return false;
                    }
                }
                if !search_norm.is_empty() {
                    let sym_norm = b3_core::b3_assets::normalize(sym);
                    let profile_name = self.company_profiles.get(sym).map(|p| b3_core::b3_assets::normalize(&p.name)).unwrap_or_default();
                    let profile_sector = self.company_profiles.get(sym).map(|p| b3_core::b3_assets::normalize(&p.subsector)).unwrap_or_default();
                    if !sym_norm.contains(&search_norm) && !profile_name.contains(&search_norm) && !profile_sector.contains(&search_norm) {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort items
        match self.input_analysis_sort {
            InputAnalysisSort::InputOrder => {}
            InputAnalysisSort::TickerAsc => display_tickers.sort(),
            InputAnalysisSort::PriceDesc => display_tickers.sort_by(|a, b| {
                let p_a = self.shown_price(a, 0.0);
                let p_b = self.shown_price(b, 0.0);
                p_b.partial_cmp(&p_a).unwrap_or(std::cmp::Ordering::Equal)
            }),
            InputAnalysisSort::DyDesc => display_tickers.sort_by(|a, b| {
                let dy_a = self.stock_indicators.get(a).map(|i| i.dividend_yield).unwrap_or(0.0);
                let dy_b = self.stock_indicators.get(b).map(|i| i.dividend_yield).unwrap_or(0.0);
                dy_b.partial_cmp(&dy_a).unwrap_or(std::cmp::Ordering::Equal)
            }),
            InputAnalysisSort::ChangeDesc => display_tickers.sort_by(|a, b| {
                let chg_a = self.previous_close(a).and_then(|prev| change_percent(self.shown_price(a, 0.0), prev)).unwrap_or(-999.0);
                let chg_b = self.previous_close(b).and_then(|prev| change_percent(self.shown_price(b, 0.0), prev)).unwrap_or(-999.0);
                chg_b.partial_cmp(&chg_a).unwrap_or(std::cmp::Ordering::Equal)
            }),
            InputAnalysisSort::GrahamUpsideDesc => display_tickers.sort_by(|a, b| {
                let up_a = self.compute_graham_upside(a).unwrap_or(-999.0);
                let up_b = self.compute_graham_upside(b).unwrap_or(-999.0);
                up_b.partial_cmp(&up_a).unwrap_or(std::cmp::Ordering::Equal)
            }),
        }

        // Render content based on View Mode
        egui::ScrollArea::vertical()
            .id_salt("input_analysis_results_scroll")
            .show(ui, |ui| {
                match self.input_analysis_view_mode {
                    InputAnalysisViewMode::Cards => {
                        for sym in &display_tickers {
                            self.render_asset_card(ui, sym, palette);
                            ui.add_space(10.0);
                        }
                    }
                    InputAnalysisViewMode::Table => {
                        self.render_comparative_table(ui, &display_tickers, palette);
                    }
                }
            });
    }

    fn render_empty_state(&mut self, ui: &mut egui::Ui, palette: Palette) {
        egui::Frame::group(ui.style())
            .fill(palette.card)
            .stroke(Stroke::new(1.0, palette.nav_border))
            .inner_margin(32.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(
                        RichText::new("Nenhum ativo inserido para análise")
                            .strong()
                            .size(20.0)
                            .color(palette.text),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Insira qualquer código de ativo negociado na Bovespa (B3) no campo acima, separados por vírgula.")
                            .color(palette.muted)
                            .size(14.0),
                    );
                    ui.label(
                        RichText::new("Suporta Ações ordinárias e preferenciais (PETR4, VALE3, WEGE3), Fundos Imobiliários e Fiagros (HGLG11, MXRF11, KNCR11), ETFs (IVVB11, BOVA11) e BDRs (AAPL34, NVDC34).")
                            .color(palette.subtle)
                            .size(13.0),
                    );
                    ui.add_space(16.0);

                    if ui.button(RichText::new("Carregar Exemplo com Ações e FIIs").strong().size(14.0).color(palette.accent)).clicked() {
                        self.input_analysis_text = "PETR4, VALE3, WEGE3, HGLG11, MXRF11, XPML11".to_string();
                    }
                });
            });
    }

    fn render_asset_card(&mut self, ui: &mut egui::Ui, sym: &str, palette: Palette) {
        let class = AssetClass::from_symbol(sym);
        let profile = self.company_profiles.get(sym).cloned().unwrap_or_else(|| CompanyProfile::default_for(sym));
        let ind = self.stock_indicators.get(sym).cloned().unwrap_or_else(|| StockIndicators::default_for(sym));
        let price = self.shown_price(sym, ind.price);
        let prev_close = self.previous_close(sym);
        let change = prev_close.and_then(|prev| change_percent(price, prev));
        let market_caption = price_caption(self.market_as_of.get(sym).copied());
        let cvm_caption = fundamentals_caption(self.cvm_year.get(sym).copied());

        // Trend / Technical analysis
        let candles = self.candles.get(sym).cloned().unwrap_or_default();
        let core = self.core_multiples.get(sym);
        let trend = TrendingEngine::analyze_stock(
            sym,
            &profile.name,
            &profile.subsector,
            price,
            &candles,
            &ind,
            core,
        );

        egui::Frame::group(ui.style())
            .fill(palette.card)
            .stroke(Stroke::new(1.0, palette.nav_border))
            .inner_margin(12.0)
            .show(ui, |ui| {
                // Header Row: Ticker, Badge, Name, Price, Variation
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(sym).strong().size(22.0).color(palette.accent));
                    ui.add_space(4.0);

                    // Badge
                    let (badge_text, badge_color) = match class {
                        AssetClass::Acao => ("AÇÃO", palette.link),
                        AssetClass::Fii => ("FII", palette.positive),
                        AssetClass::Etf => ("ETF", palette.violet),
                        AssetClass::Bdr => ("BDR", palette.warning),
                        _ => ("B3", palette.muted),
                    };
                    ui.label(
                        RichText::new(format!(" {} ", badge_text))
                            .size(11.0)
                            .strong()
                            .color(badge_color)
                            .background_color(palette.badge_bg),
                    );

                    ui.add_space(6.0);
                    ui.label(RichText::new(&profile.name).strong().color(palette.text).size(15.0));

                    // Right-aligned Price and Variation
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let price_str = if price > 0.0 {
                            format!("R$ {:.2}", price)
                        } else {
                            "indisponível".to_string()
                        };
                        ui.heading(RichText::new(price_str).strong().size(20.0).color(palette.text));
                        ui.add_space(8.0);

                        match change {
                            Some(val) => {
                                let (chg_color, sign) = if val >= 0.0 {
                                    (palette.positive, "+")
                                } else {
                                    (palette.negative, "")
                                };
                                ui.label(
                                    RichText::new(format!("{}{:.2}% vs dia anterior", sign, val))
                                        .color(chg_color)
                                        .strong()
                                        .size(13.0),
                                );
                            }
                            None => {
                                ui.label(RichText::new("variação local").color(palette.muted).size(12.0));
                            }
                        }
                    });
                });

                // Subtitle Row: Sector, Subsector, Market Status & StatusInvest Real Badge
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Setor: {}  |  Segmento: {}", profile.sector.display_name(), profile.subsector))
                            .color(palette.muted)
                            .size(12.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // StatusInvest Refresh button
                        if ui.small_button("🔄 Atualizar").on_hover_text("Reconsultar este ativo no StatusInvest agora").clicked() {
                            self.query_statusinvest(sym, true);
                        }

                        // StatusInvest Real Badge
                        if self.statusinvest_loading.contains(sym) {
                            ui.label(
                                RichText::new(" 🔄 Consultando StatusInvest... ")
                                    .size(11.0)
                                    .strong()
                                    .color(palette.accent)
                                    .background_color(palette.badge_bg),
                            );
                        } else if let Some(asset) = self.statusinvest_data.get(sym) {
                            let now = chrono::Utc::now().timestamp_millis();
                            let mins_ago = ((now - asset.fetched_at_ms) / 60_000).max(0);
                            let time_text = if mins_ago == 0 {
                                "agora".to_string()
                            } else {
                                format!("há {} min", mins_ago)
                            };
                            ui.label(
                                RichText::new(format!(" ✓ StatusInvest Real ({}) ", time_text))
                                    .size(11.0)
                                    .strong()
                                    .color(palette.positive)
                                    .background_color(palette.badge_bg),
                            ).on_hover_text(format!("Dados obtidos diretamente de statusinvest.com.br/{}/{}", asset.category, asset.ticker.to_lowercase()));
                        } else if let Some(err) = self.statusinvest_errors.get(sym) {
                            ui.label(
                                RichText::new(" ⚠ StatusInvest Offline ")
                                    .size(11.0)
                                    .color(palette.warning)
                                    .background_color(palette.badge_bg),
                            ).on_hover_text(err);
                        }

                        ui.label(RichText::new(format!("{}  ·  {}", market_caption, cvm_caption)).color(palette.label).size(11.0));
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Body Section tailored to Asset Class
                match class {
                    AssetClass::Fii => {
                        self.render_fii_details_card(ui, sym, &ind, price, palette);
                    }
                    AssetClass::Acao => {
                        self.render_acao_details_card(ui, sym, &ind, price, &trend, palette);
                    }
                    _ => {
                        self.render_other_asset_details_card(ui, sym, &ind, price, class, palette);
                    }
                }

                ui.add_space(8.0);

                // Card Footer / Action Bar
                ui.horizontal(|ui| {
                    // Trend signal badge
                    let (r, g, b) = trend.signal.color_code();
                    let sig_col = egui::Color32::from_rgb(r, g, b);
                    ui.label(
                        RichText::new(format!(" Sinal: {} (Score {:.0}/100) ", trend.signal.label(), trend.overall_score))
                            .size(12.0)
                            .strong()
                            .color(sig_col)
                            .background_color(palette.badge_bg),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Quick action buttons
                        if ui.button("Ver Decisão").clicked() {
                            self.apply_selected_ticker(sym);
                            self.active_view = ActiveView::Decision;
                        }

                        if ui.button("Ficha Completa").clicked() {
                            self.apply_selected_ticker(sym);
                            self.active_view = ActiveView::StatusInvest;
                        }

                        let in_watchlist = self.visible_tickers.contains(&sym.to_string());
                        if in_watchlist {
                            ui.label(RichText::new("✓ Na Watchlist").color(palette.accent).size(12.0));
                        } else if self.visible_tickers.len() < MAX_VISIBLE_ASSETS {
                            if ui.button("+ Watchlist").clicked() {
                                self.visible_tickers.push(sym.to_string());
                                self.persist_visible_tickers();
                            }
                        }
                    });
                });
            });
    }

    fn render_fii_details_card(&self, ui: &mut egui::Ui, sym: &str, ind: &StockIndicators, price: f64, palette: Palette) {
        let fii = self.fii_details.get(sym).cloned().unwrap_or_else(|| b3_core::fii::FiiDetails::default_for(sym));

        let vpa = if fii.nav_per_share > 0.0 { fii.nav_per_share } else { ind.vpa };
        let pvp = if vpa > 0.0 && price > 0.0 { price / vpa } else { ind.pvp };
        let dy = if fii.dividend_yield_12m > 0.0 { fii.dividend_yield_12m } else { ind.dividend_yield };
        let last_div = fii.last_dividend;
        let bazin_price = if last_div > 0.0 {
            (last_div * 12.0) / 0.06
        } else {
            ind.bazin_ceiling_price
        };
        let bazin_upside = if price > 0.0 && bazin_price > 0.0 {
            ((bazin_price - price) / price) * 100.0
        } else {
            0.0
        };

        ui.columns(2, |cols| {
            // Left Column: Valuation & Proventos
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Cotação, Desconto & Rendimentos").strong().color(palette.text).size(13.0));
                ui.add_space(4.0);

                egui::Grid::new(format!("fii_grid_val_{}", sym))
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("P/VP:").color(palette.label));
                        let pvp_color = if pvp < 1.0 { palette.positive } else { palette.warning };
                        let pvp_desc = if pvp < 1.0 {
                            format!("{:.2} (Desconto de {:.1}%)", pvp, (1.0 - pvp) * 100.0)
                        } else if pvp > 1.0 {
                            format!("{:.2} (Ágio de {:.1}%)", pvp, (pvp - 1.0) * 100.0)
                        } else {
                            format!("{:.2} (No par)", pvp)
                        };
                        ui.label(RichText::new(pvp_desc).strong().color(pvp_color));
                        ui.end_row();

                        ui.label(RichText::new("VPA (VP da Cota):").color(palette.label));
                        ui.label(RichText::new(format!("R$ {:.2}", vpa)).strong().color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("Último Rendimento:").color(palette.label));
                        ui.label(RichText::new(format!("R$ {:.2} / cota", last_div)).strong().color(palette.positive));
                        ui.end_row();

                        ui.label(RichText::new("Dividend Yield (12M):").color(palette.label));
                        ui.label(RichText::new(format!("{:.2}% a.a.", dy)).strong().color(palette.positive));
                        ui.end_row();

                        ui.label(RichText::new("Preço Teto Bazin (6%):").color(palette.label));
                        let up_col = if bazin_upside >= 0.0 { palette.positive } else { palette.negative };
                        ui.label(
                            RichText::new(format!("R$ {:.2} ({:+.1}%)", bazin_price, bazin_upside))
                                .strong()
                                .color(up_col),
                        );
                        ui.end_row();
                    });
            });

            // Right Column: Operacional & Gestão
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Estrutura, Vacância & Cotistas").strong().color(palette.text).size(13.0));
                ui.add_space(4.0);

                egui::Grid::new(format!("fii_grid_ops_{}", sym))
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Segmento FII:").color(palette.label));
                        ui.label(RichText::new(fii.segment.display_name()).strong().color(palette.accent));
                        ui.end_row();

                        ui.label(RichText::new("Vacância Física:").color(palette.label));
                        let vac_col = if fii.physical_vacancy_percent > 10.0 { palette.negative } else { palette.text };
                        ui.label(RichText::new(format!("{:.1}%", fii.physical_vacancy_percent)).color(vac_col));
                        ui.end_row();

                        ui.label(RichText::new("Vacância Financeira:").color(palette.label));
                        ui.label(RichText::new(format!("{:.1}%", fii.financial_vacancy_percent)).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("Número de Cotistas:").color(palette.label));
                        ui.label(RichText::new(format!("{}", fii.shareholders_count)).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("Gestora:").color(palette.label));
                        ui.label(RichText::new(&fii.manager).color(palette.subtle));
                        ui.end_row();

                        ui.label(RichText::new("Taxa de Administração:").color(palette.label));
                        ui.label(RichText::new(format!("{:.2}% a.a.", fii.admin_fee_percent)).color(palette.muted));
                        ui.end_row();
                    });
            });
        });

        // If properties exist, show a brief mention
        if !fii.properties.is_empty() {
            ui.add_space(4.0);
            let prop = &fii.properties[0];
            ui.label(
                RichText::new(format!("Ativo principal: {} ({}, {}) · {:.0} m²", prop.name, prop.city, prop.state, prop.area_m2))
                    .color(palette.muted)
                    .size(11.0),
            );
        }
    }

    fn render_acao_details_card(
        &self,
        ui: &mut egui::Ui,
        sym: &str,
        ind: &StockIndicators,
        _price: f64,
        trend: &b3_analytics::StockTrendAnalysis,
        palette: Palette,
    ) {
        // Valuation Top 4 Highlight Metric Cards
        let pl_str = format!("{:.2}", ind.pl);
        let pvp_str = format!("{:.2}", ind.pvp);
        let dy_str = format!("{:.2}%", ind.dividend_yield);
        let cards = [
            ("P/L", if self.statusinvest_data.contains_key(sym) { pl_str } else { self.metric(sym, MetricId::Pl, &pl_str) }),
            ("P/VP", if self.statusinvest_data.contains_key(sym) { pvp_str } else { self.metric(sym, MetricId::Pvp, &pvp_str) }),
            ("Dividend Yield", if self.statusinvest_data.contains_key(sym) { dy_str } else { self.metric(sym, MetricId::DividendYield, &dy_str) }),
            ("EV/EBITDA", if ind.ev_ebitda > 0.0 { format!("{:.2}", ind.ev_ebitda) } else { "-".to_string() }),
        ];

        ui.horizontal(|ui| {
            let card_w = (ui.available_width() - 36.0) / cards.len() as f32;
            for (label, val) in cards {
                egui::Frame::group(ui.style())
                    .fill(palette.card_alt)
                    .stroke(Stroke::new(1.0, palette.nav_border))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_width(card_w.max(70.0));
                        ui.label(RichText::new(label).color(palette.label).size(11.0));
                        ui.label(RichText::new(val).strong().color(palette.text).size(16.0));
                    });
            }
        });

        ui.add_space(6.0);

        // Fundamentals Grid (2 Columns)
        ui.columns(2, |cols| {
            // Left Column: Rentabilidade & Endividamento
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Rentabilidade & Saúde Financeira").strong().color(palette.text).size(13.0));
                ui.add_space(4.0);

                egui::Grid::new(format!("acao_grid_rent_{}", sym))
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("ROE:").color(palette.label));
                        ui.label(RichText::new(format!("{:.1}%", ind.roe)).strong().color(palette.positive));
                        ui.end_row();

                        ui.label(RichText::new("ROIC:").color(palette.label));
                        ui.label(RichText::new(if ind.roic > 0.0 { format!("{:.1}%", ind.roic) } else { "-".to_string() }).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("Margem Líquida:").color(palette.label));
                        ui.label(RichText::new(format!("{:.1}%", ind.margem_liquida)).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("Margem Bruta:").color(palette.label));
                        ui.label(RichText::new(format!("{:.1}%", ind.margem_bruta)).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("Dív. Líquida / EBITDA:").color(palette.label));
                        let div_col = if ind.divida_liq_ebitda > 3.0 { palette.negative } else { palette.text };
                        ui.label(RichText::new(format!("{:.2}x", ind.divida_liq_ebitda)).color(div_col));
                        ui.end_row();

                        ui.label(RichText::new("Liquidez Corrente:").color(palette.label));
                        ui.label(RichText::new(format!("{:.2}", ind.liquidez_corrente)).color(palette.text));
                        ui.end_row();
                    });
            });

            // Right Column: Valuation Clássico & Por Ação
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Modelos de Valuation & Por Ação").strong().color(palette.text).size(13.0));
                ui.add_space(4.0);

                let graham_val = self.metric(sym, MetricId::Graham, &format!("R$ {:.2}", ind.graham_fair_value));
                let graham_upside = self.compute_graham_upside(sym);

                let bazin_val = self.metric(sym, MetricId::Bazin, &format!("R$ {:.2}", ind.bazin_ceiling_price));
                let bazin_upside = self.compute_bazin_upside(sym);

                egui::Grid::new(format!("acao_grid_val_{}", sym))
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Preço Justo Graham:").color(palette.label));
                        let g_col = graham_upside.map_or(palette.text, |u| if u >= 0.0 { palette.positive } else { palette.negative });
                        let g_str = match graham_upside {
                            Some(u) => format!("{} ({:+.1}%)", graham_val, u),
                            None => graham_val,
                        };
                        ui.label(RichText::new(g_str).strong().color(g_col));
                        ui.end_row();

                        ui.label(RichText::new("Preço Teto Bazin (6%):").color(palette.label));
                        let b_col = bazin_upside.map_or(palette.text, |u| if u >= 0.0 { palette.positive } else { palette.negative });
                        let b_str = match bazin_upside {
                            Some(u) => format!("{} ({:+.1}%)", bazin_val, u),
                            None => bazin_val,
                        };
                        ui.label(RichText::new(b_str).strong().color(b_col));
                        ui.end_row();

                        ui.label(RichText::new("LPA (Lucro por Ação):").color(palette.label));
                        ui.label(RichText::new(self.metric(sym, MetricId::Lpa, &format!("R$ {:.2}", ind.lpa))).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("VPA (Valor Patr. por Ação):").color(palette.label));
                        ui.label(RichText::new(self.metric(sym, MetricId::Vpa, &format!("R$ {:.2}", ind.vpa))).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("DPA (Dividendo por Ação):").color(palette.label));
                        ui.label(RichText::new(self.metric(sym, MetricId::Dpa, &format!("R$ {:.2}", ind.dpa))).color(palette.positive));
                        ui.end_row();

                        ui.label(RichText::new("Piotroski F-Score:").color(palette.label));
                        ui.label(RichText::new(format!("{}/9", ind.piotroski_f_score)).color(palette.text));
                        ui.end_row();

                        ui.label(RichText::new("IFR / RSI (14):").color(palette.label));
                        let rsi_str = trend.rsi_14.map_or("-".to_string(), |r| format!("{:.1} ({})", r, trend.rsi_status));
                        ui.label(RichText::new(rsi_str).color(palette.muted));
                        ui.end_row();
                    });
            });
        });
    }

    fn render_other_asset_details_card(
        &self,
        ui: &mut egui::Ui,
        sym: &str,
        ind: &StockIndicators,
        price: f64,
        class: AssetClass,
        palette: Palette,
    ) {
        let profile = self.company_profiles.get(sym).cloned().unwrap_or_else(|| CompanyProfile::default_for(sym));

        ui.horizontal(|ui| {
            egui::Grid::new(format!("other_asset_grid_{}", sym))
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Tipo de Ativo:").color(palette.label));
                    ui.label(RichText::new(asset_class_name(class)).strong().color(palette.accent));
                    ui.end_row();

                    ui.label(RichText::new("Referência / Setor:").color(palette.label));
                    ui.label(RichText::new(&profile.subsector).strong().color(palette.text));
                    ui.end_row();

                    ui.label(RichText::new("Preço Atual:").color(palette.label));
                    ui.label(RichText::new(format!("R$ {:.2}", price)).strong().color(palette.positive));
                    ui.end_row();

                    if ind.dividend_yield > 0.0 {
                        ui.label(RichText::new("Dividend Yield:").color(palette.label));
                        ui.label(RichText::new(format!("{:.2}%", ind.dividend_yield)).strong().color(palette.positive));
                        ui.end_row();
                    }

                    if ind.pvp > 0.0 {
                        ui.label(RichText::new("P/VP:").color(palette.label));
                        ui.label(RichText::new(format!("{:.2}", ind.pvp)).color(palette.text));
                        ui.end_row();
                    }

                    ui.label(RichText::new("Descrição do Ativo:").color(palette.label));
                    ui.label(RichText::new(&profile.description).color(palette.muted));
                    ui.end_row();
                });
        });
    }

    fn render_comparative_table(&mut self, ui: &mut egui::Ui, tickers: &[String], palette: Palette) {
        ui.label(RichText::new("Tabela Comparativa Consolidada").strong().size(14.0).color(palette.text));
        ui.add_space(4.0);

        egui::Grid::new("input_analysis_comparative_grid")
            .striped(true)
            .spacing([12.0, 7.0])
            .show(ui, |ui| {
                // Table Headers
                ui.label(RichText::new("Ticker").strong());
                ui.label(RichText::new("Tipo").strong());
                ui.label(RichText::new("Nome").strong());
                ui.label(RichText::new("Preço").strong());
                ui.label(RichText::new("Var 1d").strong());
                ui.label(RichText::new("P/VP").strong());
                ui.label(RichText::new("P/L").strong());
                ui.label(RichText::new("DY").strong());
                ui.label(RichText::new("Graham / VPA").strong());
                ui.label(RichText::new("Bazin Teto").strong());
                ui.label(RichText::new("ROE / Rend.").strong());
                ui.label(RichText::new("Sinal").strong());
                ui.label(RichText::new("Status").strong());
                ui.label(RichText::new("Ação").strong());
                ui.end_row();

                for sym in tickers {
                    let class = AssetClass::from_symbol(sym);
                    let profile = self.company_profiles.get(sym).cloned().unwrap_or_else(|| CompanyProfile::default_for(sym));
                    let ind = self.stock_indicators.get(sym).cloned().unwrap_or_else(|| StockIndicators::default_for(sym));
                    let price = self.shown_price(sym, ind.price);
                    let prev_close = self.previous_close(sym);
                    let change = prev_close.and_then(|prev| change_percent(price, prev));

                    // Ticker
                    ui.label(RichText::new(sym).strong().color(palette.accent));

                    // Tipo
                    ui.label(RichText::new(asset_class_badge_label(class)).color(palette.muted).size(11.0));

                    // Nome
                    let short_name: String = profile.name.chars().take(24).collect();
                    ui.label(RichText::new(short_name).color(palette.text));

                    // Preço
                    ui.label(RichText::new(format!("R$ {:.2}", price)).strong());

                    // Variação
                    match change {
                        Some(val) => {
                            let col = if val >= 0.0 { palette.positive } else { palette.negative };
                            ui.label(RichText::new(format!("{:+.2}%", val)).color(col));
                        }
                        None => {
                            ui.label(RichText::new("-").color(palette.muted));
                        }
                    }

                    // P/VP
                    let pvp = if class == AssetClass::Fii {
                        if let Some(fii) = self.fii_details.get(sym) {
                            if fii.nav_per_share > 0.0 { price / fii.nav_per_share } else { ind.pvp }
                        } else {
                            ind.pvp
                        }
                    } else {
                        ind.pvp
                    };
                    let pvp_col = if pvp < 1.0 { palette.positive } else { palette.text };
                    ui.label(RichText::new(format!("{:.2}", pvp)).color(pvp_col));

                    // P/L
                    if class == AssetClass::Fii {
                        ui.label(RichText::new("N/A (FII)").color(palette.muted).size(11.0));
                    } else {
                        ui.label(RichText::new(format!("{:.2}", ind.pl)));
                    }

                    // DY
                    let dy = if class == AssetClass::Fii {
                        self.fii_details.get(sym).map(|f| f.dividend_yield_12m).unwrap_or(ind.dividend_yield)
                    } else {
                        ind.dividend_yield
                    };
                    let dy_col = if dy >= 6.0 { palette.positive } else { palette.accent };
                    ui.label(RichText::new(format!("{:.2}%", dy)).color(dy_col).strong());

                    // Graham / VPA
                    if class == AssetClass::Fii {
                        let vpa = self.fii_details.get(sym).map(|f| f.nav_per_share).unwrap_or(ind.vpa);
                        ui.label(RichText::new(format!("VPA R${:.2}", vpa)).color(palette.muted));
                    } else {
                        let g_val = ind.graham_fair_value;
                        ui.label(RichText::new(format!("R$ {:.2}", g_val)).color(palette.text));
                    }

                    // Bazin Teto
                    let bazin = if class == AssetClass::Fii {
                        if let Some(fii) = self.fii_details.get(sym) {
                            if fii.last_dividend > 0.0 { (fii.last_dividend * 12.0) / 0.06 } else { ind.bazin_ceiling_price }
                        } else {
                            ind.bazin_ceiling_price
                        }
                    } else {
                        ind.bazin_ceiling_price
                    };
                    ui.label(RichText::new(format!("R$ {:.2}", bazin)).color(palette.text));

                    // ROE / Rendimento
                    if class == AssetClass::Fii {
                        let last_rend = self.fii_details.get(sym).map(|f| f.last_dividend).unwrap_or(0.0);
                        ui.label(RichText::new(format!("R$ {:.2}/cota", last_rend)).color(palette.positive));
                    } else {
                        ui.label(RichText::new(format!("{:.1}%", ind.roe)).color(palette.text));
                    }

                    // Sinal
                    let candles = self.candles.get(sym).cloned().unwrap_or_default();
                    let core = self.core_multiples.get(sym);
                    let trend = TrendingEngine::analyze_stock(
                        sym,
                        &profile.name,
                        &profile.subsector,
                        price,
                        &candles,
                        &ind,
                        core,
                    );
                    let (r, g, b) = trend.signal.color_code();
                    ui.label(RichText::new(trend.signal.label()).color(egui::Color32::from_rgb(r, g, b)).strong());

                    // Status
                    if self.statusinvest_loading.contains(sym) {
                        ui.label(RichText::new("🔄 Buscando").color(palette.accent).size(11.0));
                    } else if self.statusinvest_data.contains_key(sym) {
                        ui.label(RichText::new("✓ StatusInvest").color(palette.positive).size(11.0)).on_hover_text("Valores reais e atualizados do StatusInvest");
                    } else if let Some(err) = self.statusinvest_errors.get(sym) {
                        ui.label(RichText::new("⚠ Erro").color(palette.warning).size(11.0)).on_hover_text(err);
                    } else {
                        ui.label(RichText::new("-").color(palette.muted).size(11.0));
                    }

                    // Ação
                    ui.horizontal(|ui| {
                        if ui.button("Ver Decisão").clicked() {
                            self.apply_selected_ticker(sym);
                            self.active_view = ActiveView::Decision;
                        }
                        if ui.small_button("🔄").on_hover_text("Atualizar este ativo no StatusInvest").clicked() {
                            self.query_statusinvest(sym, true);
                        }
                    });

                    ui.end_row();
                }
            });
    }

    fn compute_graham_upside(&self, sym: &str) -> Option<f64> {
        let price = self.shown_price(sym, 0.0);
        if price <= 0.0 {
            return None;
        }
        let fair = if self.cvm_year.contains_key(sym) {
            self.core_multiples.get(sym).and_then(|m| m.graham_fair_value)
        } else {
            self.stock_indicators.get(sym).map(|i| i.graham_fair_value)
        };
        fair.map(|f| ((f - price) / price) * 100.0)
    }

    fn compute_bazin_upside(&self, sym: &str) -> Option<f64> {
        let price = self.shown_price(sym, 0.0);
        if price <= 0.0 {
            return None;
        }
        let bazin = if self.cvm_year.contains_key(sym) {
            self.core_multiples.get(sym).and_then(|m| m.bazin_ceiling_price)
        } else {
            self.stock_indicators.get(sym).map(|i| i.bazin_ceiling_price)
        };
        bazin.map(|b| ((b - price) / price) * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_tickers_comma_separated() {
        let input = "PETR4, VALE3, HGLG11, MXRF11";
        let parsed = parse_input_tickers(input);
        assert_eq!(parsed, vec!["PETR4", "VALE3", "HGLG11", "MXRF11"]);
    }

    #[test]
    fn test_parse_input_tickers_extra_spaces_and_casing() {
        let input = " petr4 ,  vale3,  hglg11  ";
        let parsed = parse_input_tickers(input);
        assert_eq!(parsed, vec!["PETR4", "VALE3", "HGLG11"]);
    }

    #[test]
    fn test_parse_input_tickers_semicolon_and_newlines() {
        let input = "PETR4; VALE3\nMXRF11, XPML11\r\nBTLG11";
        let parsed = parse_input_tickers(input);
        assert_eq!(parsed, vec!["PETR4", "VALE3", "MXRF11", "XPML11", "BTLG11"]);
    }

    #[test]
    fn test_parse_input_tickers_deduplication() {
        let input = "PETR4, VALE3, PETR4, vale3, HGLG11";
        let parsed = parse_input_tickers(input);
        assert_eq!(parsed, vec!["PETR4", "VALE3", "HGLG11"]);
    }

    #[test]
    fn test_parse_input_tickers_empty_tokens() {
        let input = ",,,  , PETR4,,, , ";
        let parsed = parse_input_tickers(input);
        assert_eq!(parsed, vec!["PETR4"]);
    }

    #[test]
    fn test_parse_input_tickers_quotes_and_brackets() {
        let input = "[\"PETR4\", 'VALE3', [HGLG11], 'MXRF11']";
        let parsed = parse_input_tickers(input);
        assert_eq!(parsed, vec!["PETR4", "VALE3", "HGLG11", "MXRF11"]);
    }

    #[test]
    fn test_parse_input_tickers_n_assets() {
        let input = "PETR4, VALE3, WEGE3, ITUB4, BBAS3, HGLG11, KNCR11, MXRF11, XPML11, IVVB11, AAPL34";
        let parsed = parse_input_tickers(input);
        assert_eq!(parsed.len(), 11);
        assert_eq!(parsed[0], "PETR4");
        assert_eq!(parsed[10], "AAPL34");
    }

    #[test]
    fn test_asset_class_labels_and_names() {
        assert_eq!(asset_class_name(AssetClass::Acao), "Ação");
        assert_eq!(asset_class_name(AssetClass::Fii), "Fundo Imobiliário (FII)");
        assert_eq!(asset_class_name(AssetClass::Etf), "ETF");
        assert_eq!(asset_class_name(AssetClass::Bdr), "BDR");

        assert_eq!(asset_class_badge_label(AssetClass::Acao), "AÇÃO");
        assert_eq!(asset_class_badge_label(AssetClass::Fii), "FII");
        assert_eq!(asset_class_badge_label(AssetClass::Etf), "ETF");
        assert_eq!(asset_class_badge_label(AssetClass::Bdr), "BDR");
    }

    #[test]
    fn test_get_clipboard_text_does_not_panic() {
        let _ = get_clipboard_text();
    }

    #[test]
    fn test_statusinvest_asset_into_indicators() {
        let mock_asset = b3_ingest::statusinvest::StatusInvestAsset {
            ticker: "PETR4".to_string(),
            name: "Petróleo Brasileiro S.A.".to_string(),
            category: "acoes".to_string(),
            price: 52.71,
            change_percent: Some(0.98),
            previous_close: Some(52.20),
            min_52s: Some(34.10),
            max_52s: Some(54.00),
            dividend_yield: 6.96,
            pl: 5.09,
            pvp: 1.41,
            ev_ebitda: 3.12,
            ev_ebit: 3.65,
            p_ebitda: 2.80,
            p_ebit: 3.20,
            vpa: 37.32,
            lpa: 10.35,
            dpa: 3.67,
            p_sr: 0.95,
            p_ativo: 0.55,
            p_cap_giro: 4.20,
            p_ativo_circ_liq: -1.2,
            peg_ratio: 0.85,
            divida_liq_pl: 0.65,
            divida_liq_ebitda: 0.85,
            divida_liq_ebit: 1.10,
            liquidez_corrente: 1.45,
            margem_bruta: 52.3,
            margem_ebitda: 42.1,
            margem_ebit: 33.2,
            margem_liquida: 24.5,
            roe: 27.73,
            roa: 12.4,
            roic: 21.8,
            giro_ativos: 0.62,
            cagr_receitas_5a: 14.5,
            cagr_lucros_5a: 18.2,
            graham_fair_value: 92.4,
            bazin_ceiling_price: 61.16,
            fii_nav_per_share: None,
            fii_last_dividend: None,
            fii_shareholders_count: None,
            fii_segment: None,
            fii_physical_vacancy: None,
            fii_financial_vacancy: None,
            fii_net_asset_value: None,
            sector: "Petróleo, Gás e Biocombustíveis".to_string(),
            subsector: "Petróleo, Gás e Biocombustíveis".to_string(),
            segment: "Exploração, Refino e Distribuição".to_string(),
            fetched_at_ms: 1700000000,
        };

        let ind = mock_asset.into_stock_indicators();
        assert_eq!(ind.symbol, "PETR4");
        assert_eq!(ind.price, 52.71);
        assert_eq!(ind.pl, 5.09);
        assert_eq!(ind.pvp, 1.41);
        assert_eq!(ind.dividend_yield, 6.96);
        assert_eq!(ind.roe, 27.73);
        assert_eq!(ind.vpa, 37.32);
        assert_eq!(ind.lpa, 10.35);
    }
}
