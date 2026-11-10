import { type Candle } from "../core/quote.ts";
import { type StockIndicators } from "../core/indicators.ts";
import { type CoreMultiples } from "./core-multiples.ts";

export type TrendSignal =
  | "StrongBuy"
  | "Buy"
  | "Neutral"
  | "Sell"
  | "StrongSell";

export function trendSignalLabel(signal: TrendSignal): string {
  switch (signal) {
    case "StrongBuy":
      return "Forte Compra";
    case "Buy":
      return "Compra";
    case "Neutral":
      return "Neutro";
    case "Sell":
      return "Venda";
    case "StrongSell":
      return "Forte Venda";
  }
}

export function trendSignalRgb(signal: TrendSignal): [number, number, number] {
  switch (signal) {
    case "StrongBuy":
      return [46, 160, 67];
    case "Buy":
      return [86, 180, 100];
    case "Neutral":
      return [150, 155, 165];
    case "Sell":
      return [220, 140, 50];
    case "StrongSell":
      return [248, 81, 73];
  }
}

export interface StockTrendAnalysis {
  ticker: string;
  name: string;
  sector: string;
  price: number;
  change1dPct: number;
  signal: TrendSignal;
  overallScore: number;
  technicalScore: number;
  fundamentalScore: number;
  rsi14: number;
  sma20: number;
  sma50: number;
  priceVsSma20Pct: number;
  priceVsSma50Pct: number;
  bollingerUpper: number;
  bollingerMiddle: number;
  bollingerLower: number;
  bollingerPctB: number;
  momentum20dPct: number;
  grahamFairValue: number;
  grahamUpsidePct: number;
  bazinCeilingPrice: number;
  bazinUpsidePct: number;
  dividendYield: number;
  pl: number;
  pvp: number;
  roe: number;
  margemLiquida: number;
  piotroskiFScore: number;
  altmanZScore: number;
  summaryRationale: string;
  buyCatalysts: string[];
  riskFactors: string[];
}

export function calculateSma(prices: readonly number[], period: number): number | null {
  if (period <= 0 || prices.length < period) return null;
  const slice = prices.slice(prices.length - period);
  const sum = slice.reduce((acc, v) => acc + v, 0.0);
  return sum / period;
}

export function calculateRsi(prices: readonly number[], period = 14): number | null {
  if (period <= 0 || prices.length < period + 1) return null;
  const start = prices.length - (period + 1);
  let gains = 0.0;
  let losses = 0.0;

  for (let i = start; i < prices.length - 1; i++) {
    const diff = prices[i + 1]! - prices[i]!;
    if (diff > 0) gains += diff;
    else losses += Math.abs(diff);
  }

  const avgGain = gains / period;
  const avgLoss = losses / period;
  if (avgLoss <= 1e-9) return 100.0;
  const rs = avgGain / avgLoss;
  return 100.0 - 100.0 / (1.0 + rs);
}

export function calculateBollingerBands(
  prices: readonly number[],
  period = 20,
  numStd = 2.0
): [number, number, number, number] | null {
  if (period <= 0 || prices.length < period) return null;
  const slice = prices.slice(prices.length - period);
  const middle = slice.reduce((acc, v) => acc + v, 0.0) / period;
  const variance =
    slice.reduce((acc, v) => acc + (v - middle) * (v - middle), 0.0) / period;
  const stdDev = Math.sqrt(variance);
  const upper = middle + numStd * stdDev;
  const lower = middle - numStd * stdDev;
  const lastPrice = prices[prices.length - 1]!;
  const bandwidth = upper - lower;
  const pctB = bandwidth <= 1e-6 ? 0.5 : (lastPrice - lower) / bandwidth;
  return [upper, middle, lower, pctB];
}

