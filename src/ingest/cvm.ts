import { FinancialPeriod, type PeriodKind } from "../core/financials.ts";

export interface CvmAccountRecord {
  cnpjCia: string;
  denomCia: string;
  cdCvm: number;
  dtRefer: string;
  cdConta: string;
  dsConta: string;
  vlConta: number;
}

export class CvmIngestor {
  static parseCvmCsvLine(line: string): CvmAccountRecord | null {
    const cols = line.split(";");
    if (cols.length < 12) return null;

    const cnpjCia = cols[0]!.trim();
    const dtRefer = cols[1]!.trim();
    if (!/^\d{4}-\d{2}-\d{2}$/.test(dtRefer)) return null;

    const denomCia = cols[3]!.trim();
    const cdCvm = Number.parseInt(cols[4]!.trim(), 10);
    if (Number.isNaN(cdCvm)) return null;

    const cdConta = cols[8]!.trim();
    const dsConta = cols[9]!.trim();
    const vlConta = Number.parseFloat(cols[10]!.trim().replace(",", "."));
    if (Number.isNaN(vlConta)) return null;

    return {
      cnpjCia,
      denomCia,
      cdCvm,
      dtRefer,
      cdConta,
      dsConta,
      vlConta,
    };
  }

  static buildFinancialPeriod(
    year: number,
    kind: PeriodKind,
    records: readonly CvmAccountRecord[]
  ): FinancialPeriod {
    const accounts: Record<string, number> = {};
    for (const r of records) {
      accounts[r.cdConta] = r.vlConta;
    }
    return new FinancialPeriod(
      year,
      kind,
      `${year}-01-01`,
      `${year}-12-31`,
      accounts
    );
  }
}
