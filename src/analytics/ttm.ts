import { FinancialPeriod } from "../core/financials.ts";

export class TtmAggregator {
  static aggregateQuarters(quarters: readonly FinancialPeriod[]): FinancialPeriod | null {
    if (quarters.length === 0) return null;
    if (quarters.length < 4) {
      const last = quarters[quarters.length - 1]!;
      return new FinancialPeriod(
        last.year,
        last.kind,
        last.dateStart,
        last.dateEnd,
        last.accounts
      );
    }

    const last4 = quarters.slice(-4);
    const earliest = last4[0]!;
    const latest = last4[last4.length - 1]!;

    const aggregatedAccounts: Record<string, number> = {};

    // 1. Balance sheet accounts (codes starting with "1" or "2") from latest quarter
    for (const [code, val] of Object.entries(latest.accounts)) {
      if (code.startsWith("1") || code.startsWith("2") || code === "SHARES") {
        aggregatedAccounts[code] = val;
      }
    }

    // 2. Flow accounts (codes starting with "3", "6", or "7") summed across last 4 quarters
    for (const q of last4) {
      for (const [code, val] of Object.entries(q.accounts)) {
        if (code.startsWith("3") || code.startsWith("6") || code.startsWith("7")) {
          aggregatedAccounts[code] = (aggregatedAccounts[code] ?? 0.0) + val;
        }
      }
    }

    return new FinancialPeriod(
      latest.year,
      "TTM",
      earliest.dateStart,
      latest.dateEnd,
      aggregatedAccounts
    );
  }
}
