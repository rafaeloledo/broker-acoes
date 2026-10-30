use eframe::egui::{self, Color32, RichText, Stroke};

use crate::theme::Palette;
use crate::{ActiveView, AppState, MAX_VISIBLE_ASSETS};

const NAV: &[(ActiveView, &str)] = &[
    (ActiveView::Decision, "Decisão"),
    (ActiveView::News, "Notícias"),
    (ActiveView::GpuTerminal, "Terminal"),
    (ActiveView::MarketHeatmap, "Mapa"),
    (ActiveView::Screener, "Busca"),
    (ActiveView::PortfolioTax, "Carteira"),
    (ActiveView::StatusInvest, "Ficha"),
];

impl AppState {
    pub(crate) fn render_shell(&mut self, ui: &mut egui::Ui) {
        let palette = Palette::from_dark(self.dark_theme);
        ui.ctx().set_visuals(palette.visuals());
        ui.painter().rect_filled(ui.max_rect(), 0.0, palette.canvas);

        egui::Panel::top("github_nav")
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(palette.nav)
                    .inner_margin(egui::Margin::symmetric(12, 4)),
            )
            .show(ui, |ui| self.render_top_nav(ui, palette));

        if self.menu_open {
            egui::Panel::left("github_menu")
                .resizable(false)
                .exact_size(248.0)
                .show_separator_line(true)
                .frame(egui::Frame::new().fill(palette.nav).inner_margin(12.0))
                .show(ui, |ui| self.render_side_menu(ui, palette));
        }

        let max_width = match self.active_view {
            ActiveView::GpuTerminal | ActiveView::MarketHeatmap => 1280.0,
            _ => 1040.0,
        };
        show_centered(ui, max_width, |ui| {
            ui.add_space(12.0);
            self.render_watchlist(ui, palette);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            match self.active_view {
                ActiveView::Decision => self.render_decision_view(ui),
                ActiveView::News => self.render_news_view(ui),
                ActiveView::StatusInvest => self.render_status_invest_view(ui),
                ActiveView::GpuTerminal => self.render_gpu_terminal_view(ui),
                ActiveView::MarketHeatmap => self.render_heatmap_view(ui),
                ActiveView::Screener => self.render_screener_view(ui),
                ActiveView::PortfolioTax => self.render_portfolio_tax_view(ui),
            }
        });

        self.render_search_modal(ui.ctx());
    }

    fn render_top_nav(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let full_x = ui.max_rect().x_range();
        let mut selected_x = None;
        let row = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if hamburger(ui, palette) {
                self.menu_open = !self.menu_open;
            }
            ui.add_space(8.0);
            paint_avatar(ui, palette);
            ui.add_space(6.0);
            ui.label(RichText::new("Broker Ações").strong().size(15.0).color(palette.text));
            ui.add_space(16.0);

            for (view, label) in NAV {
                let selected = self.active_view == *view;
                let (clicked, x_range) = nav_tab(ui, label, self.nav_count(*view), selected, palette);
                if clicked {
                    self.active_view = *view;
                }
                if self.active_view == *view {
                    selected_x = Some(x_range);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if theme_chip(ui, "Claro", !self.dark_theme, palette) {
                    self.dark_theme = false;
                }
                if theme_chip(ui, "Escuro", self.dark_theme, palette) {
                    self.dark_theme = true;
                }
            });
        });

        let y = row.response.rect.bottom();
        ui.painter().hline(full_x, y, Stroke::new(1.0, palette.nav_border));
        if let Some(x_range) = selected_x {
            ui.painter().hline(x_range, y, Stroke::new(2.0, palette.text));
        }
    }

    fn render_side_menu(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.horizontal(|ui| {
            paint_avatar(ui, palette);
            ui.add_space(8.0);
            ui.label(RichText::new("Broker Ações").strong().color(palette.text));
        });
        ui.add_space(14.0);
        ui.label(RichText::new("Navegação").size(12.0).color(palette.muted));
        ui.add_space(6.0);
        for (view, label) in NAV {
            if menu_row(ui, label, self.nav_count(*view), self.active_view == *view, palette) {
                self.active_view = *view;
            }
        }
    }

    fn nav_count(&self, view: ActiveView) -> Option<usize> {
        match view {
            ActiveView::News => {
                let count = if !self.news_rows.is_empty() {
                    self.news_rows.len()
                } else {
                    self.decision_news.len()
                };
                (count > 0).then_some(count)
            }
            ActiveView::PortfolioTax => (!self.trades.is_empty()).then_some(self.trades.len()),
            _ => None,
        }
    }

    fn render_watchlist(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let mut slot_to_remove = None;
        let mut slot_to_change = None;

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.label(RichText::new("Ativos").color(palette.muted));

            for i in 0..self.visible_tickers.len() {
                let sym = self.visible_tickers[i].clone();
                let chip_color = if self.focused_ticker_idx == i {
                    palette.accent
                } else {
                    palette.text
                };
                if ui.button(RichText::new(&sym).strong().color(chip_color)).clicked() {
                    self.focused_ticker_idx = i;
                    self.news_ticker = sym.clone();
                    self.selected_ticker = sym;
                }
                if ui.small_button("trocar").clicked() {
                    slot_to_change = Some(i);
                }
                if self.visible_tickers.len() > 1 && ui.small_button("x").clicked() {
                    slot_to_remove = Some(i);
                }
            }

            if self.visible_tickers.len() < MAX_VISIBLE_ASSETS
                && ui
                    .button(RichText::new("+ Adicionar").strong().color(palette.accent))
                    .clicked()
            {
                self.search_target_slot = None;
                self.b3_search_query.clear();
                self.is_search_open = true;
                self.just_opened_search = true;
            }
        });

        if let Some(index) = slot_to_remove {
            self.visible_tickers.remove(index);
            if self.focused_ticker_idx >= self.visible_tickers.len() {
                self.focused_ticker_idx = self.visible_tickers.len().saturating_sub(1);
            }
            if !self.visible_tickers.is_empty() {
                self.selected_ticker = self.visible_tickers[self.focused_ticker_idx].clone();
            }
            self.persist_visible_tickers();
        }

        if let Some(index) = slot_to_change {
            self.search_target_slot = Some(index);
            self.b3_search_query.clear();
            self.is_search_open = true;
            self.just_opened_search = true;
        }
    }
}

