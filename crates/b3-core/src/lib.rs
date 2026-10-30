pub mod b3_assets;
pub mod corporate_actions;
pub mod financials;
pub mod fii;
pub mod indicators;
pub mod news;
pub mod quote;
pub mod tax;
pub mod ticker;

pub use b3_assets::*;
pub use corporate_actions::*;
pub use financials::*;
pub use fii::*;
pub use indicators::*;
pub use news::{news_for_reader, NewsBrief, NewsItem, NEWS_READ_LIMIT, NEWS_READ_MAX_CHARS};
pub use quote::*;
pub use tax::*;
pub use ticker::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_class_classification() {
        assert_eq!(AssetClass::from_symbol("PETR4"), AssetClass::Acao);
        assert_eq!(AssetClass::from_symbol("VALE3"), AssetClass::Acao);
        assert_eq!(AssetClass::from_symbol("HGLG11"), AssetClass::Fii);
        assert_eq!(AssetClass::from_symbol("KNCR11"), AssetClass::Fii);
        assert_eq!(AssetClass::from_symbol("BOVA11"), AssetClass::Etf);
        assert_eq!(AssetClass::from_symbol("AAPL34"), AssetClass::Bdr);
        assert_eq!(AssetClass::from_symbol("IBOV"), AssetClass::Indice);
    }

    #[test]
    fn test_gpu_candle_conversion() {
        let candle = Candle {
            timestamp_ms: 1_700_000_000_000,
            open: 35.0,
            high: 36.5,
            low: 34.8,
            close: 36.2,
            volume: 10_000_000.0,
            trades: 500,
            adjusted_close: 36.2,
        };
        let gpu_c = candle.to_gpu_candle(0.0);
        assert_eq!(gpu_c.open, 35.0);
        assert_eq!(gpu_c.high, 36.5);
        assert_eq!(gpu_c.low, 34.8);
        assert_eq!(gpu_c.close, 36.2);
        assert_eq!(gpu_c.is_up, 1.0);
    }

    #[test]
    fn test_order_book_spread() {
        let ob = OrderBook {
            symbol: "PETR4".to_string(),
            timestamp_ms: 0,
            bids: vec![OrderBookLevel { price: 36.40, quantity: 1000.0, orders_count: 5 }],
            asks: vec![OrderBookLevel { price: 36.42, quantity: 1200.0, orders_count: 4 }],
        };
        assert!((ob.spread().unwrap() - 0.02).abs() < 1e-6);
        assert!((ob.mid_price().unwrap() - 36.41).abs() < 1e-6);
    }
}

