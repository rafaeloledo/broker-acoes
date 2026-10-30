use eframe::egui::{self, RichText};

use crate::theme::Palette;

use b3_core::news::NewsItem;
use chrono::{DateTime, Utc};

use crate::AppState;

impl AppState {
    pub(crate) fn render_news_view(&mut self, ui: &mut egui::Ui) {
        let palette = Palette::from_ui(ui);
        if self.news_ticker.is_empty() {
            self.news_ticker = self
                .visible_tickers
                .get(self.focused_ticker_idx)
                .cloned()
                .unwrap_or_default();
        }
        self.reload_news_rows();

        ui.heading(RichText::new("Notícias").size(18.0));
        ui.label(RichText::new("Fatos relevantes da CVM e itens do RSS que citam o ativo.").color(palette.muted));
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            ui.label("Ativo:");
            let tickers = self.visible_tickers.clone();
            for ticker in tickers {
                if ui.selectable_label(self.news_ticker == ticker, &ticker).clicked() {
                    self.news_ticker = ticker;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.news_text).hint_text("Texto no título").desired_width(220.0));
            for (label, days) in [("7 dias", Some(7i64)), ("30 dias", Some(30)), ("12 meses", Some(365)), ("Tudo", None)] {
                if ui.selectable_label(self.news_period_days == days, label).clicked() {
                    self.news_period_days = days;
                }
            }
        });
        ui.add_space(8.0);

        if self.news_rows.is_empty() {
            ui.label(RichText::new(self.empty_news_message()).color(palette.muted));
            return;
        }

        let rows = self.news_rows.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for item in rows {
                news_row(ui, &item);
            }
        });
    }

    pub(crate) fn render_decision_news(&mut self, ui: &mut egui::Ui, sym: &str) {
        let palette = Palette::from_ui(ui);
        if self.decision_news_ticker != sym || self.decision_news_generation != self.news_generation {
            self.decision_news = self.db.search_news(Some(sym), None, None, None, 5).unwrap_or_default();
            self.decision_news_ticker = sym.to_string();
            self.decision_news_generation = self.news_generation;
            self.news_attempted_at = self.db.latest_news_attempt().ok().flatten();
        }

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Notícias").strong().size(16.0));
            if ui.button("Ver todas").clicked() {
                self.news_ticker = sym.to_string();
                self.active_view = super::super::ActiveView::News;
            }
        });
        if self.decision_news.is_empty() {
            ui.label(RichText::new(self.empty_news_message()).color(palette.muted).size(12.0));
            return;
        }
        let rows = self.decision_news.clone();
        for item in rows {
            news_row(ui, &item);
        }
    }

    fn reload_news_rows(&mut self) {
        let key = format!("{}|{}|{:?}", self.news_ticker, self.news_text, self.news_period_days);
        if key == self.news_loaded_key && self.news_generation == self.news_loaded_generation {
            return;
        }
        let now = Utc::now().timestamp_millis();
        let from_ms = self.news_period_days.map(|days| now - days * 86_400_000);
        let text = if self.news_text.trim().is_empty() {
            None
        } else {
            Some(self.news_text.as_str())
        };
        self.news_rows = self
            .db
            .search_news(Some(&self.news_ticker), text, from_ms, None, 200)
            .unwrap_or_default();
        self.news_attempted_at = self.db.latest_news_attempt().ok().flatten();
        self.news_loaded_key = key;
        self.news_loaded_generation = self.news_generation;
    }

    fn empty_news_message(&self) -> String {
        match self.news_attempted_at {
            Some(ms) => format!("Nenhuma notícia neste filtro. Última coleta: {}.", format_brt(ms)),
            None => "Ainda não houve coleta de notícias.".to_string(),
        }
    }
}

fn news_row(ui: &mut egui::Ui, item: &NewsItem) {
    let palette = Palette::from_ui(ui);
    let response = egui::Frame::group(ui.style())
        .fill(palette.card)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(RichText::new(&item.title).strong().color(palette.text));
            ui.label(
                RichText::new(format!("{} · {} · {}", item.source, item.ticker, format_brt(item.published_at_ms)))
                    .color(palette.muted)
                    .size(12.0),
            );
        })
        .response;
    if response.interact(egui::Sense::click()).clicked() {
        open_http(&item.url);
    }
    ui.add_space(4.0);
}

fn format_brt(ms: i64) -> String {
    let Some(utc) = DateTime::<Utc>::from_timestamp_millis(ms) else {
        return String::new();
    };
    (utc + chrono::Duration::hours(-3)).format("%d/%m/%Y %H:%M").to_string()
}

fn open_http(url: &str) {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return;
    }
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
}
