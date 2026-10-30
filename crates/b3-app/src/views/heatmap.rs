use eframe::egui::{self, Color32, RichText, Sense, Vec2};
use b3_gpu::egui_callback::GpuTreemapCallback;
use b3_gpu::treemap::TreemapUniforms;

use crate::{ActiveView, AppState};

impl AppState {
    pub(crate) fn render_heatmap_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Mapa").color(Color32::WHITE).size(18.0));
            ui.separator();
            ui.label(RichText::new("Verde = Alta | Vermelho = Baixa | Pulso branco = simulação de ticks").color(Color32::GRAY));
        });

        ui.add_space(6.0);

        let area_w = ui.available_width();
        let area_h = ui.available_height().max(420.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(area_w, area_h), Sense::click());

        // Check clicks on treemap cells to switch active ticker
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let norm_x = (pos.x - rect.left()) / rect.width();
                let norm_y = (pos.y - rect.top()) / rect.height();

                let mut clicked_ticker = None;
                for (idx, node) in self.heatmap_nodes.iter().enumerate() {
                    if norm_x >= node.x && norm_x <= (node.x + node.w) && norm_y >= node.y && norm_y <= (node.y + node.h) {
                        if let Some(t) = self.heatmap_tickers.get(idx) {
                            clicked_ticker = Some(t.clone());
                            break;
                        }
                    }
                }
                if let Some(t) = clicked_ticker {
                    self.apply_selected_ticker(&t);
                    self.active_view = ActiveView::Decision;
                }
            }
        }

        // Render with GPU Treemap pipeline
        let uniforms = TreemapUniforms {
            view_width: rect.width(),
            view_height: rect.height(),
            time_sec: self.telemetry.last_frame_time.elapsed().as_secs_f32(),
            _pad: 0.0,
        };

        let callback = egui_wgpu::Callback::new_paint_callback(
            rect,
            GpuTreemapCallback {
                pipeline: self.treemap_pipe.clone(),
                nodes: self.heatmap_nodes.clone(),
                uniforms,
            },
        );
        ui.painter().add(callback);

        // Overlay crisp text labels on top of GPU cells
        for (idx, node) in self.heatmap_nodes.iter().enumerate() {
            let px = rect.left() + node.x * rect.width() + 10.0;
            let py = rect.top() + node.y * rect.height() + 10.0;
            if let Some(t) = self.heatmap_tickers.get(idx) {
                let sign = if node.change_pct >= 0.0 { "+" } else { "" };
                ui.painter().text(
                    egui::pos2(px, py),
                    egui::Align2::LEFT_TOP,
                    format!("{}\n{}{:.2}%", t, sign, node.change_pct),
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
            }
        }
    }
}
