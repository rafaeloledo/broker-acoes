import { type StockIndicators, defaultStockIndicators } from "../core/indicators.ts";
import { type FiiDetails, type FiiSegment, defaultFiiDetails } from "../core/fii.ts";
import {
  type CompanyProfile,
  defaultCompanyProfile,
  marketSectorFromName,
} from "../core/ticker.ts";
import { normalize } from "../core/b3-assets.ts";

export interface StatusInvestAsset {
  ticker: string;
  category: string;
  name: string;
  sector: string;
  subsector: string;
  segment: string;
  price: number;
  changePercent: number;
  previousClose: number;
  min52s: number;
  max52s: number;
  dividendYield: number;
  pl: number;
  pvp: number;
  evEbitda: number;
  evEbit: number;
  pEbitda: number;
  pEbit: number;
  vpa: number;
  lpa: number;
  dpa: number;
  pSr: number;
  pAtivo: number;
  pCapGiro: number;
  pAtivoCircLiq: number;
  pegRatio: number;
  dividaLiqPl: number;
  dividaLiqEbitda: number;
  dividaLiqEbit: number;
  liquidezCorrente: number;
  margemBruta: number;
  margemEbitda: number;
  margemEbit: number;
  margemLiquida: number;
  roe: number;
  roa: number;
  roic: number;
  giroAtivos: number;
  cagrReceitas5a: number;
  cagrLucros5a: number;
  grahamFairValue: number;
  bazinCeilingPrice: number;
  // FII fields
  fiiNavPerShare: number;
  fiiLastDividend: number;
  fiiShareholdersCount: number | null;
  fiiTotalShares?: number | null;
  fiiNetAssetValue?: number | null;
  fiiSegment: string;
  fiiPhysicalVacancy: number | null;
  fiiFinancialVacancy: number | null;
  fiiAdminFee: number | null;
  fiiPerformanceFee?: string | null;
  fiiAdminName?: string | null;
  fiiMandate?: string | null;
}

const STOCK_UNITS = new Set([
  "TAEE11",
  "KLBN11",
  "BPAC11",
  "SANB11",
  "SAPR11",
  "ENGI11",
  "ALUP11",
  "TIET11",
  "SULA11",
]);

const ETF_11 = new Set([
  "BOVA11",
  "IVVB11",
  "SMAL11",
  "HASH11",
  "GOLD11",
  "NASD11",
  "DIVO11",
  "BRAX11",
  "SPXI11",
]);

export function decodeHtmlEntities(raw: string): string {
  if (!raw) return "";
  return raw
    .replace(/&#x([0-9a-fA-F]+);/g, (_, hex) =>
      String.fromCodePoint(Number.parseInt(hex, 16))
    )
    .replace(/&#(\d+);/g, (_, dec) =>
      String.fromCodePoint(Number.parseInt(dec, 10))
    )
    .replace(/&nbsp;/gi, " ")
    .replace(/&amp;/gi, "&")
    .replace(/&quot;/gi, '"')
    .replace(/&apos;/gi, "'")
    .replace(/&lt;/gi, "<")
    .replace(/&gt;/gi, ">")
    .trim();
}

export function categoryOrderFor(ticker: string): string[] {
  const sym = ticker.trim().toUpperCase();
  if (STOCK_UNITS.has(sym)) {
    return ["acoes", "fundos-imobiliarios", "etfs"];
  }
  if (ETF_11.has(sym)) {
    return ["etfs", "acoes", "fundos-imobiliarios"];
  }
  if (sym.endsWith("34") || sym.endsWith("35") || sym.endsWith("33")) {
    return ["bdrs", "acoes", "etfs"];
  }
  if (sym.endsWith("11")) {
    return ["fundos-imobiliarios", "acoes", "etfs"];
  }
  return ["acoes", "fundos-imobiliarios", "bdrs", "etfs"];
}

export function parsePtBrF64(raw: string): number | null {
  const cleaned = decodeHtmlEntities(raw)
    .replace(/R\$/gi, "")
    .replace(/%/g, "")
    .replace(/a\.a\.?/gi, "")
    .replace(/\s+/g, "")
    .trim();
  if (!cleaned || cleaned === "-" || cleaned === "--" || cleaned === "-%") {
    return null;
  }
  const normalized = cleaned.replace(/\./g, "").replace(",", ".");
  const val = Number.parseFloat(normalized);
  return Number.isFinite(val) ? val : null;
}

