import { brtHourMinute } from "./quote.ts";

export type MetricId =
  | "Pl"
  | "Pvp"
  | "DividendYield"
  | "Graham"
  | "Bazin"
  | "Lpa"
  | "Vpa"
  | "Dpa"
  | "Outside";

export interface CoreMultiplesLike {
  lpa: number | null;
  vpa: number | null;
  dpa: number | null;
  pl: number | null;
  pvp: number | null;
  dividendYield: number | null;
  grahamFairValue: number | null;
  bazinCeilingPrice: number | null;
}

export function priceCaption(fetchedAtMs: number | null | undefined): string {
  if (fetchedAtMs != null && fetchedAtMs > 0) {
    const [hh, mm] = brtHourMinute(fetchedAtMs);
    const hStr = String(hh).padStart(2, "0");
    const mStr = String(mm).padStart(2, "0");
    return `Mercado · ${hStr}:${mStr}`;
  }
  return "Cotação local";
}

export function fundamentalsCaption(cvmYear: number | null | undefined): string {
  if (cvmYear != null && cvmYear > 0) {
    return `CVM ${cvmYear}`;
  }
  return "Cenário local";
}

export function changePercent(price: number, previousClose: number): number | null {
  if (previousClose > 0.0 && price > 0.0) {
    return ((price - previousClose) / previousClose) * 100.0;
  }
  return null;
}

export function formatMetric(
  core: CoreMultiplesLike | null | undefined,
  metric: MetricId
): string {
  if (!core || metric === "Outside") return "indisponível";

  switch (metric) {
    case "Pl":
      return core.pl != null ? core.pl.toFixed(2) : "indisponível";
    case "Pvp":
      return core.pvp != null ? core.pvp.toFixed(2) : "indisponível";
    case "DividendYield":
      return core.dividendYield != null
        ? `${core.dividendYield.toFixed(2)}%`
        : "indisponível";
    case "Graham":
      return core.grahamFairValue != null
        ? `R$ ${core.grahamFairValue.toFixed(2)}`
        : "indisponível";
    case "Bazin":
      return core.bazinCeilingPrice != null
        ? `R$ ${core.bazinCeilingPrice.toFixed(2)}`
        : "indisponível";
    case "Lpa":
      return core.lpa != null ? `R$ ${core.lpa.toFixed(2)}` : "indisponível";
    case "Vpa":
      return core.vpa != null ? `R$ ${core.vpa.toFixed(2)}` : "indisponível";
    case "Dpa":
      return core.dpa != null ? `R$ ${core.dpa.toFixed(2)}` : "indisponível";
  }
}
