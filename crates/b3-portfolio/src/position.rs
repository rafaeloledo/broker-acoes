use b3_core::tax::{OperationSide, PortfolioPosition, TradeOperation};
use std::collections::HashMap;

pub struct PortfolioTracker {
    positions: HashMap<String, (f64, f64)>, // symbol -> (quantity, average_price)
}

impl Default for PortfolioTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PortfolioTracker {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    pub fn process_trade(&mut self, trade: &TradeOperation) -> f64 {
        let (qty, pm) = self.positions.entry(trade.symbol.clone()).or_insert((0.0, 0.0));
        let mut realized_pnl = 0.0;

        match trade.side {
            OperationSide::Buy => {
                let cost = trade.effective_total();
                let current_invested = *qty * *pm;
                let new_qty = *qty + trade.quantity;
                if new_qty > 0.0 {
                    *pm = (current_invested + cost) / new_qty;
                    *qty = new_qty;
                }
            }
            OperationSide::Sell => {
                let sell_received = trade.effective_total();
                let cost_basis = trade.quantity * *pm;
                realized_pnl = sell_received - cost_basis;

                *qty = (*qty - trade.quantity).max(0.0);
                if *qty == 0.0 {
                    *pm = 0.0;
                }
            }
        }

        realized_pnl
    }

    pub fn get_positions(&self, current_prices: &HashMap<String, f64>) -> Vec<PortfolioPosition> {
        let mut result = Vec::new();

        for (symbol, &(qty, pm)) in &self.positions {
            if qty <= 0.0001 {
                continue;
            }
            let current_price = current_prices.get(symbol).copied().unwrap_or(pm);
            let total_invested = qty * pm;
            let current_value = qty * current_price;
            let pnl = current_value - total_invested;
            let pnl_pct = if total_invested > 0.0 { (pnl / total_invested) * 100.0 } else { 0.0 };

            result.push(PortfolioPosition {
                symbol: symbol.clone(),
                quantity: qty,
                average_price: pm,
                total_invested,
                current_price,
                current_value,
                unrealized_pnl: pnl,
                unrealized_pnl_percent: pnl_pct,
            });
        }

        result.sort_by(|a, b| b.current_value.partial_cmp(&a.current_value).unwrap());
        result
    }
}
