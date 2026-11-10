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
  fiiShareholdersCount: number;
  fiiSegment: string;
  fiiPhysicalVacancy: number;
  fiiFinancialVacancy: number;
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
  const cleaned = raw
    .replace(/R\$/gi, "")
    .replace(/%/g, "")
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
  const s = normalize(raw);
  if (s.includes("LOGIST") || s.includes("GALP")) return "Logistica";
  if (s.includes("LAJE") || s.includes("ESCRIT")) return "LajesCorporativas";
  if (s.includes("SHOP")) return "Shoppings";
  if (s.includes("PAPEL") || s.includes("RECEB") || s.includes("TITULO") || s.includes("CRI"))
    return "PapeisCRIs";
  if (s.includes("FOF") || s.includes("FUNDO DE FUNDO")) return "FoF";
  if (s.includes("AGRO") || s.includes("FIAGRO")) return "Agro";
  if (s.includes("EDUC") || s.includes("HOSP")) return "EducacionalHospitalar";
  if (s.includes("RESID")) return "Residencial";
  if (s.includes("HIBR") || s.includes("RENDA URB")) return "Hibrido";
  return "Hibrido";
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
          Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
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
      /<h3[^>]*class="[^"]*title[^"]*"[^>]*>([\s\S]*?)<\/h3>[\s\S]*?<strong[^>]*class="[^"]*value[^"]*"[^>]*>([\s\S]*?)<\/strong>/gi,
      /<span[^>]*class="[^"]*title[^"]*"[^>]*>([\s\S]*?)<\/span>[\s\S]*?<strong[^>]*class="[^"]*value[^"]*"[^>]*>([\s\S]*?)<\/strong>/gi,
      /<span[^>]*class="[^"]*sub-value[^"]*"[^>]*>([\s\S]*?)<\/span>[\s\S]*?<strong[^>]*class="[^"]*value[^"]*"[^>]*>([\s\S]*?)<\/strong>/gi,
    ];

    for (const re of pairRegexes) {
      let m: RegExpExecArray | null;
      while ((m = re.exec(html)) !== null) {
        const rawKey = m[1]!
          .replace(/<[^>]+>/g, " ")
          .replace(/\s+/g, " ")
          .trim();
        const rawVal = m[2]!
          .replace(/<[^>]+>/g, " ")
          .replace(/\s+/g, " ")
          .trim();
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

    const price = findNum("VALOR ATUAL", "COTAÇÃO", "PRECO ATUAL");
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
    let name = sym;
    if (titleMatch?.[1]) {
      const rawTitle = titleMatch[1]
        .replace(/<[^>]+>/g, "")
        .split("-")[1]
        ?.split("|")[0]
        ?.trim();
      if (rawTitle) name = rawTitle;
    }

    const baseInd = defaultStockIndicators(sym);
    const dividendYield = findNum("D.Y", "DIVIDEND YIELD") ?? baseInd.dividendYield;
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
    const liquidezCorrente = findNum("LIQ. CORRENTE") ?? baseInd.liquidezCorrente;
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
    const fiiShareholdersCount = Math.round(
      findNum("N DE COTISTAS", "NUMERO DE COTISTAS") ?? 150_000
    );
    const fiiPhysicalVacancy = findNum("VACANCIA") ?? 3.5;
    const fiiFinancialVacancy = fiiPhysicalVacancy * 0.9;
    const fiiSegment = pairs.get("SEGMENTO") ?? "Híbrido";

    const sector = pairs.get("SETOR DE ATUACAO") ?? fiiSegment;
    const subsector = pairs.get("SUBSETOR DE ATUACAO") ?? sector;
    const segment = pairs.get("SEGMENTO DE ATUACAO") ?? fiiSegment;

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
      fiiSegment,
      fiiPhysicalVacancy,
      fiiFinancialVacancy,
    };
  }

  async fetchAsset(ticker: string): Promise<StatusInvestAsset | null> {
    const sym = ticker.trim().toUpperCase();
    const categories = categoryOrderFor(sym);

    for (const cat of categories) {
      const url = `https://statusinvest.com.br/${cat}/${sym.toLowerCase()}`;
      const html = await this.fetchHtml(url);
      if (!html) continue;
      const parsed = StatusInvestClient.parseHtml(sym, cat, html);
      if (parsed) return parsed;
    }
    return null;
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

      const str = (
        camel: string,
        snake: string,
        fallback: string
      ): string => {
        const v = obj[camel] ?? obj[snake];
        return typeof v === "string" && v.trim() ? v.trim() : fallback;
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
          lpa > 0 && vpa > 0 ? Math.sqrt(22.5 * lpa * vpa) : baseInd.grahamFairValue
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
        fiiShareholdersCount: num(
          "fiiShareholdersCount",
          "fii_shareholders_count",
          baseFii.shareholdersCount
        ),
        fiiSegment: str("fiiSegment", "fii_segment", "Híbrido"),
        fiiPhysicalVacancy: num(
          "fiiPhysicalVacancy",
          "fii_physical_vacancy",
          baseFii.physicalVacancyPercent
        ),
        fiiFinancialVacancy: num(
          "fiiFinancialVacancy",
          "fii_financial_vacancy",
          baseFii.financialVacancyPercent
        ),
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

    const navPerShare =
      fin(asset.fiiNavPerShare, 0) > 0
        ? fin(asset.fiiNavPerShare, base.navPerShare)
        : base.navPerShare;
    const lastDiv =
      fin(asset.fiiLastDividend, 0) > 0
        ? fin(asset.fiiLastDividend, base.lastDividend)
        : base.lastDividend;
    const physVac = fin(asset.fiiPhysicalVacancy, base.physicalVacancyPercent);
    const finVac = fin(asset.fiiFinancialVacancy, physVac * 0.9);
    const dy12 = fin(asset.dividendYield, base.dividendYield12m);

    return {
      ...base,
      symbol: asset.ticker,
      segment: classifyFiiSegment(asset.fiiSegment || asset.segment || ""),
      navPerShare,
      netAssetValue: navPerShare * base.totalShares,
      physicalVacancyPercent: physVac,
      financialVacancyPercent: finVac,
      shareholdersCount:
        fin(asset.fiiShareholdersCount, 0) > 0
          ? fin(asset.fiiShareholdersCount, base.shareholdersCount)
          : base.shareholdersCount,
      lastDividend: lastDiv,
      dividendYield12m: dy12 > 0 ? dy12 : base.dividendYield12m,
    };
  }

  static intoCompanyProfile(asset: StatusInvestAsset): CompanyProfile {
    const base = defaultCompanyProfile(asset.ticker);
    return {
      ...base,
      symbol: asset.ticker,
      name: asset.name || base.name,
      sector: asset.sector
        ? marketSectorFromName(asset.sector)
        : base.sector,
      subsector: asset.subsector || base.subsector,
      segment: asset.segment || base.segment,
    };
  }
}
