use eframe::egui::{self, RichText};

use crate::theme::Palette;
use crate::AppState;
use crate::presentation::{change_percent, format_metric, fundamentals_caption, price_caption, MetricId};

impl AppState {
    pub(crate) fn render_decision_view(&mut self, ui: &mut egui::Ui) {
        let palette = Palette::from_ui(ui);
        let Some(sym) = self
            .visible_tickers
            .get(self.focused_ticker_idx)
            .cloned()
            .or_else(|| self.visible_tickers.first().cloned())
        else {
            ui.label(RichText::new("Nenhum ativo na watchlist.").color(palette.muted));
            return;
        };

        let profile_name = self
            .company_profiles
            .get(&sym)
            .map(|profile| profile.name.clone())
            .unwrap_or_default();
        let has_market = self.market_as_of.contains_key(&sym);
        let price = self.shown_price(&sym, 0.0);
        let change = if has_market {
            self.previous_close(&sym)
                .and_then(|previous| change_percent(price, previous))
        } else {
            None
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(RichText::new("Decisão").color(palette.muted).size(12.0));
            ui.heading(RichText::new(&sym).strong().size(28.0).color(palette.text));
            if !profile_name.is_empty() {
                ui.label(RichText::new(profile_name).color(palette.subtle).size(14.0));
            }
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let price_text = if has_market {
                    format!("R$ {price:.2}")
                } else {
                    "indisponível".to_string()
                };
                ui.label(RichText::new(price_text).strong().size(26.0).color(palette.positive));
                ui.label(RichText::new(price_caption(self.market_as_of.get(&sym).copied())).color(palette.muted).size(13.0));
                match change {
                    Some(value) => {
                        let color = if value >= 0.0 {
                            palette.positive
                        } else {
                            palette.negative
                        };
                        ui.label(RichText::new(format!("{value:+.2}% vs candle anterior")).color(color).size(13.0));
                    }
                    None => {
                        ui.label(RichText::new("variação indisponível").color(palette.muted).size(13.0));
                    }
                }
            });

            ui.label(
                RichText::new(fundamentals_caption(self.cvm_year.get(&sym).copied()))
                    .color(palette.muted)
                    .size(12.0),
            );
            ui.label(
                RichText::new("Preço de mercado quando a coleta existe. Livro, tape e o pulso do mapa são simulação.")
                    .color(palette.muted)
                    .size(12.0),
            );

            ui.add_space(12.0);
            ui.label(RichText::new("Valuation").strong().size(14.0));
            ui.add_space(6.0);

            let cards = [
                ("P/L", MetricId::Pl),
                ("P/VP", MetricId::Pvp),
                ("Dividend yield", MetricId::DividendYield),
                ("Graham", MetricId::Graham),
                ("Bazin", MetricId::Bazin),
            ];
            ui.horizontal(|ui| {
                let card_w = (ui.available_width() - 32.0) / cards.len() as f32;
                for (label, metric) in cards {
                    let value = self.decision_metric(&sym, metric);
                    egui::Frame::group(ui.style())
                        .fill(palette.card_alt)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.set_min_width(card_w.max(80.0));
                            ui.label(RichText::new(label).color(palette.label).size(12.0));
                            ui.label(RichText::new(value).strong().color(palette.text).size(18.0));
                        });
                }
            });

            self.render_decision_news(ui, &sym);
        });
    }

    fn decision_metric(&self, sym: &str, metric: MetricId) -> String {
        if self.cvm_year.contains_key(sym) {
            format_metric(self.core_multiples.get(sym), metric)
        } else {
            "indisponível".to_string()
        }
    }

    fn previous_close(&self, sym: &str) -> Option<f64> {
        let candles = self.candles.get(sym)?;
        let index = candles.len().checked_sub(2)?;
        Some(candles.get(index)?.close)
    }
}
