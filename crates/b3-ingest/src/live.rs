use b3_core::quote::{Candle, Tick};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrapiQuoteResult {
    pub symbol: String,
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    #[serde(rename = "regularMarketPrice")]
    pub regular_market_price: Option<f64>,
    #[serde(rename = "regularMarketDayHigh")]
    pub regular_market_day_high: Option<f64>,
    #[serde(rename = "regularMarketDayLow")]
    pub regular_market_day_low: Option<f64>,
    #[serde(rename = "regularMarketVolume")]
    pub regular_market_volume: Option<f64>,
    #[serde(rename = "regularMarketChangePercent")]
    pub regular_market_change_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrapiResponse {
    pub results: Vec<BrapiQuoteResult>,
}

pub struct LiveFeedClient {
    client: Client,
}

impl Default for LiveFeedClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveFeedClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    pub async fn fetch_brapi_quote(&self, ticker: &str) -> anyhow::Result<Option<Candle>> {
        let url = format!("https://brapi.dev/api/quote/{}", ticker);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Ok(None);
        }

        let data: BrapiResponse = resp.json().await?;
        Ok(Self::candle_from_brapi(data, Utc::now().timestamp_millis()))
    }

    pub fn parse_brapi_quote(body: &str, now_ms: i64) -> anyhow::Result<Option<Candle>> {
        let data: BrapiResponse = serde_json::from_str(body)?;
        Ok(Self::candle_from_brapi(data, now_ms))
    }

    fn candle_from_brapi(data: BrapiResponse, now_ms: i64) -> Option<Candle> {
        let res = data.results.first()?;
        let price = res.regular_market_price?;
        let high = res.regular_market_day_high.unwrap_or(price);
        let low = res.regular_market_day_low.unwrap_or(price);
        let volume = res.regular_market_volume.unwrap_or(0.0);
        Some(Candle {
            timestamp_ms: now_ms,
            open: price,
            high,
            low,
            close: price,
            volume,
            trades: 1,
            adjusted_close: price,
        })
    }

    /// Pregão B3 em dias úteis, 10:00–18:00 no horário de Brasília.
    pub fn is_b3_session(now_ms: i64) -> bool {
        use chrono::{Datelike, TimeZone, Timelike, Utc};
        let Some(utc) = Utc.timestamp_millis_opt(now_ms).single() else {
            return false;
        };
        let brt = utc - chrono::Duration::hours(3);
        let weekday = brt.weekday();
        if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
            return false;
        }
        let minutes = brt.hour() * 60 + brt.minute();
        (10 * 60..18 * 60).contains(&minutes)
    }

    pub const QUOTE_POLL_INTERVAL_MS: i64 = 5 * 60 * 1000;

    /// High-performance tick synthesizer for local hardware acceleration tests
    pub fn simulate_tick(symbol: &str, last_price: f64) -> Tick {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let now = Utc::now().timestamp_millis();
        let mut hasher = DefaultHasher::new();
        now.hash(&mut hasher);
        symbol.hash(&mut hasher);
        let h = hasher.finish();

        let delta = ((h % 200) as f64 - 99.5) / 1000.0; // +/- R$ 0.10
        let new_price = (last_price + delta).max(0.50);
        let qty = ((h % 50) + 1) * 100;
        let is_buyer = (h % 2) == 0;

        Tick {
            timestamp_ms: now,
            price: (new_price * 100.0).round() / 100.0,
            quantity: qty as f64,
            buyer_broker: ((h % 30) + 1) as u16,
            seller_broker: (((h >> 8) % 30) + 1) as u16,
            is_buyer_aggressor: is_buyer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_brapi_quote_without_network() {
        let body = r#"{
            "results": [{
                "symbol": "PETR4",
                "regularMarketPrice": 36.4,
                "regularMarketDayHigh": 36.8,
                "regularMarketDayLow": 36.1,
                "regularMarketVolume": 1000.0
            }]
        }"#;
        let candle = LiveFeedClient::parse_brapi_quote(body, 1_700_000_000_000)
            .unwrap()
            .unwrap();
        assert!((candle.close - 36.4).abs() < 1e-9);
        assert!((candle.high - 36.8).abs() < 1e-9);
        assert_eq!(candle.timestamp_ms, 1_700_000_000_000);
    }

    #[test]
    fn test_b3_session_skips_weekend_and_night() {
        // 2024-06-03 was a Monday. 15:00 UTC = 12:00 BRT, inside the session.
        let monday_noon_brt = 1_717_426_800_000;
        assert!(LiveFeedClient::is_b3_session(monday_noon_brt));
        // 02:00 UTC = 23:00 BRT of the previous day... use 03:00 UTC = 00:00 BRT Monday, outside.
        let monday_midnight_brt = 1_717_383_600_000;
        assert!(!LiveFeedClient::is_b3_session(monday_midnight_brt));
    }
}
