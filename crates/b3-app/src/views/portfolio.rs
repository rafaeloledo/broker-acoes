use eframe::egui::{self, Color32, RichText};

use crate::AppState;

impl AppState {
    pub(crate) fn render_portfolio_tax_view(&mut self, ui: &mut egui::Ui) {
        let positions = self.portfolio_tracker.get_positions(&self.current_prices);

        let total_invested: f64 = positions.iter().map(|p| p.total_invested).sum();
        let total_value: f64 = positions.iter().map(|p| p.current_value).sum();
        let total_pnl = total_value - total_invested;
        let total_pnl_pct = if total_invested > 0.0 { (total_pnl / total_invested) * 100.0 } else { 0.0 };

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Portfolio Summary Banner
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(22, 26, 36))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Patrimônio em Custódia B3").color(Color32::GRAY).size(12.0));
                            ui.heading(RichText::new(format!("R$ {:.2}", total_value)).size(26.0).color(Color32::WHITE));
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            ui.label(RichText::new("Total Aplicado (Preço Médio)").color(Color32::GRAY).size(12.0));
                            ui.label(RichText::new(format!("R$ {:.2}", total_invested)).size(18.0).color(Color32::from_rgb(180, 195, 210)));
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            let col = if total_pnl >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                            ui.label(RichText::new("Lucro / Prejuízo Não Realizado").color(Color32::GRAY).size(12.0));
                            ui.label(RichText::new(format!("R$ {:+.2} ({:+.2}%)", total_pnl, total_pnl_pct)).strong().size(18.0).color(col));
                        });
                    });
                });

            ui.add_space(10.0);

            // Open Positions Table
            ui.heading(RichText::new("Posições em Aberto").size(16.0));
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(24, 28, 38))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    egui::Grid::new("portfolio_positions_grid")
                        .striped(true)
                        .spacing([18.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Ticker").strong());
                            ui.label(RichText::new("Quantidade").strong());
                            ui.label(RichText::new("Preço Médio (PM)").strong());
                            ui.label(RichText::new("Cotação Atual").strong());
                            ui.label(RichText::new("Valor Aplicado").strong());
                            ui.label(RichText::new("Saldo Atual").strong());
                            ui.label(RichText::new("Retorno (R$)").strong());
                            ui.label(RichText::new("Retorno (%)").strong());
                            ui.end_row();

                            for p in &positions {
                                let col = if p.unrealized_pnl >= 0.0 { Color32::from_rgb(0, 220, 140) } else { Color32::from_rgb(255, 80, 90) };
                                ui.label(RichText::new(&p.symbol).strong().color(Color32::WHITE));
                                ui.label(format!("{:.0}", p.quantity));
                                ui.label(format!("R$ {:.2}", p.average_price));
                                ui.label(format!("R$ {:.2}", p.current_price));
                                ui.label(format!("R$ {:.2}", p.total_invested));
                                ui.label(format!("R$ {:.2}", p.current_value));
                                ui.label(RichText::new(format!("R$ {:+.2}", p.unrealized_pnl)).color(col));
                                ui.label(RichText::new(format!("{:+.2}%", p.unrealized_pnl_percent)).color(col));
                                ui.end_row();
                            }
                        });
                });

            ui.add_space(12.0);

            // Monthly Tax & DARF (Code 6015) Engine
            ui.heading(RichText::new("Apuração Mensal de Imposto de Renda (DARF 6015)").color(Color32::from_rgb(0, 210, 160)).size(16.0));
            ui.label(RichText::new("Conforme normas da Receita Federal: Isenção de R$ 20.000/mês para Ações Swing Trade | Compensação de Prejuízos Acumulados").color(Color32::GRAY).size(11.0));

            ui.add_space(6.0);

            for report in &self.monthly_reports {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 30, 42))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(RichText::new(format!("Competência: {:02}/{}", report.month, report.year)).size(16.0).color(Color32::WHITE));
                            ui.separator();
                            let exempt_label = if report.is_acoes_exempt {
                                RichText::new("ISENTO (Vendas <= R$ 20.000)").color(Color32::from_rgb(0, 220, 140))
                            } else {
                                RichText::new("TRIBUTÁVEL (Vendas > R$ 20.000)").color(Color32::from_rgb(255, 180, 80))
                            };
                            ui.label(exempt_label);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let darf_col = if report.total_darf_due > 0.0 { Color32::from_rgb(255, 80, 90) } else { Color32::from_rgb(0, 220, 140) };
                                ui.label(RichText::new(format!("DARF 6015: R$ {:.2}", report.total_darf_due)).strong().size(18.0).color(darf_col));
                            });
                        });

                        ui.separator();

                        ui.columns(4, |cols| {
                            cols[0].label(format!("Total Vendas Ações: R$ {:.2}", report.acoes_swing_sales));
                            cols[0].label(format!("Resultado Ações: R$ {:+.2}", report.net_result_acoes_swing));

                            cols[1].label(format!("Day Trade (20%): R$ {:+.2}", report.net_result_day_trade));
                            cols[1].label(format!("FIIs (20%): R$ {:+.2}", report.net_result_fii));

                            cols[2].label(format!("Compensação Prejuízo: R$ {:.2}", report.loss_offset_acoes_swing));
                            cols[2].label(format!("IRRF Retido na Fonte: R$ {:.2}", report.irrf_deducted));

                            cols[3].label(RichText::new(format!("Vencimento: {}", report.due_date.format("%d/%m/%Y"))).color(Color32::from_rgb(100, 180, 255)));
                            cols[3].label(format!("Prejuízo Carregado: R$ {:.2}", report.carryover_loss_acoes_swing));
                        });
                    });
            }
        });
    }
}
