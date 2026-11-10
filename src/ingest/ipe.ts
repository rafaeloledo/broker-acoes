import { unzipSync } from "fflate";
import { type NewsItem } from "../core/news.ts";
import { normalize } from "../core/b3-assets.ts";
import { cnpjDigits, issuerForTicker, tickersForCnpj } from "./companies.ts";
import { decodeCvmText } from "./dfp.ts";

export function filterIpeCsv(
  csvText: string,
  tickers: readonly string[]
): NewsItem[] {
  const wantedTickers = new Set(tickers.map((t) => t.trim().toUpperCase()));
  const watchlistCnpjs = new Set<string>();
  for (const t of wantedTickers) {
    const iss = issuerForTicker(t);
    if (iss) watchlistCnpjs.add(iss.cnpjDigits);
  }
  if (watchlistCnpjs.size === 0) return [];

  const lines = csvText.split(/\r?\n/);
  if (lines.length < 2) return [];

  const headers = lines[0]!.split(";").map((h) => normalize(h.trim()));
  const idxCnpj = headers.indexOf("CNPJ_COMPANHIA");
  const idxCategoria = headers.indexOf("CATEGORIA");
  const idxAssunto = headers.indexOf("ASSUNTO");
  const idxDtRefer = headers.indexOf("DATA_REFERENCIA");
  const idxDtEntrega = headers.indexOf("DATA_ENTREGA");
  const idxLink = headers.indexOf("LINK_DOWNLOAD");
  const idxProtocolo = headers.indexOf("PROTOCOLO_ENTREGA");

  if (idxCnpj < 0 || idxCategoria < 0 || idxLink < 0) return [];

  const out: NewsItem[] = [];
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i]!;
    if (!line.trim()) continue;
    const cols = line.split(";");

    const categoria = normalize((cols[idxCategoria] ?? "").trim());
    if (categoria !== "FATO RELEVANTE") continue;

    const cnpj = cnpjDigits(cols[idxCnpj] ?? "");
    if (!watchlistCnpjs.has(cnpj)) continue;

    const url = (cols[idxLink] ?? "").trim();
    if (!url.startsWith("http://") && !url.startsWith("https://")) continue;

    const rawTitle = idxAssunto >= 0 ? (cols[idxAssunto] ?? "").trim() : "";
    const title = rawTitle || "Fato relevante";

    const dateStr =
      (idxDtEntrega >= 0 ? (cols[idxDtEntrega] ?? "").trim() : "") ||
      (idxDtRefer >= 0 ? (cols[idxDtRefer] ?? "").trim() : "");
    const ymd = dateStr.slice(0, 10);
    const publishedAtMs = /^\d{4}-\d{2}-\d{2}$/.test(ymd)
      ? Date.parse(`${ymd}T15:00:00Z`)
      : Date.now();

    const externalId =
      idxProtocolo >= 0 ? (cols[idxProtocolo] ?? "").trim() || null : null;

    const mapped = tickersForCnpj(cnpj);
    for (const t of mapped) {
      if (wantedTickers.has(t)) {
        out.push({
          ticker: t,
          title,
          url,
          source: "CVM",
          publishedAtMs,
          summary: null,
          externalId,
        });
      }
    }
  }

  return out;
}

export function extractIpeNews(
  zipBytes: Uint8Array,
  tickers: readonly string[]
): NewsItem[] {
  const files = unzipSync(zipBytes);
  const items: NewsItem[] = [];
  for (const [name, data] of Object.entries(files)) {
    if (!name.toLowerCase().endsWith(".csv")) continue;
    const text = decodeCvmText(data);
    items.push(...filterIpeCsv(text, tickers));
  }
  return items;
}

export async function downloadIpeNews(
  year: number,
  tickers: readonly string[]
): Promise<NewsItem[]> {
  const url = `https://dados.cvm.gov.br/dados/CIA_ABERTA/DOC/IPE/DADOS/ipe_cia_aberta_${year}.zip`;
  const res = await fetch(url, { signal: AbortSignal.timeout(20000) });
  if (res.status === 404) return [];
  if (!res.ok) {
    throw new Error(`CVM IPE download failed for ${year} with HTTP ${res.status}`);
  }
  const buf = new Uint8Array(await res.arrayBuffer());
  return extractIpeNews(buf, tickers);
}
