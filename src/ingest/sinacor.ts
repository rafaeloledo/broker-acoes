import { type TradeOperation } from "../core/tax.ts";

export const SAMPLE_SINACOR_NOTE = `
NOTA DE CORRETAGEM - B3 / SINACOR
Data pregão: 01/01/2026
Corretora: XP Investimentos
1-BOVESPA C VISTA PETR4 PETROBRAS PN 200 34,20 6.840,00 D
1-BOVESPA C VISTA VALE3 VALE ON 100 56,50 5.650,00 D
1-BOVESPA C VISTA WEGE3 WEG ON 150 48,00 7.200,00 D
1-BOVESPA C VISTA HGLG11 CSHG LOG FII 50 151,00 7.550,00 D
1-BOVESPA C VISTA BBAS3 BRASIL ON 200 26,80 5.360,00 D
1-BOVESPA V VISTA PETR4 PETROBRAS PN 50 37,10 1.855,00 C
`.trim();

function parsePtNumber(raw: string): number {
  const s = raw.trim();
  if (s.includes(",") && s.includes(".")) {
    return Number.parseFloat(s.replace(/\./g, "").replace(",", "."));
  }
  if (s.includes(",")) {
    return Number.parseFloat(s.replace(",", "."));
  }
  return Number.parseFloat(s);
}

export class SinacorParser {
  static parseTradeLines(text: string, broker = "XP Investimentos"): TradeOperation[] {
    let dateStr = "2026-01-01";
    const dateMatch = text.match(/(\d{2})\/(\d{2})\/(\d{4})/);
    if (dateMatch) {
      dateStr = `${dateMatch[3]}-${dateMatch[2]}-${dateMatch[1]}`;
    }

    const compactDate = dateStr.replace(/-/g, "");
    const lineRegex =
      /\b([CV])\s+(?:VISTA|FRACIONARIO|FRAC)\s+([A-Z0-9]{4,6})\s+.*?\s+(\d+)\s+([\d.,]+)\s+([\d.,]+)\s+([CD])\b/gi;

    const trades: TradeOperation[] = [];
    let match: RegExpExecArray | null;
    let idx = 1;

    while ((match = lineRegex.exec(text)) !== null) {
      const sideChar = match[1]!.toUpperCase();
      const symbol = match[2]!.toUpperCase();
      const quantity = Number.parseFloat(match[3]!);
      const unitPrice = parsePtNumber(match[4]!);
      const totalGross = parsePtNumber(match[5]!);

      if (
        Number.isNaN(quantity) ||
        Number.isNaN(unitPrice) ||
        Number.isNaN(totalGross)
      ) {
        continue;
      }

      const side = sideChar === "V" ? "Sell" : "Buy";
      const liquidationFee = Math.max(totalGross * 0.00025, 0.01);
      const registrationFee = Math.max(totalGross * 0.00005, 0.01);
      const irrf = side === "Sell" ? totalGross * 0.00005 : 0.0;

      trades.push({
        id: `TRD-${compactDate}-${idx}`,
        date: dateStr,
        symbol,
        side,
        kind: "SwingTrade",
        quantity,
        unitPrice,
        totalGross,
        liquidationFee,
        registrationFee,
        brokerageFee: 0.0,
        issTax: 0.0,
        irrf,
        brokerName: broker,
        noteNumber: `NC-${idx}`,
      });
      idx += 1;
    }

    return trades;
  }
}
