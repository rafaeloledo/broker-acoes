import { cvmCodes } from "../core/financials.ts";

export interface CoreMultiples {
  lpa: number | null;
  vpa: number | null;
  dpa: number | null;
  pl: number | null;
  pvp: number | null;
  dividendYield: number | null;
  grahamFairValue: number | null;
  bazinCeilingPrice: number | null;
}

export function emptyCoreMultiples(): CoreMultiples {
  return {
    lpa: null,
    vpa: null,
    dpa: null,
    pl: null,
    pvp: null,
    dividendYield: null,
    grahamFairValue: null,
    bazinCeilingPrice: null,
  };
}

export function coreMultiples(
  price: number,
  lucro: number | null | undefined,
  patrimonio: number | null | undefined,
  dividendos: number | null | undefined,
  shares: number | null | undefined
): CoreMultiples {
  if (shares == null || shares <= 0.0) {
    return emptyCoreMultiples();
  }

  const lpa = lucro != null ? lucro / shares : null;
  const vpa = patrimonio != null ? patrimonio / shares : null;
  const dpa = dividendos != null ? Math.abs(dividendos) / shares : null;

  const pl = lpa != null && lpa > 0.0 && price > 0.0 ? price / lpa : null;
  const pvp = vpa != null && vpa > 0.0 && price > 0.0 ? price / vpa : null;
  const dividendYield =
    dpa != null && price > 0.0 ? (dpa / price) * 100.0 : null;

  const grahamFairValue =
    lpa != null && vpa != null && lpa > 0.0 && vpa > 0.0
      ? Math.sqrt(22.5 * lpa * vpa)
      : null;

  const bazinCeilingPrice = dpa != null ? dpa / 0.06 : null;

  return {
    lpa,
    vpa,
    dpa,
    pl,
    pvp,
    dividendYield,
    grahamFairValue,
    bazinCeilingPrice,
  };
}

export function multiplesFromAccounts(
  price: number,
  accounts: readonly [string, number][]
): CoreMultiples {
  const map = new Map<string, number>(accounts);

  const lucro =
    map.get(cvmCodes.LUCRO_LIQUIDO_CONSOLIDADO) ??
    map.get(cvmCodes.LUCRO_LIQUIDO_CONTINUADO) ??
    null;
  const patrimonio = map.get(cvmCodes.PATRIMONIO_LIQUIDO) ?? null;
  const dividendos =
    map.get(cvmCodes.DIVIDENDOS_PAGOS) ??
    map.get(cvmCodes.DIVIDENDOS_PAGOS_ALT) ??
    null;
  const shares = map.get(cvmCodes.SHARES_TOTAL) ?? null;

  return coreMultiples(price, lucro, patrimonio, dividendos, shares);
}
