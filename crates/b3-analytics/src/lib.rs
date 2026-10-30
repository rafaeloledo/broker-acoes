pub mod core_multiples;
pub mod screener;
pub mod ttm;
pub mod valuation;

pub use core_multiples::*;
pub use screener::*;
pub use ttm::*;
pub use valuation::*;

#[cfg(test)]
mod tests {
    use super::*;
    use b3_core::indicators::StockIndicators;

    #[test]
    fn test_graham_and_bazin_formulas() {
        // LPA 5, VPA 30, DPA 5, preço 20. Não usa o cenário fixo de default_for.
        let multiples = core_multiples(20.0, Some(5.0), Some(30.0), Some(5.0), Some(1.0));
        let graham = multiples.graham_fair_value.expect("graham");
        let bazin = multiples.bazin_ceiling_price.expect("bazin");
        assert!((graham - (22.5 * 5.0 * 30.0_f64).sqrt()).abs() < 1e-9);
        assert!(graham > 50.0);
        assert!((bazin - (5.0 / 0.06)).abs() < 1e-9);
        assert!(bazin > 80.0);
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