fn show_centered(ui: &mut egui::Ui, max_width: f32, body: impl FnOnce(&mut egui::Ui)) {
    let available = ui.available_rect_before_wrap();
    let width = (available.width() - 32.0).clamp(0.0, max_width);
    let x = available.left() + (available.width() - width) * 0.5;
    let rect = egui::Rect::from_min_size(
        egui::pos2(x, available.top()),
        egui::vec2(width, available.height()),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("centered_page")
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        body,
    );
}

fn paint_avatar(ui: &mut egui::Ui, palette: Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 14.0, palette.accent);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "BA",
        egui::FontId::proportional(11.0),
        Color32::WHITE,
    );
}

fn hamburger(ui: &mut egui::Ui, palette: Palette) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 6.0, palette.hover);
    }
    let center = rect.center();
    for dy in [-5.0, 0.0, 5.0] {
        ui.painter().line_segment(
            [
                egui::pos2(center.x - 7.0, center.y + dy),
                egui::pos2(center.x + 7.0, center.y + dy),
            ],
            Stroke::new(1.6, palette.text),
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

fn nav_tab(
    ui: &mut egui::Ui,
    label: &str,
    count: Option<usize>,
    selected: bool,
    palette: Palette,
) -> (bool, egui::Rangef) {
    let color = if selected { palette.text } else { palette.muted };
    let galley = ui.painter().layout_no_wrap(label.to_owned(), egui::FontId::proportional(14.0), color);
    let text_width = galley.size().x;
    let count_galley = count.map(|n| {
        ui.painter()
            .layout_no_wrap(n.to_string(), egui::FontId::proportional(12.0), palette.muted)
    });
    let count_width = count_galley.as_ref().map(|galley| galley.size().x + 14.0).unwrap_or(0.0);
    let gap = if count.is_some() { 6.0 } else { 0.0 };
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(text_width + gap + count_width + 12.0, 36.0),
        egui::Sense::click(),
    );
    if response.hovered() {
        ui.painter().rect_filled(rect, 6.0, palette.hover);
    }
    let text_pos = egui::pos2(rect.left() + 6.0, rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(text_pos, galley, color);
    if let Some(count_galley) = count_galley {
        let badge = egui::Rect::from_min_size(
            egui::pos2(text_pos.x + text_width + gap, rect.center().y - 9.0),
            egui::vec2(count_galley.size().x + 12.0, 18.0),
        );
        ui.painter().rect_filled(badge, 9.0, palette.badge_bg);
        ui.painter().galley(
            egui::pos2(badge.left() + 6.0, badge.center().y - count_galley.size().y * 0.5),
            count_galley,
            palette.muted,
        );
    }
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    (response.clicked(), rect.x_range())
}

fn theme_chip(ui: &mut egui::Ui, label: &str, on: bool, palette: Palette) -> bool {
    let color = if on { palette.text } else { palette.muted };
    let galley = ui.painter().layout_no_wrap(label.to_owned(), egui::FontId::proportional(13.0), color);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(galley.size().x + 16.0, 26.0),
        egui::Sense::click(),
    );
    if on || response.hovered() {
        ui.painter().rect_filled(rect, 6.0, palette.hover);
    }
    if on {
        ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0, palette.nav_border), egui::StrokeKind::Inside);
    }
    ui.painter().galley(
        egui::pos2(rect.left() + 8.0, rect.center().y - galley.size().y * 0.5),
        galley,
        color,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

fn menu_row(ui: &mut egui::Ui, label: &str, count: Option<usize>, selected: bool, palette: Palette) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(rect, 6.0, palette.hover);
    }
    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())),
            2.0,
            palette.accent,
        );
    }
    let color = if selected { palette.text } else { palette.muted };
    ui.painter().text(
        rect.left_center() + egui::vec2(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        color,
    );
    if let Some(count) = count {
        ui.painter().text(
            rect.right_center() + egui::vec2(-10.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            count.to_string(),
            egui::FontId::proportional(12.0),
            palette.muted,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}
