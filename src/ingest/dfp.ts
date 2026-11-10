import { unzipSync } from "fflate";
import { cvmCodes, CORE_ACCOUNTS } from "../core/financials.ts";
import { normalize } from "../core/b3-assets.ts";
import { cnpjDigits, issuerForTicker, tickersForCnpj } from "./companies.ts";

export interface ExtractedTickerYear {
  ticker: string;
  year: number;
  referenceDate: string;
  accounts: [string, number][];
}

export function candidateDfpYears(todayYear: number): number[] {
  return [todayYear - 1, todayYear - 2];
}

export function nextDfpYear(
  stored: ReadonlyMap<string, number | null>,
  candidates: readonly number[],
  tickers: readonly string[]
): number | null {
  const mappedTickers = tickers.filter((t) => issuerForTicker(t) != null);
  if (mappedTickers.length === 0) return null;

  for (const year of candidates) {
    const missingAny = mappedTickers.some((t) => {
      const latest = stored.get(t.toUpperCase()) ?? null;
      return latest == null || latest < year;
    });
    if (missingAny) return year;
  }
  return null;
}

export function decodeCvmText(bytes: Uint8Array): string {
  try {
    const utf8 = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    return utf8;
  } catch {
    return new TextDecoder("iso-8859-1").decode(bytes);
  }
}

export function sharesInUnits(raw: number): number {
  if (raw > 0.0 && raw < 100_000_000.0) {
    return raw * 1000.0;
  }
  return raw;
}

interface StatementRowAcc {
  accounts: Map<string, number>;
  plDepth: number;
  referenceDate: string;
}

export function filterStatementCsv(
  csvText: string,
  watchlistCnpjs: ReadonlySet<string>,
  accMap: Map<string, StatementRowAcc>
): void {
  const lines = csvText.split(/\r?\n/);
  if (lines.length < 2) return;

  const headers = lines[0]!.split(";").map((h) => normalize(h.trim()));
  const idxCnpj = headers.indexOf("CNPJ_CIA");
  const idxDtRefer = headers.indexOf("DT_REFER");
  const idxOrdem = headers.indexOf("ORDEM_EXERC");
  const idxEscala = headers.indexOf("ESCALA_MOEDA");
  const idxCdConta = headers.indexOf("CD_CONTA");
  const idxDsConta = headers.indexOf("DS_CONTA");
  const idxVlConta = headers.indexOf("VL_CONTA");

  if (
    idxCnpj < 0 ||
    idxDtRefer < 0 ||
    idxCdConta < 0 ||
    idxDsConta < 0 ||
    idxVlConta < 0
  ) {
    return;
  }

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i]!;
    if (!line.trim()) continue;
    const cols = line.split(";");
    const cnpj = cnpjDigits(cols[idxCnpj] ?? "");
    if (!watchlistCnpjs.has(cnpj)) continue;

    if (idxOrdem >= 0) {
      const ordem = normalize(cols[idxOrdem] ?? "");
      if (ordem.includes("PENULT")) continue;
    }

    const dtRefer = (cols[idxDtRefer] ?? "").trim();
    const year = Number.parseInt(dtRefer.slice(0, 4), 10);
    if (Number.isNaN(year)) continue;

    let vl = Number.parseFloat((cols[idxVlConta] ?? "").trim().replace(",", "."));
    if (Number.isNaN(vl)) continue;

    if (idxEscala >= 0) {
      const escala = normalize(cols[idxEscala] ?? "");
      if (escala.includes("MIL")) {
        vl *= 1000.0;
      }
    }

    const cdConta = (cols[idxCdConta] ?? "").trim();
    const dsFolded = normalize((cols[idxDsConta] ?? "").trim());

    const key = `${cnpj}:${year}`;
    let entry = accMap.get(key);
    if (!entry) {
      entry = {
        accounts: new Map(),
        plDepth: Number.MAX_SAFE_INTEGER,
        referenceDate: dtRefer,
      };
      accMap.set(key, entry);
    }

    // Bank-safe equity canonicalization
    if (dsFolded.includes("PATRIM") && dsFolded.includes("LIQUID")) {
      const depth = (cdConta.match(/\./g) ?? []).length;
      if (depth <= entry.plDepth) {
        entry.plDepth = depth;
        entry.accounts.set(cvmCodes.PATRIMONIO_LIQUIDO, vl);
      }
      continue;
    }

    // Dividends / JCP paid summation
    const isDivOrJcp =
      dsFolded.includes("DIVIDEND") ||
      dsFolded.includes("JUROS SOBRE") ||
      dsFolded.includes("JCP");
    const isPaid =
      (dsFolded.includes("PAG") || dsFolded.includes("PGTO")) &&
      !dsFolded.includes("RECEB");

    if (isDivOrJcp && isPaid) {
      const prev = entry.accounts.get(cvmCodes.DIVIDENDOS_PAGOS) ?? 0.0;
      entry.accounts.set(cvmCodes.DIVIDENDOS_PAGOS, prev + vl);
      continue;
    }

    if (
      CORE_ACCOUNTS.includes(cdConta) &&
      cdConta !== cvmCodes.PATRIMONIO_LIQUIDO &&
      cdConta !== cvmCodes.DIVIDENDOS_PAGOS &&
      cdConta !== cvmCodes.DIVIDENDOS_PAGOS_ALT
    ) {
      entry.accounts.set(cdConta, vl);
    }
  }
}