function classifyFiiSegment(raw: string): FiiSegment {
  const s = normalize(decodeHtmlEntities(raw));
  if (s.includes("LOGIST") || s.includes("GALP") || s.includes("INDUSTR"))
    return "Logistica";
  if (s.includes("LAJE") || s.includes("ESCRIT")) return "LajesCorporativas";
  if (s.includes("SHOP")) return "Shoppings";
  if (
    s.includes("PAPEL") ||
    s.includes("RECEB") ||
    s.includes("TITULO") ||
    s.includes("VALORES MOBILIARIOS") ||
    s.includes("CRI")
  )
    return "PapeisCRIs";
  if (s.includes("FOF") || s.includes("FUNDO DE FUNDO")) return "FoF";
  if (s.includes("AGRO") || s.includes("FIAGRO")) return "Agro";
  if (s.includes("EDUC") || s.includes("HOSP")) return "EducacionalHospitalar";
  if (s.includes("RESID") || s.includes("INCORPOR")) return "Residencial";
  if (s.includes("HIBR") || s.includes("RENDA URB") || s.includes("MISTO"))
    return "Hibrido";
  return "Hibrido";
}

export interface FiiExternalEnrichment {
  shareholdersCount: number | null;
  totalShares: number | null;
  netAssetValue: number | null;
  physicalVacancy: number | null;
  financialVacancy: number | null;
  adminFee: number | null;
  performanceFee: string | null;
  adminName: string | null;
  mandate: string | null;
  segment: string | null;
}

export class StatusInvestClient {
  private readonly userAgent =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

  async fetchHtml(url: string): Promise<string | null> {
    // 1. Try curl first to bypass Cloudflare TLS fingerprinting
    try {
      const proc = Bun.spawn(
        ["curl", "-s", "-L", "-A", this.userAgent, "--max-time", "12", url],
        { stdout: "pipe", stderr: "ignore" }
      );
      const text = await new Response(proc.stdout).text();
      await proc.exited;
      if (this.isValidHtml(text)) {
        return text;
      }
    } catch {
      // Fallback to fetch
    }

    // 2. Fallback to native fetch
    try {
      const res = await fetch(url, {
        headers: {
          "User-Agent": this.userAgent,
          Accept:
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
          "Accept-Language": "pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7",
        },
        signal: AbortSignal.timeout(12000),
      });
      if (!res.ok) return null;
      const text = await res.text();
      return this.isValidHtml(text) ? text : null;
    } catch {
      return null;
    }
  }

  private isValidHtml(html: string): boolean {
    if (!html || html.length <= 3000) return false;
    if (
      html.includes("Just a moment...") ||
      html.includes("Página não encontrada") ||
      html.includes("Ops! Não encontramos")
    ) {
      return false;
    }
    return true;
  }

