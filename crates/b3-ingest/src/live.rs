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
        if let Some(res) = data.results.first() {
            if let Some(price) = res.regular_market_price {
                let high = res.regular_market_day_high.unwrap_or(price);
                let low = res.regular_market_day_low.unwrap_or(price);
                let volume = res.regular_market_volume.unwrap_or(0.0);
                return Ok(Some(Candle {
                    timestamp_ms: Utc::now().timestamp_millis(),
                    open: price,
                    high,
                    low,
                    close: price,
                    volume,
                    trades: 1,
                    adjusted_close: price,
                }));
            }
        }
        Ok(None)
    }

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