export function filterCapitalCsv(
  csvText: string,
  watchlistCnpjs: ReadonlySet<string>,
  accMap: Map<string, StatementRowAcc>
): void {
  const lines = csvText.split(/\r?\n/);
  if (lines.length < 2) return;

  const headers = lines[0]!.split(";").map((h) => normalize(h.trim()));
  const idxCnpj = headers.indexOf("CNPJ_CIA");
  const idxDtRefer = headers.indexOf("DT_REFER");
  const idxTotal = headers.findIndex(
    (h) =>
      h.includes("QUANTIDADE_TOTAL_ACOES_CAPITAL_INTEGRALIZADO") ||
      h.includes("QT_ACAO_TOTAL_CAP_INTEGR")
  );
  const idxOn = headers.findIndex(
    (h) =>
      h.includes("QUANTIDADE_ACAO_ORDINARIA_CAPITAL_INTEGRALIZADO") ||
      h.includes("QT_ACAO_ORDIN_CAP_INTEGR")
  );
  const idxPn = headers.findIndex(
    (h) =>
      h.includes("QUANTIDADE_ACAO_PREFERENCIAL_CAPITAL_INTEGRALIZADO") ||
      h.includes("QT_ACAO_PREF_CAP_INTEGR")
  );

  if (idxCnpj < 0 || idxDtRefer < 0) return;

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i]!;
    if (!line.trim()) continue;
    const cols = line.split(";");
    const cnpj = cnpjDigits(cols[idxCnpj] ?? "");
    if (!watchlistCnpjs.has(cnpj)) continue;

    const dtRefer = (cols[idxDtRefer] ?? "").trim();
    const year = Number.parseInt(dtRefer.slice(0, 4), 10);
    if (Number.isNaN(year)) continue;

    let rawShares = 0.0;
    if (idxTotal >= 0) {
      rawShares =
        Number.parseFloat((cols[idxTotal] ?? "").trim().replace(",", ".")) || 0.0;
    }
    if (rawShares <= 0.0 && (idxOn >= 0 || idxPn >= 0)) {
      const on =
        idxOn >= 0
          ? Number.parseFloat((cols[idxOn] ?? "").trim().replace(",", ".")) || 0.0
          : 0.0;
      const pn =
        idxPn >= 0
          ? Number.parseFloat((cols[idxPn] ?? "").trim().replace(",", ".")) || 0.0
          : 0.0;
      rawShares = on + pn;
    }

    if (rawShares <= 0.0) continue;
    const shares = sharesInUnits(rawShares);

    const key = `${cnpj}:${year}`;
    let entry = accMap.get(key);
    if (!entry) {
      entry = {
        accounts: new Map(),
        plDepth: Number.MAX_SAFE_INTEGER,
        referenceDate: dtRefer,
      };
      accMap.set(key, entry);
    }
    entry.accounts.set(cvmCodes.SHARES_TOTAL, shares);
  }
}

export function extractWatchlistAccounts(
  zipBytes: Uint8Array,
  tickers: readonly string[]
): ExtractedTickerYear[] {
  const watchlistCnpjs = new Set<string>();
  const wantedTickerSet = new Set(tickers.map((t) => t.trim().toUpperCase()));

  for (const t of wantedTickerSet) {
    const iss = issuerForTicker(t);
    if (iss) watchlistCnpjs.add(iss.cnpjDigits);
  }

  if (watchlistCnpjs.size === 0) return [];

  const files = unzipSync(zipBytes);
  const accMap = new Map<string, StatementRowAcc>();

  for (const [name, data] of Object.entries(files)) {
    const lower = name.toLowerCase();
    if (!lower.endsWith(".csv")) continue;

    if (lower.includes("composicao_capital")) {
      const text = decodeCvmText(data);
      filterCapitalCsv(text, watchlistCnpjs, accMap);
    } else if (
      lower.includes("dre_con") ||
      lower.includes("bpp_con") ||
      lower.includes("dfc_mi_con")
    ) {
      const text = decodeCvmText(data);
      filterStatementCsv(text, watchlistCnpjs, accMap);
    }
  }

  const results: ExtractedTickerYear[] = [];
  for (const [key, entry] of accMap.entries()) {
    const [cnpj, yearStr] = key.split(":");
    const year = Number(yearStr);
    const mappedTickers = tickersForCnpj(cnpj ?? "");
    const accounts = Array.from(entry.accounts.entries());
    if (accounts.length === 0) continue;

    for (const t of mappedTickers) {
      if (wantedTickerSet.has(t)) {
        results.push({
          ticker: t,
          year,
          referenceDate: entry.referenceDate,
          accounts,
        });
      }
    }
  }

  return results;
}

export async function downloadDfpAccounts(
  year: number,
  tickers: readonly string[]
): Promise<ExtractedTickerYear[]> {
  const url = `https://dados.cvm.gov.br/dados/CIA_ABERTA/DOC/DFP/DADOS/dfp_cia_aberta_${year}.zip`;
  const res = await fetch(url, { signal: AbortSignal.timeout(25000) });
  if (res.status === 404) return [];
  if (!res.ok) {
    throw new Error(`CVM DFP download failed for ${year} with HTTP ${res.status}`);
  }
  const buf = new Uint8Array(await res.arrayBuffer());
  return extractWatchlistAccounts(buf, tickers);
}
