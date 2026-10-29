use b3_core::tax::{MonthlyTaxReport, OperationKind, OperationSide, TaxCategory, TradeOperation};
use b3_core::ticker::AssetClass;
use chrono::Datelike;
use std::collections::{BTreeMap, HashMap};

pub struct TaxCalculator {
    accumulated_losses: HashMap<TaxCategory, f64>,
}

impl Default for TaxCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl TaxCalculator {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(TaxCategory::AcoesSwing, 0.0);
        map.insert(TaxCategory::DayTrade, 0.0);
        map.insert(TaxCategory::Fii, 0.0);
        map.insert(TaxCategory::EtfBdr, 0.0);
        Self { accumulated_losses: map }
    }

    pub fn compute_monthly_taxes(
        &mut self,
        trades: &[TradeOperation],
        portfolio: &mut crate::position::PortfolioTracker,
    ) -> Vec<MonthlyTaxReport> {
        // Group trades by (Year, Month)
        let mut monthly_trades: BTreeMap<(i32, u32), Vec<&TradeOperation>> = BTreeMap::new();
        for t in trades {
            let key = (t.date.year(), t.date.month());
            monthly_trades.entry(key).or_default().push(t);
        }

        let mut reports = Vec::new();

        for ((year, month), list) in monthly_trades {
            let mut acoes_swing_sales = 0.0;
            let mut profit_acoes_swing = 0.0;
            let mut profit_day_trade = 0.0;
            let mut profit_fii = 0.0;
            let mut profit_etf_bdr = 0.0;
            let mut total_irrf = 0.0;

            for t in list {
                let realized = portfolio.process_trade(t);
                let asset_class = AssetClass::from_symbol(&t.symbol);

                if t.side == OperationSide::Sell {
                    total_irrf += t.irrf;

                    if t.kind == OperationKind::DayTrade {
                        profit_day_trade += realized;
                    } else {
                        match asset_class {
                            AssetClass::Acao => {
                                acoes_swing_sales += t.effective_total();
                                profit_acoes_swing += realized;
                            }
                            AssetClass::Fii => {
                                profit_fii += realized;
                            }
                            AssetClass::Etf | AssetClass::Bdr => {
                                profit_etf_bdr += realized;
                            }
                            _ => {
                                profit_acoes_swing += realized;
                            }
                        }
                    }
                }
            }

            // Exemption check for Ações Swing Trade: R$ 20.000,00
            let is_acoes_exempt = acoes_swing_sales <= 20_000.0;

            // Loss offset logic
            let (tax_base_acoes, loss_offset_acoes) = if is_acoes_exempt && profit_acoes_swing > 0.0 {
                // If exempt, profit is 100% tax-free!
                (0.0, 0.0)
            } else if profit_acoes_swing > 0.0 {
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::AcoesSwing).unwrap();
                let offset = (*prev_loss).min(profit_acoes_swing);
                *prev_loss -= offset;
                (profit_acoes_swing - offset, offset)
            } else {
                // Negative: accumulate loss
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::AcoesSwing).unwrap();
                *prev_loss += profit_acoes_swing.abs();
                (0.0, 0.0)
            };

            // Day Trade (20%)
            let (tax_base_dt, loss_offset_dt) = if profit_day_trade > 0.0 {
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::DayTrade).unwrap();
                let offset = (*prev_loss).min(profit_day_trade);
                *prev_loss -= offset;
                (profit_day_trade - offset, offset)
            } else {
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::DayTrade).unwrap();
                *prev_loss += profit_day_trade.abs();
                (0.0, 0.0)
            };

            // FII (20%)
            let (tax_base_fii, loss_offset_fii) = if profit_fii > 0.0 {
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::Fii).unwrap();
                let offset = (*prev_loss).min(profit_fii);
                *prev_loss -= offset;
                (profit_fii - offset, offset)
            } else {
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::Fii).unwrap();
                *prev_loss += profit_fii.abs();
                (0.0, 0.0)
            };

            // ETF / BDR (15%)
            let (tax_base_etf, loss_offset_etf) = if profit_etf_bdr > 0.0 {
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::EtfBdr).unwrap();
                let offset = (*prev_loss).min(profit_etf_bdr);
                *prev_loss -= offset;
                (profit_etf_bdr - offset, offset)
            } else {
                let prev_loss = self.accumulated_losses.get_mut(&TaxCategory::EtfBdr).unwrap();
                *prev_loss += profit_etf_bdr.abs();
                (0.0, 0.0)
            };

            let tax_due_acoes = tax_base_acoes * 0.15;
            let tax_due_dt = tax_base_dt * 0.20;
            let tax_due_fii = tax_base_fii * 0.20;
            let tax_due_etf = tax_base_etf * 0.15;

            let gross_tax = tax_due_acoes + tax_due_dt + tax_due_fii + tax_due_etf;
            let net_darf = (gross_tax - total_irrf).max(0.0);
            let final_darf = if net_darf >= 10.0 { net_darf } else { 0.0 };

            reports.push(MonthlyTaxReport {
                year,
                month,
                acoes_swing_sales,
                is_acoes_exempt,
                net_result_acoes_swing: profit_acoes_swing,
                net_result_day_trade: profit_day_trade,
                net_result_fii: profit_fii,
                net_result_etf_bdr: profit_etf_bdr,
                loss_offset_acoes_swing: loss_offset_acoes,
                loss_offset_day_trade: loss_offset_dt,
                loss_offset_fii: loss_offset_fii,
                loss_offset_etf_bdr: loss_offset_etf,
                carryover_loss_acoes_swing: *self.accumulated_losses.get(&TaxCategory::AcoesSwing).unwrap(),
                carryover_loss_day_trade: *self.accumulated_losses.get(&TaxCategory::DayTrade).unwrap(),
                carryover_loss_fii: *self.accumulated_losses.get(&TaxCategory::Fii).unwrap(),
                carryover_loss_etf_bdr: *self.accumulated_losses.get(&TaxCategory::EtfBdr).unwrap(),
                tax_base_acoes_swing: tax_base_acoes,
                tax_base_day_trade: tax_base_dt,
                tax_base_fii: tax_base_fii,
                tax_base_etf_bdr: tax_base_etf,
                tax_due_acoes_swing: tax_due_acoes,
                tax_due_day_trade: tax_due_dt,
                tax_due_fii: tax_due_fii,
                tax_due_etf_bdr: tax_due_etf,
                irrf_deducted: total_irrf,
                darf_code: "6015",
                total_darf_due: final_darf,
                due_date: MonthlyTaxReport::due_date_for(year, month),
            });
        }

        reports
    }
}
