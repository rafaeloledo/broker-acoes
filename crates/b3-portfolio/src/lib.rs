pub mod position;
pub mod tax_calc;

pub use position::*;
pub use tax_calc::*;

#[cfg(test)]
mod tests {
    use super::*;
    use b3_core::tax::{OperationKind, OperationSide, TradeOperation};
    use chrono::NaiveDate;

    #[test]
    fn test_preco_medio_calculation() {
        let mut tracker = PortfolioTracker::new();

        // 1. Buy 100 at R$ 30.00
        tracker.process_trade(&TradeOperation {
            id: "1".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            symbol: "PETR4".to_string(),
            side: OperationSide::Buy,
            kind: OperationKind::SwingTrade,
            quantity: 100.0,
            unit_price: 30.0,
            total_gross: 3000.0,
            liquidation_fee: 0.0,
            registration_fee: 0.0,
            brokerage_fee: 0.0,
            iss_tax: 0.0,
            irrf: 0.0,
            broker_name: "XP".to_string(),
            note_number: "1".to_string(),
        });

        // 2. Buy 100 at R$ 40.00 -> PM should be exactly R$ 35.00
        tracker.process_trade(&TradeOperation {
            id: "2".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            symbol: "PETR4".to_string(),
            side: OperationSide::Buy,
            kind: OperationKind::SwingTrade,
            quantity: 100.0,
            unit_price: 40.0,
            total_gross: 4000.0,
            liquidation_fee: 0.0,
            registration_fee: 0.0,
            brokerage_fee: 0.0,
            iss_tax: 0.0,
            irrf: 0.0,
            broker_name: "XP".to_string(),
            note_number: "2".to_string(),
        });

        let prices = std::collections::HashMap::new();
        let pos = tracker.get_positions(&prices);
        assert_eq!(pos.len(), 1);
        assert_eq!(pos[0].quantity, 200.0);
        assert_eq!(pos[0].average_price, 35.0);

        // 3. Sell 50 at R$ 50.00 -> Realized PnL = 50 * (50 - 35) = 750.0
        let pnl = tracker.process_trade(&TradeOperation {
            id: "3".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            symbol: "PETR4".to_string(),
            side: OperationSide::Sell,
            kind: OperationKind::SwingTrade,
            quantity: 50.0,
            unit_price: 50.0,
            total_gross: 2500.0,
            liquidation_fee: 0.0,
            registration_fee: 0.0,
            brokerage_fee: 0.0,
            iss_tax: 0.0,
            irrf: 0.0,
            broker_name: "XP".to_string(),
            note_number: "3".to_string(),
        });

        assert_eq!(pnl, 750.0);
        let pos = tracker.get_positions(&prices);
        assert_eq!(pos[0].quantity, 150.0);
        assert_eq!(pos[0].average_price, 35.0); // PM remains 35.0 on sells!
    }

    #[test]
    fn test_monthly_tax_exemption_rule() {
        let mut tracker = PortfolioTracker::new();
        let mut calc = TaxCalculator::new();

        // Buy 100 PETR4 at R$ 30.00
        let buy = TradeOperation {
            id: "B1".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            symbol: "PETR4".to_string(),
            side: OperationSide::Buy,
            kind: OperationKind::SwingTrade,
            quantity: 100.0,
            unit_price: 30.0,
            total_gross: 3000.0,
            liquidation_fee: 0.0,
            registration_fee: 0.0,
            brokerage_fee: 0.0,
            iss_tax: 0.0,
            irrf: 0.0,
            broker_name: "XP".to_string(),
            note_number: "1".to_string(),
        };

        // Sell 50 PETR4 at R$ 40.00 -> Sales = R$ 2000.00 (under R$ 20.000,00 exemption limit)
        let sell = TradeOperation {
            id: "S1".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 2, 20).unwrap(),
            symbol: "PETR4".to_string(),
            side: OperationSide::Sell,
            kind: OperationKind::SwingTrade,
            quantity: 50.0,
            unit_price: 40.0,
            total_gross: 2000.0,
            liquidation_fee: 0.0,
            registration_fee: 0.0,
            brokerage_fee: 0.0,
            iss_tax: 0.0,
            irrf: 0.0,
            broker_name: "XP".to_string(),
            note_number: "2".to_string(),
        };

        let trades = vec![buy, sell];
        let reports = calc.compute_monthly_taxes(&trades, &mut tracker);
        assert_eq!(reports.len(), 1);
        let rep = &reports[0];
        assert_eq!(rep.is_acoes_exempt, true);
        assert_eq!(rep.total_darf_due, 0.0); // Exempt!
    }
}
