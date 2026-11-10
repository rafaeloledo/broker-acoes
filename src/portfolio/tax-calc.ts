import { assetClassFromSymbol } from "../core/b3-assets.ts";
import {
  type MonthlyTaxReport,
  type TaxCategory,
  type TradeOperation,
  dueDateFor,
  tradeEffectiveTotal,
} from "../core/tax.ts";
import { PortfolioTracker } from "./position.ts";

export class TaxCalculator {
  private accumulatedLosses = new Map<TaxCategory, number>([
    ["AcoesSwing", 0.0],
    ["DayTrade", 0.0],
    ["Fii", 0.0],
    ["EtfBdr", 0.0],
  ]);

  computeMonthlyTaxes(
    trades: readonly TradeOperation[],
    portfolio: PortfolioTracker
  ): MonthlyTaxReport[] {
    // Group trades by YYYY-MM in chronological order
    const byMonth = new Map<string, TradeOperation[]>();
    const sorted = trades.slice().sort((a, b) => {
      if (a.date !== b.date) return a.date.localeCompare(b.date);
      return a.id.localeCompare(b.id);
    });

    for (const t of sorted) {
      const ym = t.date.slice(0, 7); // YYYY-MM
      const list = byMonth.get(ym) ?? [];
      list.push(t);
      byMonth.set(ym, list);
    }

    const monthKeys = Array.from(byMonth.keys()).sort();
    const reports: MonthlyTaxReport[] = [];

    for (const ym of monthKeys) {
      const parts = ym.split("-");
      const year = Number(parts[0]) || 2026;
      const month = Number(parts[1]) || 1;
      const monthTrades = byMonth.get(ym) ?? [];

      let acoesSwingSales = 0.0;
      let profitAcoesSwing = 0.0;
      let profitDayTrade = 0.0;
      let profitFii = 0.0;
      let profitEtfBdr = 0.0;
      let totalIrrf = 0.0;

      for (const t of monthTrades) {
        const realized = portfolio.processTrade(t);
        const assetClass = assetClassFromSymbol(t.symbol);

        if (t.side === "Sell") {
          totalIrrf += t.irrf;
          if (t.kind === "DayTrade") {
            profitDayTrade += realized;
          } else {
            if (assetClass === "Acao") {
              acoesSwingSales += tradeEffectiveTotal(t);
              profitAcoesSwing += realized;
            } else if (assetClass === "Fii") {
              profitFii += realized;
            } else if (assetClass === "Etf" || assetClass === "Bdr") {
              profitEtfBdr += realized;
            } else {
              acoesSwingSales += tradeEffectiveTotal(t);
              profitAcoesSwing += realized;
            }
          }
        }
      }

      const isAcoesExempt = acoesSwingSales <= 20_000.0;

      const applyCategory = (
        cat: TaxCategory,
        netResult: number,
        exempt: boolean
      ): { offset: number; taxBase: number; carryover: number } => {
        let accLoss = this.accumulatedLosses.get(cat) ?? 0.0;
        if (netResult > 0.0) {
          if (exempt) {
            return { offset: 0.0, taxBase: 0.0, carryover: accLoss };
          }
          const offset = Math.min(accLoss, netResult);
          accLoss -= offset;
          this.accumulatedLosses.set(cat, accLoss);
          return {
            offset,
            taxBase: netResult - offset,
            carryover: accLoss,
          };
        } else {
          accLoss += Math.abs(netResult);
          this.accumulatedLosses.set(cat, accLoss);
          return { offset: 0.0, taxBase: 0.0, carryover: accLoss };
        }
      };

      const acoesRes = applyCategory("AcoesSwing", profitAcoesSwing, isAcoesExempt);
      const dtRes = applyCategory("DayTrade", profitDayTrade, false);
      const fiiRes = applyCategory("Fii", profitFii, false);
      const etfRes = applyCategory("EtfBdr", profitEtfBdr, false);

      const taxDueAcoesSwing = acoesRes.taxBase * 0.15;
      const taxDueDayTrade = dtRes.taxBase * 0.20;
      const taxDueFii = fiiRes.taxBase * 0.20;
      const taxDueEtfBdr = etfRes.taxBase * 0.15;

      const grossTax =
        taxDueAcoesSwing + taxDueDayTrade + taxDueFii + taxDueEtfBdr;
      const netDarf = Math.max(grossTax - totalIrrf, 0.0);
      const finalDarf = netDarf >= 10.0 ? netDarf : 0.0;

      reports.push({
        year,
        month,
        acoesSwingSales,
        isAcoesExempt,
        netResultAcoesSwing: profitAcoesSwing,
        netResultDayTrade: profitDayTrade,
        netResultFii: profitFii,
        netResultEtfBdr: profitEtfBdr,
        lossOffsetAcoesSwing: acoesRes.offset,
        lossOffsetDayTrade: dtRes.offset,
        lossOffsetFii: fiiRes.offset,
        lossOffsetEtfBdr: etfRes.offset,
        carryoverLossAcoesSwing: acoesRes.carryover,
        carryoverLossDayTrade: dtRes.carryover,
        carryoverLossFii: fiiRes.carryover,
        carryoverLossEtfBdr: etfRes.carryover,
        taxBaseAcoesSwing: acoesRes.taxBase,
        taxBaseDayTrade: dtRes.taxBase,
        taxBaseFii: fiiRes.taxBase,
        taxBaseEtfBdr: etfRes.taxBase,
        taxDueAcoesSwing,
        taxDueDayTrade,
        taxDueFii,
        taxDueEtfBdr,
        irrfDeducted: totalIrrf,
        darfCode: "6015",
        totalDarfDue: finalDarf,
        dueDate: dueDateFor(year, month),
      });
    }

    return reports;
  }
}
