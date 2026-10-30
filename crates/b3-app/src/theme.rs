use eframe::egui::{self, Color32, Stroke};

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub canvas: Color32,
    pub nav: Color32,
    pub nav_border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub subtle: Color32,
    pub label: Color32,
    pub accent: Color32,
    pub positive: Color32,
    pub negative: Color32,
    pub link: Color32,
    pub warning: Color32,
    pub violet: Color32,
    pub card: Color32,
    pub card_alt: Color32,
    pub card_inset: Color32,
    pub badge_bg: Color32,
    pub hover: Color32,
}

impl Palette {
    pub fn from_dark(dark: bool) -> Self {
        if dark {
            Self {
                dark,
                canvas: Color32::from_rgb(13, 17, 23),
                nav: Color32::from_rgb(13, 17, 23),
                nav_border: Color32::from_rgb(48, 54, 61),
                text: Color32::WHITE,
                muted: Color32::from_rgb(139, 148, 158),
                subtle: Color32::from_rgb(180, 195, 210),
                label: Color32::from_rgb(140, 155, 175),
                accent: Color32::from_rgb(0, 210, 160),
                positive: Color32::from_rgb(0, 220, 140),
                negative: Color32::from_rgb(255, 80, 90),
                link: Color32::from_rgb(100, 180, 255),
                warning: Color32::from_rgb(255, 180, 80),
                violet: Color32::from_rgb(180, 140, 255),
                card: Color32::from_rgb(22, 27, 34),
                card_alt: Color32::from_rgb(28, 33, 42),
                card_inset: Color32::from_rgb(16, 20, 28),
                badge_bg: Color32::from_rgb(48, 54, 61),
                hover: Color32::from_rgb(33, 38, 45),
            }
        } else {
            Self {
                dark,
                canvas: Color32::from_rgb(246, 248, 250),
                nav: Color32::WHITE,
                nav_border: Color32::from_rgb(208, 215, 222),
                text: Color32::from_rgb(31, 35, 40),
                muted: Color32::from_rgb(101, 109, 118),
                subtle: Color32::from_rgb(70, 80, 95),
                label: Color32::from_rgb(87, 96, 106),
                accent: Color32::from_rgb(0, 122, 90),
                positive: Color32::from_rgb(15, 122, 70),
                negative: Color32::from_rgb(190, 40, 50),
                link: Color32::from_rgb(9, 105, 218),
                warning: Color32::from_rgb(156, 98, 0),
                violet: Color32::from_rgb(110, 64, 180),
                card: Color32::WHITE,
                card_alt: Color32::from_rgb(246, 248, 250),
                card_inset: Color32::from_rgb(234, 238, 242),
                badge_bg: Color32::from_rgb(234, 238, 242),
                hover: Color32::from_rgb(234, 238, 242),
            }
        }
    }

    pub fn from_ui(ui: &egui::Ui) -> Self {
        Self::from_dark(ui.visuals().dark_mode)
    }

    pub fn visuals(self) -> egui::Visuals {
        let mut visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.window_fill = self.canvas;
        visuals.panel_fill = self.canvas;
        visuals.faint_bg_color = self.card_alt;
        visuals.extreme_bg_color = self.card_inset;
        visuals.code_bg_color = self.card;
        visuals.hyperlink_color = self.link;
        visuals.warn_fg_color = self.warning;
        visuals.error_fg_color = self.negative;
        visuals.weak_text_color = Some(self.muted);
        visuals.selection.bg_fill = self.accent.gamma_multiply(if self.dark { 0.45 } else { 0.28 });
        visuals.selection.stroke.color = self.accent;
        visuals.widgets.noninteractive.fg_stroke.color = self.text;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.nav_border);
        visuals.widgets.inactive.fg_stroke.color = self.text;
        visuals.widgets.hovered.fg_stroke.color = self.text;
        visuals.widgets.active.fg_stroke.color = self.text;
        visuals.widgets.open.fg_stroke.color = self.text;
        visuals
    }
}
