import { type Candle, type Tick, BRT_OFFSET_MS, brtHourMinute } from "../core/quote.ts";

export const QUOTE_POLL_INTERVAL_MS = 5 * 60 * 1000;

export function isB3Session(nowMs: number): boolean {
  const brtDate = new Date(nowMs - BRT_OFFSET_MS);
  const dow = brtDate.getUTCDay(); // 0 = Sun, 1..5 = Mon..Fri, 6 = Sat
  if (dow === 0 || dow === 6) return false;
  const [hour, minute] = brtHourMinute(nowMs);
  const totalMinutes = hour * 60 + minute;
  return totalMinutes >= 600 && totalMinutes < 1080; // 10:00..18:00 BRT
}

interface BrapiQuoteResult {
  symbol?: string;
  regularMarketPrice?: number;
  regularMarketDayHigh?: number;
  regularMarketDayLow?: number;
  regularMarketVolume?: number;
  regularMarketPreviousClose?: number;
  regularMarketChangePercent?: number;
  regularMarketTime?: string;
}

interface BrapiResponse {
  results?: BrapiQuoteResult[];
}

interface YahooChartResponse {
  chart?: {
    result?: Array<{
      meta?: {
        regularMarketPrice?: number;
        regularMarketDayHigh?: number;
        regularMarketDayLow?: number;
        regularMarketVolume?: number;
        chartPreviousClose?: number;
        previousClose?: number;
        regularMarketTime?: number;
      };
    }>;
  };
}

export class LiveFeedClient {
  private readonly userAgent = "Mozilla/5.0 (X11; Linux x86_64)";

  async fetchBrapiQuote(ticker: string): Promise<Candle> {
    const sym = ticker.trim().toUpperCase();
    const token = process.env.BRAPI_TOKEN?.trim();
    const url = token
      ? `https://brapi.dev/api/quote/${encodeURIComponent(sym)}?token=${encodeURIComponent(token)}`
      : `https://brapi.dev/api/quote/${encodeURIComponent(sym)}`;

    try {
      const res = await fetch(url, {
        headers: { "User-Agent": this.userAgent },
        signal: AbortSignal.timeout(8000),
      });
      if (res.ok) {
        const body = (await res.json()) as BrapiResponse;
        const q = body.results?.[0];
        if (q && q.regularMarketPrice != null && q.regularMarketPrice > 0) {
          const price = Number(q.regularMarketPrice);
          let prevClose = price;
          if (
            q.regularMarketPreviousClose != null &&
            q.regularMarketPreviousClose > 0
          ) {
            prevClose = Number(q.regularMarketPreviousClose);
          } else if (q.regularMarketChangePercent != null) {
            const chg = Number(q.regularMarketChangePercent);
            if (Math.abs(1 + chg / 100) > 1e-6) {
              prevClose =
                Math.round((price / (1 + chg / 100)) * 100) / 100;
            }
          }

          const high =
            q.regularMarketDayHigh != null && q.regularMarketDayHigh > 0
              ? Number(q.regularMarketDayHigh)
              : Math.max(price, prevClose);
          const low =
            q.regularMarketDayLow != null && q.regularMarketDayLow > 0
              ? Number(q.regularMarketDayLow)
              : Math.min(price, prevClose);
          const volume =
            q.regularMarketVolume != null ? Number(q.regularMarketVolume) : 0;
          const tsMs = q.regularMarketTime
            ? Date.parse(q.regularMarketTime) || Date.now()
            : Date.now();

          return {
            timestampMs: tsMs,
            open: prevClose,
            high,
            low,
            close: price,
            volume,
            trades: 1,
            adjustedClose: price,
          };
        }
      }
    } catch {
      // Fallback to Yahoo Finance
    }

    return this.fetchYahooQuote(sym);
  }

  async fetchYahooQuote(ticker: string): Promise<Candle> {
    const sym = ticker.trim().toUpperCase();
    const yahooSym = sym.endsWith(".SA") ? sym : `${sym}.SA`;
    const url = `https://query1.finance.yahoo.com/v8/finance/chart/${encodeURIComponent(
      yahooSym
    )}`;

    const res = await fetch(url, {
      headers: { "User-Agent": this.userAgent },
      signal: AbortSignal.timeout(8000),
    });

    if (!res.ok) {
      throw new Error(`Yahoo quote failed for ${sym} with HTTP ${res.status}`);
    }

    const body = (await res.json()) as YahooChartResponse;
    const meta = body.chart?.result?.[0]?.meta;
    if (!meta || meta.regularMarketPrice == null || meta.regularMarketPrice <= 0) {
      throw new Error(`No regularMarketPrice in Yahoo response for ${sym}`);
    }

    const price = Number(meta.regularMarketPrice);
    const prevClose =
      meta.chartPreviousClose != null && meta.chartPreviousClose > 0
        ? Number(meta.chartPreviousClose)
        : meta.previousClose != null && meta.previousClose > 0
        ? Number(meta.previousClose)
        : price;

    const high =
      meta.regularMarketDayHigh != null && meta.regularMarketDayHigh > 0
        ? Number(meta.regularMarketDayHigh)
        : Math.max(price, prevClose);
    const low =
      meta.regularMarketDayLow != null && meta.regularMarketDayLow > 0
        ? Number(meta.regularMarketDayLow)
        : Math.min(price, prevClose);
    const volume =
      meta.regularMarketVolume != null ? Number(meta.regularMarketVolume) : 0;
    const tsMs =
      meta.regularMarketTime != null && meta.regularMarketTime > 0
        ? Number(meta.regularMarketTime) * 1000
        : Date.now();

    return {
      timestampMs: tsMs,
      open: prevClose,
      high,
      low,
      close: price,
      volume,
      trades: 1,
      adjustedClose: price,
    };
  }

  static simulateTick(symbol: string, lastPrice: number, nowMs = Date.now()): Tick {
    const str = `${nowMs}:${symbol}`;
    let h = 2166136261;
    for (let i = 0; i < str.length; i++) {
      h ^= str.charCodeAt(i);
      h = Math.imul(h, 16777619) >>> 0;
    }

    const delta = ((h % 200) - 99.5) / 1000.0;
    const newPrice = Math.max(0.5, Math.round((lastPrice + delta) * 100) / 100);
    const quantity = ((h % 50) + 1) * 100;
    const buyerBroker = (h % 30) + 1;
    const sellerBroker = ((h >>> 8) % 30) + 1;
    const isBuyerAggressor = h % 2 === 0;

    return {
      timestampMs: nowMs,
      price: newPrice,
      quantity,
      buyerBroker,
      sellerBroker,
      isBuyerAggressor,
    };
  }
}
