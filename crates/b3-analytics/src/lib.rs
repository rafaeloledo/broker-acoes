pub mod screener;
pub mod ttm;
pub mod valuation;

pub use screener::*;
pub use ttm::*;
pub use valuation::*;

#[cfg(test)]
mod tests {
    use super::*;
    use b3_core::indicators::StockIndicators;

    #[test]
    fn test_graham_and_bazin_formulas() {
        let petr4 = StockIndicators::default_for("PETR4");
        // Graham fair value: sqrt(22.5 * LPA * VPA)
        assert!(petr4.graham_fair_value > 50.0);
        // Bazin ceiling price: DPA / 0.06
        assert!(petr4.bazin_ceiling_price > 80.0);
    }

    #[test]
    fn test_screener_filtering() {
        let items = vec![
            StockIndicators::default_for("PETR4"),
            StockIndicators::default_for("VALE3"),
            StockIndicators::default_for("WEGE3"),
            StockIndicators::default_for("ITUB4"),
        ];

        let criteria = ScreenerCriteria {
            min_dy: Some(5.0),
            max_pl: Some(10.0),
            ..Default::default()
        };

        let res = ScreenerEngine::filter(&items, &criteria);
        // PETR4 (DY ~14.8%, P/L ~4.85), VALE3 (DY ~8.9%, P/L ~5.6), ITUB4 (DY ~7.2%, P/L ~8.9)
        assert!(!res.is_empty());
        for item in res {
            assert!(item.dividend_yield >= 5.0);
            assert!(item.pl <= 10.0);
        }
    }
}