  static parseHtml(
    ticker: string,
    category: string,
    html: string
  ): StatusInvestAsset | null {
    const sym = ticker.trim().toUpperCase();
    const pairs = new Map<string, string>();

    const pairRegexes = [
      /<h3[^>]*class="(?:[^"]*\s)?title(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/h3>[\s\S]{0,600}?<strong[^>]*class="(?:[^"]*\s)?value(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/strong>/gi,
      /<span[^>]*class="(?:[^"]*\s)?title(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/span>[\s\S]{0,400}?<strong[^>]*class="(?:[^"]*\s)?value(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/strong>/gi,
      /<span[^>]*>\s*(Último rendimento|Próximo rendimento)\s*<\/span>[\s\S]{0,400}?<strong[^>]*class="(?:[^"]*\s)?value(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/strong>/gi,
      /<span[^>]*class="(?:[^"]*\s)?sub-title(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/span>\s*<span[^>]*class="(?:[^"]*\s)?sub-value(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/span>/gi,
      /<span[^>]*class="(?:[^"]*\s)?sub-value(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/span>\s*(?:<div>\s*)?<strong[^>]*class="(?:[^"]*\s)?value(?:\s[^"]*)?"[^>]*>([\s\S]*?)<\/strong>/gi,
    ];

    for (const re of pairRegexes) {
      let m: RegExpExecArray | null;
      while ((m = re.exec(html)) !== null) {
        const rawKey = decodeHtmlEntities(
          m[1]!.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ")
        );
        const rawVal = decodeHtmlEntities(
          m[2]!.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ")
        );
        const key = normalize(rawKey);
        if (key && rawVal && !pairs.has(key)) {
          pairs.set(key, rawVal);
        }
      }
    }

    const findNum = (...keys: string[]): number | null => {
      for (const k of keys) {
        const nk = normalize(k);
        for (const [pk, pv] of pairs.entries()) {
          if (pk === nk || pk.startsWith(nk)) {
            const parsed = parsePtBrF64(pv);
            if (parsed != null) return parsed;
          }
        }
      }
      return null;
    };

    const price = findNum("VALOR ATUAL", "COTACAO", "PRECO ATUAL");
    if (price == null || price <= 0) return null;

    let changePercent = 0.0;
    const chgMatch = html.match(
      /arrow_(upward|downward)[\s\S]{0,250}?<b[^>]*>([\d.,+-]+)%?<\/b>/i
    );
    if (chgMatch) {
      const dir = chgMatch[1]!.toLowerCase();
      const val = parsePtBrF64(chgMatch[2]!) ?? 0.0;
      changePercent = dir === "downward" ? -Math.abs(val) : val;
    }

    const previousClose =
      Math.abs(1 + changePercent / 100) > 1e-6
        ? Number((price / (1 + changePercent / 100)).toFixed(2))
        : price;

    const titleMatch = html.match(/<title[^>]*>([\s\S]*?)<\/title>/i);
    let name = pairs.get("NOME PREGAO") ?? sym;
    if (titleMatch?.[1]) {
      const rawTitle = decodeHtmlEntities(
        titleMatch[1]
          .replace(/<[^>]+>/g, "")
          .split("-")[1]
          ?.split("|")[0]
          ?.split(":")[0] ?? ""
      ).trim();
      if (rawTitle) name = rawTitle;
    }

    const baseInd = defaultStockIndicators(sym);
    const dividendYield =
      findNum("D.Y", "DIVIDEND YIELD") ?? baseInd.dividendYield;
    const pl = findNum("P/L") ?? baseInd.pl;
    const pvp = findNum("P/VP") ?? baseInd.pvp;
    const evEbitda = findNum("EV/EBITDA") ?? baseInd.evEbitda;
    const evEbit = findNum("EV/EBIT") ?? baseInd.evEbit;
    const pEbitda = findNum("P/EBITDA") ?? baseInd.pEbitda;
    const pEbit = findNum("P/EBIT") ?? baseInd.pEbit;
    const vpa =
      findNum("VPA", "VAL. PATRIMONIAL P/COTA") ??
      (pvp > 0 ? price / pvp : baseInd.vpa);
    const lpa = findNum("LPA") ?? (pl > 0 ? price / pl : baseInd.lpa);
    const dpa = (dividendYield / 100.0) * price;

    const pSr = findNum("P/SR") ?? baseInd.pSr;
    const pAtivo = findNum("P/ATIVO") ?? baseInd.pAtivo;
    const pCapGiro = findNum("P/CAP. GIRO") ?? baseInd.pCapGiro;
    const pAtivoCircLiq = findNum("P/ATIVO CIRC. LIQ") ?? baseInd.pAtivoCircLiq;
    const pegRatio = findNum("PEG RATIO") ?? baseInd.pegRatio;
    const dividaLiqPl = findNum("DIV. LIQUIDA/PL") ?? baseInd.dividaLiqPl;
    const dividaLiqEbitda =
      findNum("DIV. LIQUIDA/EBITDA") ?? baseInd.dividaLiqEbitda;
    const dividaLiqEbit = findNum("DIV. LIQUIDA/EBIT") ?? baseInd.dividaLiqEbit;
    const liquidezCorrente =
      findNum("LIQ. CORRENTE") ?? baseInd.liquidezCorrente;
    const margemBruta = findNum("M. BRUTA") ?? baseInd.margemBruta;
    const margemEbitda = findNum("M. EBITDA") ?? baseInd.margemEbitda;
    const margemEbit = findNum("M. EBIT") ?? baseInd.margemEbit;
    const margemLiquida = findNum("M. LIQUIDA") ?? baseInd.margemLiquida;
    const roe = findNum("ROE") ?? baseInd.roe;
    const roa = findNum("ROA") ?? baseInd.roa;
    const roic = findNum("ROIC") ?? baseInd.roic;
    const giroAtivos = findNum("GIRO ATIVOS") ?? baseInd.giroAtivos;
    const cagrReceitas5a =
      findNum("CAGR RECEITAS 5 ANOS") ?? baseInd.cagrReceitas5a;
    const cagrLucros5a = findNum("CAGR LUCROS 5 ANOS") ?? baseInd.cagrLucros5a;

    const grahamFairValue =
      lpa > 0.0 && vpa > 0.0 ? Math.sqrt(22.5 * lpa * vpa) : 0.0;
    const bazinCeilingPrice = dpa / 0.06;

    const fiiNavPerShare = findNum("VAL. PATRIMONIAL P/COTA") ?? vpa;
    const fiiLastDividend =
      findNum("ULTIMO RENDIMENTO", "RENDIMENTO MENSAL MEDIO") ?? dpa / 12.0;

    const rawShareholders = findNum(
      "N DE COTISTAS",
      "NUMERO DE COTISTAS",
      "NUMERO COTISTAS"
    );
    const fiiShareholdersCount =
      rawShareholders != null && rawShareholders > 0
        ? Math.round(rawShareholders)
        : null;

    const rawTotalShares = findNum(
      "N DE COTAS",
      "NUMERO DE COTAS",
      "COTAS EMITIDAS"
    );
    const fiiTotalShares =
      rawTotalShares != null && rawTotalShares > 0
        ? Math.round(rawTotalShares)
        : null;

    const rawNetAssetValue = findNum("PATRIMONIO", "PATRIMONIO LIQUIDO");
    const fiiNetAssetValue =
      rawNetAssetValue != null && rawNetAssetValue > 0
        ? rawNetAssetValue
        : null;

    const fiiPhysicalVacancy = findNum("VACANCIA FISICA", "VACANCIA");
    const fiiFinancialVacancy = findNum("VACANCIA FINANCEIRA");
    const fiiAdminFee = findNum(
      "TAXAS ADM",
      "TAXA DE ADMINISTRACAO",
      "TAXA ADMINISTRACAO"
    );

    const fiiSegment = decodeHtmlEntities(
      pairs.get("SEGMENTO") || pairs.get("SEGMENTO ANBIMA") || "Híbrido"
    );

    const sector = decodeHtmlEntities(
      pairs.get("SETOR DE ATUACAO") ?? fiiSegment
    );
    const subsector = decodeHtmlEntities(
      pairs.get("SUBSETOR DE ATUACAO") ?? sector
    );
    const segment = decodeHtmlEntities(
      pairs.get("SEGMENTO DE ATUACAO") ?? fiiSegment
    );

    return {
      ticker: sym,
      category,
      name,
      sector,
      subsector,
      segment,
      price,
      changePercent,
      previousClose,
      min52s: findNum("MIN. 52 SEMANAS") ?? price * 0.8,
      max52s: findNum("MAX. 52 SEMANAS") ?? price * 1.2,
      dividendYield,
      pl,
      pvp,
      evEbitda,
      evEbit,
      pEbitda,
      pEbit,
      vpa,
      lpa,
      dpa,
      pSr,
      pAtivo,
      pCapGiro,
      pAtivoCircLiq,
      pegRatio,
      dividaLiqPl,
      dividaLiqEbitda,
      dividaLiqEbit,
      liquidezCorrente,
      margemBruta,
      margemEbitda,
      margemEbit,
      margemLiquida,
      roe,
      roa,
      roic,
      giroAtivos,
      cagrReceitas5a,
      cagrLucros5a,
      grahamFairValue,
      bazinCeilingPrice,
      fiiNavPerShare,
      fiiLastDividend,
      fiiShareholdersCount,
      fiiTotalShares,
      fiiNetAssetValue,
      fiiSegment,
      fiiPhysicalVacancy,
      fiiFinancialVacancy,
      fiiAdminFee,
    };
  }

  static parseFundsExplorerHtml(html: string): FiiExternalEnrichment | null {
    try {
      const m = html.match(
        /var\s+dataLayer_content\s*=\s*(\{[\s\S]*?\});\s*dataLayer\.push/
      );
      if (!m?.[1]) return null;
      const dl = JSON.parse(m[1]) as {
        pagePostTerms?: { meta?: Record<string, unknown> };
      };
      const meta = dl.pagePostTerms?.meta;
      if (!meta || typeof meta !== "object") return null;

      const toPosNum = (v: unknown): number | null => {
        if (typeof v === "number" && Number.isFinite(v) && v > 0) return v;
        if (typeof v === "string" && v.trim() !== "") {
          const p = parsePtBrF64(v);
          if (p != null && p > 0) return p;
        }
        return null;
      };

      const toNonNegNum = (v: unknown): number | null => {
        if (typeof v === "number" && Number.isFinite(v) && v >= 0) return v;
        if (typeof v === "string" && v.trim() !== "") {
          const p = parsePtBrF64(v);
          if (p != null && p >= 0) return p;
        }
        return null;
      };

      const shareholdersCount = toPosNum(meta.numero_cotista);
      const totalShares = toPosNum(meta.numero_cotas);
      const netAssetValue = toPosNum(meta.patrimonio);

      let physicalVacancy: number | null = null;
      let financialVacancy: number | null = null;
      const vacCount = Number(meta.vacancia ?? 0);
      if (Number.isFinite(vacCount) && vacCount > 0) {
        // Check the latest period (or n-2 only if n-1 is completely blank)
        for (const idx of [vacCount - 1, vacCount - 2]) {
          if (idx < 0) continue;
          const pf = toNonNegNum(meta[`vacancia_${idx}_vacancia_fisica`]);
          const ff = toNonNegNum(meta[`vacancia_${idx}_vacancia_financeira`]);
          if (pf != null || ff != null) {
            physicalVacancy = pf;
            financialVacancy = ff;
            break;
          }
        }
      }

      const adm = toPosNum(meta.taxa_de_adm);
      const gestao = toPosNum(meta.taxa_de_gestao);
      const adminFee =
        adm != null || gestao != null
          ? Number(((adm ?? 0) + (gestao ?? 0)).toFixed(4))
          : null;

      let performanceFee: string | null = null;
      const perf = meta.taxa_de_performance;
      if (typeof perf === "number" && perf > 0) {
        performanceFee = `${perf}% sobre o benchmark`;
      } else if (typeof perf === "string" && perf.trim() !== "") {
        performanceFee = perf.trim();
      }

      const adminName =
        typeof meta.administrador === "string" && meta.administrador.trim()
          ? meta.administrador.trim()
          : null;
      const mandate =
        typeof meta.tipo_anbima === "string" && meta.tipo_anbima.trim()
          ? meta.tipo_anbima.trim()
          : null;

      const setorAtuacao =
        typeof meta.setor_atuacao === "string" ? meta.setor_atuacao.trim() : "";
      const subsetorAtuacao =
        typeof meta.subsetor_atuacao === "string"
          ? meta.subsetor_atuacao.trim()
          : "";
      const segAmbima =
        typeof meta.segmento_ambima === "string"
          ? meta.segmento_ambima.trim()
          : "";
      const segment =
        setorAtuacao.toUpperCase().includes("PAPEL")
          ? "Papéis / Recebíveis"
          : subsetorAtuacao || segAmbima || setorAtuacao || null;

      return {
        shareholdersCount:
          shareholdersCount != null ? Math.round(shareholdersCount) : null,
        totalShares: totalShares != null ? Math.round(totalShares) : null,
        netAssetValue,
        physicalVacancy,
        financialVacancy,
        adminFee,
        performanceFee,
        adminName,
        mandate,
        segment,
      };
    } catch {
      return null;
    }
  }

  static parseInvestidor10Html(html: string): Partial<FiiExternalEnrichment> {
    const out: Partial<FiiExternalEnrichment> = {};
    try {
      const cells = html.matchAll(
        /<span[^>]*class="[^"]*\bname\b[^"]*"[^>]*>([\s\S]*?)<\/span>\s*<div[^>]*class="[^"]*\bvalue\b[^"]*"[^>]*>\s*<span>([\s\S]*?)<\/span>/gi
      );
      let isPapel = false;
      let rawVacancy: number | null = null;

      for (const c of cells) {
        const key = normalize(
          decodeHtmlEntities(c[1]!.replace(/<[^>]+>/g, " ").replace(/\s+/g, " "))
        );
        const val = decodeHtmlEntities(
          c[2]!.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ")
        );
        if (key.includes("TIPO DE FUNDO") && normalize(val).includes("PAPEL")) {
          isPapel = true;
        }
        if (key.includes("TAXA DE ADMINISTRACAO")) {
          const pctMatch = val.match(/(\d+(?:[.,]\d+)?)\s*%/);
          if (pctMatch?.[1]) {
            const fee = parsePtBrF64(pctMatch[1]);
            if (fee != null && fee > 0) out.adminFee = fee;
          }
        } else if (key === "VACANCIA" || key.includes("VACANCIA")) {
          const v = parsePtBrF64(val);
          if (v != null && v >= 0) rawVacancy = v;
        } else if (key.includes("NUMERO DE COTISTAS")) {
          const sc = parsePtBrF64(val);
          if (sc != null && sc > 0) out.shareholdersCount = Math.round(sc);
        } else if (key.includes("COTAS EMITIDAS")) {
          const ts = parsePtBrF64(val);
          if (ts != null && ts > 0) out.totalShares = Math.round(ts);
        }
      }

      if (!isPapel && rawVacancy != null) {
        out.physicalVacancy = rawVacancy;
      }
    } catch {
      // Ignore parse errors
    }
    return out;
  }

  async fetchAsset(ticker: string): Promise<StatusInvestAsset | null> {
    const sym = ticker.trim().toUpperCase();
    const categories = categoryOrderFor(sym);

    let parsed: StatusInvestAsset | null = null;
    for (const cat of categories) {
      const url = `https://statusinvest.com.br/${cat}/${sym.toLowerCase()}`;
      const html = await this.fetchHtml(url);
      if (!html) continue;
      parsed = StatusInvestClient.parseHtml(sym, cat, html);
      if (parsed) break;
    }

    if (!parsed) return null;

    // Enrich FII fields (Vacancy, Admin Fee, Shareholders, Total Shares) from FundsExplorer & Investidor10
    if (parsed.category === "fundos-imobiliarios") {
      const feHtml = await this.fetchHtml(
        `https://www.fundsexplorer.com.br/funds/${sym.toLowerCase()}`
      );
      if (feHtml) {
        const fe = StatusInvestClient.parseFundsExplorerHtml(feHtml);
        if (fe) {
          if (parsed.fiiShareholdersCount == null && fe.shareholdersCount != null) {
            parsed.fiiShareholdersCount = fe.shareholdersCount;
          }
          if (parsed.fiiTotalShares == null && fe.totalShares != null) {
            parsed.fiiTotalShares = fe.totalShares;
          }
          if (parsed.fiiNetAssetValue == null && fe.netAssetValue != null) {
            parsed.fiiNetAssetValue = fe.netAssetValue;
          }
          if (parsed.fiiPhysicalVacancy == null && fe.physicalVacancy != null) {
            parsed.fiiPhysicalVacancy = fe.physicalVacancy;
          }
          if (parsed.fiiFinancialVacancy == null && fe.financialVacancy != null) {
            parsed.fiiFinancialVacancy = fe.financialVacancy;
          }
          if (parsed.fiiAdminFee == null && fe.adminFee != null) {
            parsed.fiiAdminFee = fe.adminFee;
          }
          if (fe.performanceFee) {
            parsed.fiiPerformanceFee = fe.performanceFee;
          }
          if (fe.adminName) {
            parsed.fiiAdminName = fe.adminName;
          }
          if (fe.mandate) {
            parsed.fiiMandate = fe.mandate;
          }
          if (
            fe.segment &&
            (!parsed.fiiSegment ||
              parsed.fiiSegment === "Híbrido" ||
              parsed.fiiSegment === "Outros")
          ) {
            parsed.fiiSegment = fe.segment;
          }
        }
      }

      if (
        parsed.fiiAdminFee == null ||
        parsed.fiiShareholdersCount == null
      ) {
        const i10Html = await this.fetchHtml(
          `https://investidor10.com.br/fiis/${sym.toLowerCase()}/`
        );
        if (i10Html) {
          const i10 = StatusInvestClient.parseInvestidor10Html(i10Html);
          if (parsed.fiiAdminFee == null && i10.adminFee != null) {
            parsed.fiiAdminFee = i10.adminFee;
          }
          if (
            parsed.fiiShareholdersCount == null &&
            i10.shareholdersCount != null
          ) {
            parsed.fiiShareholdersCount = i10.shareholdersCount;
          }
          if (parsed.fiiTotalShares == null && i10.totalShares != null) {
            parsed.fiiTotalShares = i10.totalShares;
          }
          if (
            parsed.fiiPhysicalVacancy == null &&
            i10.physicalVacancy != null
          ) {
            parsed.fiiPhysicalVacancy = i10.physicalVacancy;
          }
        }
      }
    }

    return parsed;
  }

  static deserializeRaw(
    raw: string | Record<string, unknown>
  ): StatusInvestAsset | null {
    try {
      const obj = (
        typeof raw === "string" ? JSON.parse(raw) : raw
      ) as Record<string, unknown>;
      if (!obj || typeof obj !== "object") return null;

      const ticker = String(obj.ticker ?? "").trim().toUpperCase();
      if (!ticker) return null;

      const baseInd = defaultStockIndicators(ticker);
      const baseFii = defaultFiiDetails(ticker);

      const num = (
        camel: string,
        snake: string,
        fallback: number
      ): number => {
        const v = obj[camel] ?? obj[snake];
        if (typeof v === "number" && Number.isFinite(v)) return v;
        if (typeof v === "string") {
          const parsed = Number.parseFloat(v);
          if (Number.isFinite(parsed)) return parsed;
        }
        return fallback;
      };

      const optNum = (
        camel: string,
        snake: string
      ): number | null => {
        const v = obj[camel] ?? obj[snake];
        if (typeof v === "number" && Number.isFinite(v)) return v;
        if (typeof v === "string" && v.trim() !== "") {
          const parsed = Number.parseFloat(v);
          if (Number.isFinite(parsed)) return parsed;
        }
        return null;
      };

      const str = (
        camel: string,
        snake: string,
        fallback: string
      ): string => {
        const v = obj[camel] ?? obj[snake];
        return typeof v === "string" && v.trim()
          ? decodeHtmlEntities(v.trim())
          : fallback;
      };

      const price = num("price", "price", baseInd.price);
      const changePercent = num("changePercent", "change_percent", 0.0);
      const previousClose = num("previousClose", "previous_close", price);
      const dividendYield = num(
        "dividendYield",
        "dividend_yield",
        baseInd.dividendYield
      );
      const pl = num("pl", "pl", baseInd.pl);
      const pvp = num("pvp", "pvp", baseInd.pvp);
      const vpa = num("vpa", "vpa", baseInd.vpa);
      const lpa = num("lpa", "lpa", baseInd.lpa);
      const dpa = num("dpa", "dpa", (dividendYield / 100.0) * price);

      // Detect legacy synthetic fallback values in cached JSON
      let fiiShareholdersCount = optNum(
        "fiiShareholdersCount",
        "fii_shareholders_count"
      );
      let fiiPhysicalVacancy = optNum(
        "fiiPhysicalVacancy",
        "fii_physical_vacancy"
      );
      let fiiFinancialVacancy = optNum(
        "fiiFinancialVacancy",
        "fii_financial_vacancy"
      );
      const fiiAdminFee =
        optNum("fiiAdminFee", "fii_admin_fee") ?? baseFii.adminFeePercent;

      const isLegacyFakeRow =
        fiiShareholdersCount === 150000 &&
        fiiPhysicalVacancy === 3.5 &&
        fiiFinancialVacancy === 3.15;
      if (isLegacyFakeRow) {
        fiiShareholdersCount = baseFii.shareholdersCount;
        fiiPhysicalVacancy = baseFii.physicalVacancyPercent;
        fiiFinancialVacancy = baseFii.financialVacancyPercent;
      }

      return {
        ticker,
        category: str("category", "category", "acoes"),
        name: str("name", "name", ticker),
        sector: str("sector", "sector", "Mercado B3"),
        subsector: str("subsector", "subsector", "Geral"),
        segment: str("segment", "segment", "Mercado à Vista"),
        price,
        changePercent,
        previousClose,
        min52s: num("min52s", "min_52s", price * 0.85),
        max52s: num("max52s", "max_52s", price * 1.15),
        dividendYield,
        pl,
        pvp,
        evEbitda: num("evEbitda", "ev_ebitda", baseInd.evEbitda),
        evEbit: num("evEbit", "ev_ebit", baseInd.evEbit),
        pEbitda: num("pEbitda", "p_ebitda", baseInd.pEbitda),
        pEbit: num("pEbit", "p_ebit", baseInd.pEbit),
        vpa,
        lpa,
        dpa,
        pSr: num("pSr", "p_sr", baseInd.pSr),
        pAtivo: num("pAtivo", "p_ativo", baseInd.pAtivo),
        pCapGiro: num("pCapGiro", "p_cap_giro", baseInd.pCapGiro),
        pAtivoCircLiq: num(
          "pAtivoCircLiq",
          "p_ativo_circ_liq",
          baseInd.pAtivoCircLiq
        ),
        pegRatio: num("pegRatio", "peg_ratio", baseInd.pegRatio),
        dividaLiqPl: num("dividaLiqPl", "divida_liq_pl", baseInd.dividaLiqPl),
        dividaLiqEbitda: num(
          "dividaLiqEbitda",
          "divida_liq_ebitda",
          baseInd.dividaLiqEbitda
        ),
        dividaLiqEbit: num(
          "dividaLiqEbit",
          "divida_liq_ebit",
          baseInd.dividaLiqEbit
        ),
        liquidezCorrente: num(
          "liquidezCorrente",
          "liquidez_corrente",
          baseInd.liquidezCorrente
        ),
        margemBruta: num("margemBruta", "margem_bruta", baseInd.margemBruta),
        margemEbitda: num(
          "margemEbitda",
          "margem_ebitda",
          baseInd.margemEbitda
        ),
        margemEbit: num("margemEbit", "margem_ebit", baseInd.margemEbit),
        margemLiquida: num(
          "margemLiquida",
          "margem_liquida",
          baseInd.margemLiquida
        ),
        roe: num("roe", "roe", baseInd.roe),
        roa: num("roa", "roa", baseInd.roa),
        roic: num("roic", "roic", baseInd.roic),
        giroAtivos: num("giroAtivos", "giro_ativos", baseInd.giroAtivos),
        cagrReceitas5a: num(
          "cagrReceitas5a",
          "cagr_receitas_5a",
          baseInd.cagrReceitas5a
        ),
        cagrLucros5a: num(
          "cagrLucros5a",
          "cagr_lucros_5a",
          baseInd.cagrLucros5a
        ),
        grahamFairValue: num(
          "grahamFairValue",
          "graham_fair_value",
          lpa > 0 && vpa > 0
            ? Math.sqrt(22.5 * lpa * vpa)
            : baseInd.grahamFairValue
        ),
        bazinCeilingPrice: num(
          "bazinCeilingPrice",
          "bazin_ceiling_price",
          dpa / 0.06
        ),
        fiiNavPerShare: num(
          "fiiNavPerShare",
          "fii_nav_per_share",
          vpa > 0 ? vpa : baseFii.navPerShare
        ),
        fiiLastDividend: num(
          "fiiLastDividend",
          "fii_last_dividend",
          baseFii.lastDividend
        ),
        fiiShareholdersCount:
          fiiShareholdersCount != null && fiiShareholdersCount > 0
            ? fiiShareholdersCount
            : baseFii.shareholdersCount,
        fiiTotalShares: optNum("fiiTotalShares", "fii_total_shares"),
        fiiNetAssetValue: optNum("fiiNetAssetValue", "fii_net_asset_value"),
        fiiSegment: str("fiiSegment", "fii_segment", "Híbrido"),
        fiiPhysicalVacancy,
        fiiFinancialVacancy,
        fiiAdminFee,
        fiiPerformanceFee:
          typeof obj.fiiPerformanceFee === "string"
            ? obj.fiiPerformanceFee
            : null,
        fiiAdminName:
          typeof obj.fiiAdminName === "string" ? obj.fiiAdminName : null,
        fiiMandate:
          typeof obj.fiiMandate === "string" ? obj.fiiMandate : null,
      };
    } catch {
      return null;
    }
  }

  static intoStockIndicators(asset: StatusInvestAsset): StockIndicators {
    const base = defaultStockIndicators(asset.ticker);
    const fin = (v: number | null | undefined, fb: number) =>
      typeof v === "number" && Number.isFinite(v) ? v : fb;

    return {
      ...base,
      symbol: asset.ticker,
      price: fin(asset.price, base.price),
      pl: fin(asset.pl, base.pl),
      pvp: fin(asset.pvp, base.pvp),
      evEbitda: fin(asset.evEbitda, base.evEbitda),
      evEbit: fin(asset.evEbit, base.evEbit),
      pEbitda: fin(asset.pEbitda, base.pEbitda),
      pEbit: fin(asset.pEbit, base.pEbit),
      pSr: fin(asset.pSr, base.pSr),
      pAtivo: fin(asset.pAtivo, base.pAtivo),
      pCapGiro: fin(asset.pCapGiro, base.pCapGiro),
      pAtivoCircLiq: fin(asset.pAtivoCircLiq, base.pAtivoCircLiq),
      dividendYield: fin(asset.dividendYield, base.dividendYield),
      pegRatio: fin(asset.pegRatio, base.pegRatio),
      dividaLiqPl: fin(asset.dividaLiqPl, base.dividaLiqPl),
      dividaLiqEbitda: fin(asset.dividaLiqEbitda, base.dividaLiqEbitda),
      dividaLiqEbit: fin(asset.dividaLiqEbit, base.dividaLiqEbit),
      liquidezCorrente: fin(asset.liquidezCorrente, base.liquidezCorrente),
      margemBruta: fin(asset.margemBruta, base.margemBruta),
      margemEbitda: fin(asset.margemEbitda, base.margemEbitda),
      margemEbit: fin(asset.margemEbit, base.margemEbit),
      margemLiquida: fin(asset.margemLiquida, base.margemLiquida),
      giroAtivos: fin(asset.giroAtivos, base.giroAtivos),
      roe: fin(asset.roe, base.roe),
      roic: fin(asset.roic, base.roic),
      roa: fin(asset.roa, base.roa),
      cagrReceitas5a: fin(asset.cagrReceitas5a, base.cagrReceitas5a),
      cagrLucros5a: fin(asset.cagrLucros5a, base.cagrLucros5a),
      lpa: fin(asset.lpa, base.lpa),
      vpa: fin(asset.vpa, base.vpa),
      dpa: fin(asset.dpa, base.dpa),
      grahamFairValue: fin(asset.grahamFairValue, base.grahamFairValue),
      bazinCeilingPrice: fin(asset.bazinCeilingPrice, base.bazinCeilingPrice),
    };
  }

  static intoFiiDetails(asset: StatusInvestAsset): FiiDetails {
    const base = defaultFiiDetails(asset.ticker);
    const fin = (v: number | null | undefined, fb: number) =>
      typeof v === "number" && Number.isFinite(v) ? v : fb;
    const optFin = (
      v: number | null | undefined,
      fb: number | null
    ): number | null =>
      typeof v === "number" && Number.isFinite(v) ? v : fb;

    const navPerShare =
      fin(asset.fiiNavPerShare, 0) > 0
        ? fin(asset.fiiNavPerShare, base.navPerShare)
        : base.navPerShare;
    const totalShares =
      typeof asset.fiiTotalShares === "number" && asset.fiiTotalShares > 0
        ? asset.fiiTotalShares
        : base.totalShares;
    const netAssetValue =
      typeof asset.fiiNetAssetValue === "number" && asset.fiiNetAssetValue > 0
        ? asset.fiiNetAssetValue
        : navPerShare * totalShares;
    const lastDiv =
      fin(asset.fiiLastDividend, 0) > 0
        ? fin(asset.fiiLastDividend, base.lastDividend)
        : base.lastDividend;

    const physVac = optFin(
      asset.fiiPhysicalVacancy,
      base.physicalVacancyPercent
    );
    const finVac = optFin(
      asset.fiiFinancialVacancy,
      base.financialVacancyPercent
    );
    const shareholders =
      typeof asset.fiiShareholdersCount === "number" &&
      asset.fiiShareholdersCount > 0
        ? asset.fiiShareholdersCount
        : base.shareholdersCount;
    const adminFee =
      typeof asset.fiiAdminFee === "number" && asset.fiiAdminFee > 0
        ? asset.fiiAdminFee
        : base.adminFeePercent;
    const dy12 = fin(asset.dividendYield, base.dividendYield12m);

    return {
      ...base,
      symbol: asset.ticker,
      segment: classifyFiiSegment(asset.fiiSegment || asset.segment || ""),
      mandate: asset.fiiMandate || base.mandate,
      admin: asset.fiiAdminName || base.admin,
      navPerShare,
      totalShares,
      netAssetValue,
      physicalVacancyPercent: physVac,
      financialVacancyPercent: finVac,
      shareholdersCount: shareholders,
      adminFeePercent: adminFee,
      performanceFee: asset.fiiPerformanceFee || base.performanceFee,
      lastDividend: lastDiv,
      dividendYield12m: dy12 > 0 ? dy12 : base.dividendYield12m,
    };
  }

  static intoCompanyProfile(asset: StatusInvestAsset): CompanyProfile {
    const base = defaultCompanyProfile(asset.ticker);
    return {
      ...base,
      symbol: asset.ticker,
      name: decodeHtmlEntities(asset.name || base.name),
      sector: asset.sector
        ? marketSectorFromName(asset.sector)
        : base.sector,
      subsector: decodeHtmlEntities(asset.subsector || base.subsector),
      segment: decodeHtmlEntities(asset.segment || base.segment),
    };
  }
}
