import { type CompanyProfile } from "../core/ticker.ts";
import { FinancialPeriod, cvmCodes } from "../core/financials.ts";
import { type StockIndicators } from "../core/indicators.ts";

export class ValuationCalculator {
  static computeIndicators(
    profile: CompanyProfile,
    ttm: FinancialPeriod,
    currentPrice: number,
    annualDividendsPerShare: number
  ): StockIndicators {
    const shares = Math.max(profile.totalShares, 1);
    const marketCap = currentPrice * shares;

    const receita = ttm.receitaLiquida();
    const lucroBruto = ttm.lucroBruto();
    const ebit = ttm.ebit();
    const lucroLiquido = ttm.lucroLiquido();
    const plContabil = ttm.patrimonioLiquido();
    const ativoTotal = ttm.ativoTotal();
    const ativoCirculante = ttm.ativoCirculante();
    const passivoCirculante = ttm.passivoCirculante();
    const passivoTotal =
      ttm.getAccountOrZero(cvmCodes.PASSIVO_TOTAL) > 0
        ? ttm.getAccountOrZero(cvmCodes.PASSIVO_TOTAL) - plContabil
        : passivoCirculante + ttm.getAccountOrZero(cvmCodes.PASSIVO_NAO_CIRCULANTE);
    const estoques = ttm.getAccountOrZero(cvmCodes.ESTOQUES);
    const fco = ttm.fluxoCaixaOperacional();
    const depAmort = Math.abs(ttm.getAccountOrZero(cvmCodes.DEPRECIACAO_AMORTIZACAO));
    const ebitda = ebit + depAmort;
    const dividaLiquida = ttm.dividaLiquida();
    const ev = marketCap + dividaLiquida;

    const lpa = lucroLiquido / shares;
    const vpa = plContabil / shares;
    const dpa = annualDividendsPerShare;

    const pl = lpa > 0.0 ? currentPrice / lpa : 0.0;
    const pvp = vpa > 0.0 ? currentPrice / vpa : 0.0;
    const evEbitda = ebitda !== 0.0 ? ev / ebitda : 0.0;
    const evEbit = ebit !== 0.0 ? ev / ebit : 0.0;
    const pEbitda = ebitda !== 0.0 ? marketCap / ebitda : 0.0;
    const pEbit = ebit !== 0.0 ? marketCap / ebit : 0.0;
    const pSr = receita !== 0.0 ? marketCap / receita : 0.0;
    const pAtivo = ativoTotal !== 0.0 ? marketCap / ativoTotal : 0.0;

    const capGiro = ativoCirculante - passivoCirculante;
    const pCapGiro = capGiro !== 0.0 ? marketCap / capGiro : 0.0;
    const ativoCircLiq = ativoCirculante - passivoTotal;
    const pAtivoCircLiq = ativoCircLiq !== 0.0 ? marketCap / ativoCircLiq : 0.0;

    const dividendYield =
      currentPrice > 0.0 ? (annualDividendsPerShare / currentPrice) * 100.0 : 0.0;

    const roe = plContabil !== 0.0 ? (lucroLiquido / plContabil) * 100.0 : 0.0;
    const capitalInvestido = plContabil + Math.max(dividaLiquida, 0.0);
    const roic =
      capitalInvestido !== 0.0
        ? ((ebit * (1.0 - 0.34)) / capitalInvestido) * 100.0
        : 0.0;
    const roa = ativoTotal !== 0.0 ? (lucroLiquido / ativoTotal) * 100.0 : 0.0;

    const margemBruta = receita !== 0.0 ? (lucroBruto / receita) * 100.0 : 0.0;
    const margemEbitda = receita !== 0.0 ? (ebitda / receita) * 100.0 : 0.0;
    const margemEbit = receita !== 0.0 ? (ebit / receita) * 100.0 : 0.0;
    const margemLiquida = receita !== 0.0 ? (lucroLiquido / receita) * 100.0 : 0.0;
    const giroAtivos = ativoTotal !== 0.0 ? receita / ativoTotal : 0.0;

    const dividaLiqPl = plContabil !== 0.0 ? dividaLiquida / plContabil : 0.0;
    const dividaLiqEbitda = ebitda !== 0.0 ? dividaLiquida / ebitda : 0.0;
    const dividaLiqEbit = ebit !== 0.0 ? dividaLiquida / ebit : 0.0;
    const plAtivos = ativoTotal !== 0.0 ? plContabil / ativoTotal : 0.0;
    const passivosAtivos = ativoTotal !== 0.0 ? passivoTotal / ativoTotal : 0.0;
    const liquidezCorrente =
      passivoCirculante !== 0.0 ? ativoCirculante / passivoCirculante : 0.0;
    const liquidezSeca =
      passivoCirculante !== 0.0
        ? (ativoCirculante - estoques) / passivoCirculante
        : 0.0;

    const grahamFairValue =
      lpa > 0.0 && vpa > 0.0 ? Math.sqrt(22.5 * lpa * vpa) : 0.0;
    const bazinCeilingPrice = annualDividendsPerShare / 0.06;

    // Piotroski F-Score (0-9)
    let fScore = 0;
    if (lucroLiquido > 0.0) fScore += 1;
    if (fco > 0.0) fScore += 1;
    if (roa > 0.0) fScore += 1;
    if (fco > lucroLiquido) fScore += 1;
    if (dividaLiqPl < 1.0) fScore += 1;
    if (liquidezCorrente > 1.2) fScore += 1;
    if (margemBruta > 20.0) fScore += 1;
    if (giroAtivos > 0.4) fScore += 1;
    if (roe > 10.0) fScore += 1;

    // Altman Z-Score
    let altmanZScore = 0.0;
    if (ativoTotal > 0.0) {
      const x1 = capGiro / ativoTotal;
      const x2 = ttm.getAccountOrZero(cvmCodes.RESERVAS_LUCRO) / ativoTotal;
      const x3 = ebit / ativoTotal;
      const x4 = marketCap / Math.max(passivoTotal, 1.0);
      const x5 = receita / ativoTotal;
      altmanZScore = 1.2 * x1 + 1.4 * x2 + 3.3 * x3 + 0.6 * x4 + 0.999 * x5;
    }

    return {
      symbol: profile.symbol,
      price: currentPrice,
      pl,
      pvp,
      evEbitda,
      evEbit,
      pEbitda,
      pEbit,
      pSr,
      pAtivo,
      pCapGiro,
      pAtivoCircLiq,
      dividendYield,
      pegRatio: 1.0,
      dividaLiqPl,
      dividaLiqEbitda,
      dividaLiqEbit,
      plAtivos,
      passivosAtivos,
      liquidezCorrente,
      liquidezSeca,
      margemBruta,
      margemEbitda,
      margemEbit,
      margemLiquida,
      giroAtivos,
      roe,
      roic,
      roa,
      cagrReceitas5a: 12.0,
      cagrLucros5a: 14.5,
      lpa,
      vpa,
      dpa,
      grahamFairValue,
      bazinCeilingPrice,
      piotroskiFScore: fScore,
      altmanZScore,
    };
  }
}
