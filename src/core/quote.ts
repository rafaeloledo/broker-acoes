export interface GpuCandle {
  timestampSec: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  isUp: number;
  padding: number;
}

export interface Candle {
  timestampMs: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  trades: number;
  adjustedClose: number;
}

export function candleToGpuCandle(candle: Candle, baseTimestampSec: number): GpuCandle {
  return {
    timestampSec: candle.timestampMs / 1000.0 - baseTimestampSec,
    open: candle.open,
    high: candle.high,
    low: candle.low,
    close: candle.close,
    volume: candle.volume,
    isUp: candle.close >= candle.open ? 1.0 : 0.0,
    padding: 0.0,
  };
}

export const BRT_OFFSET_MS = 3 * 3_600_000;
const DAY_MS = 86_400_000;

function divEuclid(a: number, b: number): number {
  return Math.floor(a / b);
}

function remEuclid(a: number, b: number): number {
  return ((a % b) + b) % b;
}

export function brtDayStartMs(timestampMs: number): number {
  return divEuclid(timestampMs - BRT_OFFSET_MS, DAY_MS) * DAY_MS + BRT_OFFSET_MS;
}

export function brtHourMinute(timestampMs: number): [number, number] {
  const dayMs = remEuclid(timestampMs - BRT_OFFSET_MS, DAY_MS);
  const totalMinutes = Math.floor(dayMs / 60_000);
  const hour = Math.floor(totalMinutes / 60);
  const minute = totalMinutes % 60;
  return [hour, minute];
}

export interface Tick {
  timestampMs: number;
  price: number;
  quantity: number;
  buyerBroker: number;
  sellerBroker: number;
  isBuyerAggressor: boolean;
}

export interface OrderBookLevel {
  price: number;
  quantity: number;
  ordersCount: number;
}

export interface OrderBook {
  symbol: string;
  timestampMs: number;
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
}

export function orderBookSpread(book: OrderBook): number | null {
  const bestBid = book.bids[0];
  const bestAsk = book.asks[0];
  if (!bestBid || !bestAsk) return null;
  return bestAsk.price - bestBid.price;
}

export function orderBookMidPrice(book: OrderBook): number | null {
  const bestBid = book.bids[0];
  const bestAsk = book.asks[0];
  if (!bestBid || !bestAsk) return null;
  return (bestBid.price + bestAsk.price) / 2.0;
}

export function createSyntheticOrderBook(symbol: string, midPrice: number): OrderBook {
  const bids: OrderBookLevel[] = [];
  const asks: OrderBookLevel[] = [];
  for (let i = 0; i < 8; i++) {
    const step = (i + 1) * 0.01;
    bids.push({
      price: Math.max(0.01, Number((midPrice - step).toFixed(2))),
      quantity: (i + 2) * 300,
      ordersCount: (i % 5) + 2,
    });
    asks.push({
      price: Number((midPrice + step).toFixed(2)),
      quantity: (i + 2) * 250,
      ordersCount: ((i + 1) % 5) + 2,
    });
  }
  return {
    symbol,
    timestampMs: Date.now(),
    bids,
    asks,
  };
}

export function createSyntheticCandles(basePrice: number, count = 180): Candle[] {
  const candles: Candle[] = [];
  const now = Date.now();
  let price = Math.max(basePrice * 0.88, 0.5);

  for (let i = 0; i < count; i++) {
    const t = i * 0.14;
    const wave = Math.sin(t) * 0.014 + Math.cos(t * 0.6) * 0.009;
    const open = Number(price.toFixed(2));
    const close = Number(Math.max(0.5, open * (1.0 + wave)).toFixed(2));
    const spread = Math.max(0.02, Math.abs(close - open) * 0.6 + basePrice * 0.008);
    const high = Number((Math.max(open, close) + spread).toFixed(2));
    const low = Number(Math.max(0.1, Math.min(open, close) - spread).toFixed(2));
    const volume = Math.round((1_200_000 + (i % 17) * 180_000) * (1 + Math.abs(wave) * 10));

    candles.push({
      timestampMs: now - (count - 1 - i) * 86_400_000,
      open,
      high,
      low,
      close,
      volume,
      trades: 1500 + (i % 20) * 120,
      adjustedClose: close,
    });
    price = close;
  }

  if (candles.length > 0) {
    const last = candles[candles.length - 1]!;
    const prev = candles.length > 1 ? candles[candles.length - 2]!.close : basePrice * 0.99;
    last.open = Number(prev.toFixed(2));
    last.close = Number(basePrice.toFixed(2));
    last.adjustedClose = last.close;
    last.high = Number(Math.max(last.high, last.open, last.close).toFixed(2));
    last.low = Number(Math.min(last.low, last.open, last.close).toFixed(2));
  }

  return candles;
}
