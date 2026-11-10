import { type StockIndicators } from "../core/indicators.ts";

export interface ScreenerCriteria {
  minDy?: number | null;
  maxPl?: number | null;
  minPl?: number | null;
  maxPvp?: number | null;
  minRoe?: number | null;
  minRoic?: number | null;
  minMargemLiquida?: number | null;
  maxDividaLiqEbitda?: number | null;
  minLiquidezCorrente?: number | null;
}

export class ScreenerEngine {
  static filter(
    items: readonly StockIndicators[],
    criteria: ScreenerCriteria
  ): StockIndicators[] {
    return items.filter((i) => {
      if (criteria.minDy != null && i.dividendYield < criteria.minDy) return false;
      if (criteria.maxPl != null && (i.pl <= 0.0 || i.pl > criteria.maxPl))
        return false;
      if (criteria.minPl != null && i.pl < criteria.minPl) return false;
      if (criteria.maxPvp != null && (i.pvp <= 0.0 || i.pvp > criteria.maxPvp))
        return false;
      if (criteria.minRoe != null && i.roe < criteria.minRoe) return false;
      if (criteria.minRoic != null && i.roic < criteria.minRoic) return false;
      if (
        criteria.minMargemLiquida != null &&
        i.margemLiquida < criteria.minMargemLiquida
      )
        return false;
      if (
        criteria.maxDividaLiqEbitda != null &&
        i.dividaLiqEbitda > criteria.maxDividaLiqEbitda
      )
        return false;
      if (
        criteria.minLiquidezCorrente != null &&
        i.liquidezCorrente < criteria.minLiquidezCorrente
      )
        return false;
      return true;
    });
  }

  static magicFormula(items: readonly StockIndicators[]): StockIndicators[] {
    const valid = items.filter((i) => i.evEbit > 0.0 && i.roic > 0.0);

    const byEvEbit = valid.slice().sort((a, b) => a.evEbit - b.evEbit);
    const byRoic = valid.slice().sort((a, b) => b.roic - a.roic);

    const evRank = new Map<string, number>();
    byEvEbit.forEach((item, idx) => evRank.set(item.symbol, idx));

    const roicRank = new Map<string, number>();
    byRoic.forEach((item, idx) => roicRank.set(item.symbol, idx));

    return valid.slice().sort((a, b) => {
      const scoreA = (evRank.get(a.symbol) ?? 999) + (roicRank.get(a.symbol) ?? 999);
      const scoreB = (evRank.get(b.symbol) ?? 999) + (roicRank.get(b.symbol) ?? 999);
      return scoreA - scoreB;
    });
  }

  static bazinPreset(): ScreenerCriteria {
    return {
      minDy: 6.0,
      maxPl: 15.0,
      minPl: 1.0,
      maxPvp: null,
      minRoe: 10.0,
      minRoic: null,
      minMargemLiquida: 8.0,
      maxDividaLiqEbitda: 2.5,
      minLiquidezCorrente: 1.0,
    };
  }

  static grahamPreset(): ScreenerCriteria {
    return {
      minDy: null,
      maxPl: 15.0,
      minPl: 1.0,
      maxPvp: 1.5,
      minRoe: 8.0,
      minRoic: null,
      minMargemLiquida: 5.0,
      maxDividaLiqEbitda: 3.0,
      minLiquidezCorrente: 1.5,
    };
  }
}
