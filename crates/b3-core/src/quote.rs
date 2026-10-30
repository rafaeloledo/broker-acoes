use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct GpuCandle {
    pub timestamp_sec: f32,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
    pub is_up: f32, // 1.0 = green/up, 0.0 = red/down
    pub _padding: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp_ms: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trades: u32,
    pub adjusted_close: f64,
}

/// Início do dia civil em Brasília (UTC−3, sem horário de verão), em milissegundos Unix.
pub fn brt_day_start_ms(timestamp_ms: i64) -> i64 {
    const BRT_OFFSET_MS: i64 = 3 * 3_600_000;
    let brt_ms = timestamp_ms - BRT_OFFSET_MS;
    let day = brt_ms.div_euclid(86_400_000);
    day * 86_400_000 + BRT_OFFSET_MS
}

/// Hora e minuto em Brasília a partir de um instante Unix em milissegundos.
pub fn brt_hour_minute(timestamp_ms: i64) -> (u32, u32) {
    const BRT_OFFSET_MS: i64 = 3 * 3_600_000;
    let brt_ms = timestamp_ms - BRT_OFFSET_MS;
    let seconds = brt_ms.div_euclid(1000).rem_euclid(86_400);
    let hour = (seconds / 3_600) as u32;
    let minute = ((seconds % 3_600) / 60) as u32;
    (hour, minute)
}

impl Candle {
    pub fn to_gpu_candle(&self, base_timestamp_sec: f32) -> GpuCandle {
        let ts_sec = (self.timestamp_ms as f64 / 1000.0) as f32 - base_timestamp_sec;
        let is_up = if self.close >= self.open { 1.0 } else { 0.0 };
        GpuCandle {
            timestamp_sec: ts_sec,
            open: self.open as f32,
            high: self.high as f32,
            low: self.low as f32,
            close: self.close as f32,
            volume: self.volume as f32,
            is_up,
            _padding: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick {
    pub timestamp_ms: i64,
    pub price: f64,
    pub quantity: f64,
    pub buyer_broker: u16,
    pub seller_broker: u16,
    pub is_buyer_aggressor: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub quantity: f64,
    pub orders_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub timestamp_ms: i64,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

impl OrderBook {
    pub fn spread(&self) -> Option<f64> {
        let best_bid = self.bids.first()?.price;
        let best_ask = self.asks.first()?.price;
        Some(best_ask - best_bid)
    }

    pub fn mid_price(&self) -> Option<f64> {
        let best_bid = self.bids.first()?.price;
        let best_ask = self.asks.first()?.price;
        Some((best_bid + best_ask) / 2.0)
    }
}
