import { type Candle } from "../core/quote.ts";

const VALID_BDI = new Set(["02", "12", "14", "96"]);

export class CotahistParser {
  static parseLine(line: string): [string, Candle] | null {
    if (line.length < 240 || !line.startsWith("01")) {
      return null;
    }

    const dateRaw = line.slice(2, 10);
    if (!/^\d{8}$/.test(dateRaw)) return null;

    const bdi = line.slice(10, 12);
    if (!VALID_BDI.has(bdi)) return null;

    const codneg = line.slice(12, 24).trim().toUpperCase();
    if (!codneg) return null;

    const y = dateRaw.slice(0, 4);
    const m = dateRaw.slice(4, 6);
    const d = dateRaw.slice(6, 8);
    const timestampMs = Date.parse(`${y}-${m}-${d}T18:00:00Z`);
    if (Number.isNaN(timestampMs)) return null;

    const open = Number.parseInt(line.slice(56, 69), 10) / 100.0;
    const high = Number.parseInt(line.slice(69, 82), 10) / 100.0;
    const low = Number.parseInt(line.slice(82, 95), 10) / 100.0;
    const close = Number.parseInt(line.slice(108, 121), 10) / 100.0;
    const trades = Number.parseInt(line.slice(152, 170), 10) || 0;
    const volume = Number.parseInt(line.slice(170, 188), 10) / 100.0;

    if (
      Number.isNaN(open) ||
      Number.isNaN(high) ||
      Number.isNaN(low) ||
      Number.isNaN(close)
    ) {
      return null;
    }

    return [
      codneg,
      {
        timestampMs,
        open,
        high,
        low,
        close,
        volume: Number.isNaN(volume) ? 0.0 : volume,
        trades,
        adjustedClose: close,
      },
    ];
  }

  static parseCotahistContent(content: string): [string, Candle][] {
    const out: [string, Candle][] = [];
    const lines = content.split(/\r?\n/);
    for (const line of lines) {
      const parsed = CotahistParser.parseLine(line);
      if (parsed) out.push(parsed);
    }
    return out;
  }
}