export function calculateMomentumPct(
  prices: readonly number[],
  period = 20
): number | null {
  if (period <= 0 || prices.length <= period) return null;
  const last = prices[prices.length - 1]!;
  const prev = prices[prices.length - 1 - period]!;
  if (Math.abs(prev) <= 1e-9) return null;
  return ((last - prev) / prev) * 100.0;
}

function clamp(v: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, v));
}

export class TrendingEngine {
  static analyzeStock(
    ticker: string,
    name: string,
    sector: string,
    price: number,
    candles: readonly Candle[],
    indicators: StockIndicators,
    coreMultiples?: CoreMultiples | null
  ): StockTrendAnalysis {
    const effectivePrice = price > 0 ? price : indicators.price;

    const prices: number[] = [];
    if (candles.length > 0) {
      for (const c of candles) {
        if (c.close > 0) prices.push(c.close);
      }
    }

    if (prices.length < 55) {
      prices.length = 0;
      for (let i = 0; i < 60; i++) {
        const pct = Math.sin((i + 1) * 0.15) * 0.012;
        prices.push(effectivePrice * (1.0 - pct));
      }
      prices[prices.length - 1] = effectivePrice;
    } else {
      prices[prices.length - 1] = effectivePrice;
    }

    // Previous close & daily change
    let prevClose =
      prices.length >= 2 ? prices[prices.length - 2]! : effectivePrice;
    if (candles.length > 0) {
      const lastCandle = candles[candles.length - 1]!;
      if (
        lastCandle.open > 0.0 &&
        Math.abs(lastCandle.open - effectivePrice) / lastCandle.open < 0.25
      ) {
        prevClose = lastCandle.open;
      }
    }
    const change1dPct =
      prevClose > 0.0 ? ((effectivePrice - prevClose) / prevClose) * 100.0 : 0.0;

    const sma20 = calculateSma(prices, 20) ?? effectivePrice;
    const sma50 = calculateSma(prices, 50) ?? effectivePrice;
    const rsi14 = calculateRsi(prices, 14) ?? 50.0;
    const [bollingerUpper, bollingerMiddle, bollingerLower, bollingerPctB] =
      calculateBollingerBands(prices, 20, 2.0) ?? [
        effectivePrice * 1.04,
        effectivePrice,
        effectivePrice * 0.96,
        0.5,
      ];
    const momentum20dPct = calculateMomentumPct(prices, 20) ?? 0.0;

    const priceVsSma20Pct =
      sma20 > 0.0 ? ((effectivePrice - sma20) / sma20) * 100.0 : 0.0;
    const priceVsSma50Pct =
      sma50 > 0.0 ? ((effectivePrice - sma50) / sma50) * 100.0 : 0.0;

    // Technical score
    let techScore = 50.0;
    if (priceVsSma20Pct > 4.0) techScore += 12.0;
    else if (priceVsSma20Pct > 0.0) techScore += 7.0;
    else if (priceVsSma20Pct < -4.0) techScore -= 12.0;
    else techScore -= 6.0;

    if (priceVsSma50Pct > 5.0) techScore += 10.0;
    else if (priceVsSma50Pct > 0.0) techScore += 5.0;
    else if (priceVsSma50Pct < -5.0) techScore -= 10.0;
    else techScore -= 5.0;

    if (sma20 > sma50) techScore += 10.0;
    else techScore -= 10.0;

    if (rsi14 < 30.0) techScore += 18.0;
    else if (rsi14 < 45.0) techScore += 9.0;
    else if (rsi14 <= 65.0) techScore += 14.0;
    else if (rsi14 <= 75.0) techScore += 6.0;
    else techScore -= 12.0;

    if (bollingerPctB < 0.20) techScore += 8.0;
    else if (bollingerPctB > 0.85) techScore += 4.0;
    else techScore += 5.0;

    if (momentum20dPct > 6.0) techScore += 10.0;
    else if (momentum20dPct > 1.0) techScore += 5.0;
    else if (momentum20dPct < -6.0) techScore -= 10.0;
    else if (momentum20dPct < -1.0) techScore -= 4.0;

    const technicalScore = clamp(techScore, 5.0, 98.0);

    // Fundamental score
    const effectiveGraham =
      coreMultiples?.grahamFairValue ?? indicators.grahamFairValue ?? 0.0;
    const effectiveBazin =
      coreMultiples?.bazinCeilingPrice ?? indicators.bazinCeilingPrice ?? 0.0;
    const effectiveDy =
      coreMultiples?.dividendYield ?? indicators.dividendYield ?? 0.0;
    const effectivePl = coreMultiples?.pl ?? indicators.pl ?? 0.0;
    const effectivePvp = coreMultiples?.pvp ?? indicators.pvp ?? 0.0;

    const grahamUpsidePct =
      effectivePrice > 0.0 && effectiveGraham > 0.0
        ? ((effectiveGraham - effectivePrice) / effectivePrice) * 100.0
        : 0.0;
    const bazinUpsidePct =
      effectivePrice > 0.0 && effectiveBazin > 0.0
        ? ((effectiveBazin - effectivePrice) / effectivePrice) * 100.0
        : 0.0;

    let fundScore = 50.0;
    if (grahamUpsidePct >= 50.0) fundScore += 24.0;
    else if (grahamUpsidePct >= 25.0) fundScore += 16.0;
    else if (grahamUpsidePct >= 5.0) fundScore += 8.0;
    else if (grahamUpsidePct < -25.0) fundScore -= 14.0;
    else fundScore -= 5.0;

    if (bazinUpsidePct >= 35.0) fundScore += 18.0;
    else if (bazinUpsidePct >= 15.0) fundScore += 12.0;
    else if (bazinUpsidePct >= 0.0) fundScore += 6.0;
    else if (bazinUpsidePct < -20.0) fundScore -= 10.0;
    else fundScore -= 4.0;

    if (effectiveDy >= 10.0) fundScore += 14.0;
    else if (effectiveDy >= 6.0) fundScore += 9.0;
    else if (effectiveDy >= 3.0) fundScore += 4.0;
    else if (effectiveDy === 0.0) fundScore -= 4.0;

    if (effectivePl > 0.0 && effectivePl <= 8.0) fundScore += 10.0;
    else if (effectivePl > 8.0 && effectivePl <= 15.0) fundScore += 6.0;
    else if (effectivePl > 25.0 || effectivePl < 0.0) fundScore -= 10.0;

    if (effectivePvp > 0.0 && effectivePvp <= 1.2) fundScore += 8.0;
    else if (effectivePvp > 1.2 && effectivePvp <= 2.2) fundScore += 4.0;
    else if (effectivePvp > 3.5) fundScore -= 8.0;

    if (indicators.roe >= 18.0) fundScore += 10.0;
    else if (indicators.roe >= 11.0) fundScore += 5.0;
    else if (indicators.roe < 0.0) fundScore -= 10.0;

    if (indicators.margemLiquida >= 14.0) fundScore += 8.0;
    else if (indicators.margemLiquida >= 7.0) fundScore += 4.0;
    else if (indicators.margemLiquida < 0.0) fundScore -= 8.0;

    if (indicators.piotroskiFScore >= 7) fundScore += 6.0;
    else if (indicators.piotroskiFScore >= 1 && indicators.piotroskiFScore <= 3)
      fundScore -= 6.0;

    const fundamentalScore = clamp(fundScore, 5.0, 98.0);
    const overallScore = clamp(
      0.45 * technicalScore + 0.55 * fundamentalScore,
      0.0,
      100.0
    );

    let signal: TrendSignal;
    if (overallScore >= 72.0) signal = "StrongBuy";
    else if (overallScore >= 57.0) signal = "Buy";
    else if (overallScore >= 43.0) signal = "Neutral";
    else if (overallScore >= 28.0) signal = "Sell";
    else signal = "StrongSell";

    const buyCatalysts: string[] = [];
    const riskFactors: string[] = [];

    if (grahamUpsidePct >= 15.0) {
      buyCatalysts.push(
        `Desconto de ${grahamUpsidePct.toFixed(1)}% frente ao valor intrínseco de Graham (R$ ${effectiveGraham.toFixed(2)})`
      );
    } else if (grahamUpsidePct < -10.0) {
      riskFactors.push(
        `Cotação acima do preço justo de Graham (R$ ${effectiveGraham.toFixed(2)})`
      );
    }

    if (bazinUpsidePct >= 5.0) {
      buyCatalysts.push(
        `Acima de 6% de yield projetado pelo teto de Bazin (R$ ${effectiveBazin.toFixed(2)}, margem +${bazinUpsidePct.toFixed(1)}%)`
      );
    } else if (bazinUpsidePct < -15.0) {
      riskFactors.push(
        `Acima do preço teto de Bazin (R$ ${effectiveBazin.toFixed(2)}) para meta de 6% a.a.`
      );
    }

    if (effectiveDy >= 6.0) {
      buyCatalysts.push(`Dividend Yield atrativo de ${effectiveDy.toFixed(2)}%`);
    }

    if (indicators.roe >= 15.0) {
      buyCatalysts.push(
        `Alta rentabilidade sobre patrimônio (ROE ${indicators.roe.toFixed(1)}%)`
      );
    }

    if (sma20 >= sma50 && priceVsSma20Pct >= 0.0) {
      buyCatalysts.push(
        `Tendência técnica de alta (preço acima da SMA20 e SMA20 > SMA50)`
      );
    } else if (priceVsSma20Pct < -2.0) {
      riskFactors.push(
        `Pressão vendedora de curto prazo (${priceVsSma20Pct.toFixed(1)}% vs SMA20)`
      );
    }

    if (rsi14 < 35.0) {
      buyCatalysts.push(`IFR-14 em região de sobrevenda (${rsi14.toFixed(1)})`);
    } else if (rsi14 > 70.0) {
      riskFactors.push(`IFR-14 em região de sobrecompra (${rsi14.toFixed(1)})`);
    }

    if (buyCatalysts.length === 0) {
      buyCatalysts.push(
        `Liquidez consistente na B3 e acompanhamento institucional no setor ${sector}`
      );
    }
    if (riskFactors.length === 0) {
      riskFactors.push(
        `Volatilidade macroeconômica e sensibilidade à curva de juros (Selic/DI)`
      );
    }

    const summaryRationale = `${ticker} apresenta sinal quantitativo de ${trendSignalLabel(
      signal
    )} (Score ${overallScore.toFixed(0)}/100), combinando fundamento ${fundamentalScore.toFixed(
      0
    )}/100 (DY ${effectiveDy.toFixed(1)}%, P/L ${effectivePl.toFixed(
      1
    )}x) e momento técnico ${technicalScore.toFixed(0)}/100 (IFR ${rsi14.toFixed(
      1
    )}).`;

    return {
      ticker,
      name,
      sector,
      price: effectivePrice,
      change1dPct,
      signal,
      overallScore,
      technicalScore,
      fundamentalScore,
      rsi14,
      sma20,
      sma50,
      priceVsSma20Pct,
      priceVsSma50Pct,
      bollingerUpper,
      bollingerMiddle,
      bollingerLower,
      bollingerPctB,
      momentum20dPct,
      grahamFairValue: effectiveGraham,
      grahamUpsidePct,
      bazinCeilingPrice: effectiveBazin,
      bazinUpsidePct,
      dividendYield: effectiveDy,
      pl: effectivePl,
      pvp: effectivePvp,
      roe: indicators.roe,
      margemLiquida: indicators.margemLiquida,
      piotroskiFScore: indicators.piotroskiFScore,
      altmanZScore: indicators.altmanZScore,
      summaryRationale,
      buyCatalysts,
      riskFactors,
    };
  }
}
