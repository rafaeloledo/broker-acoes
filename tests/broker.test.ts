import { describe, expect, test } from "bun:test";
import {
  assetClassFromSymbol,
  findB3Asset,
  searchB3Assets,
  getAllB3Assets,
  defaultCompanyProfile,
  defaultStockIndicators,
  defaultFiiDetails,
  brtDayStartMs,
  brtHourMinute,
  dueDateFor,
  newsForReader,
  priceCaption,
  fundamentalsCaption,
  changePercent,
  formatMetric,
  FinancialPeriod,
  cvmCodes,
  type TradeOperation,
  type NewsItem,
} from "../src/core/index.ts";
import {
  coreMultiples,
  multiplesFromAccounts,
  TtmAggregator,
  ValuationCalculator,
  ScreenerEngine,
  calculateSma,
  calculateRsi,
  calculateBollingerBands,
  calculateMomentumPct,
  TrendingEngine,
} from "../src/analytics/index.ts";
import { PortfolioTracker, TaxCalculator } from "../src/portfolio/index.ts";
import { BrokerDatabase } from "../src/storage/index.ts";
import {
  issuerForTicker,
  tickersForCnpj,
  LiveFeedClient,
  CvmIngestor,
  filterStatementCsv,
  filterCapitalCsv,
  filterIpeCsv,
  filterRss,
  StatusInvestClient,
  SinacorParser,
  SAMPLE_SINACOR_NOTE,
  CotahistParser,
} from "../src/ingest/index.ts";

describe("b3-core: asset catalog, tickers & presentation", () => {
  test("catalog has 187 assets and handles aliases like ARZZ3/SOMA3 -> AZZA3", () => {
    const all = getAllB3Assets();
    expect(all.length).toBe(187);

    const petr4 = findB3Asset("petr4");
    expect(petr4?.ticker).toBe("PETR4");
    expect(petr4?.basePrice).toBe(52.71);

    const azza = findB3Asset("ARZZ3");
    expect(azza?.ticker).toBe("AZZA3");
    expect(findB3Asset("SOMA3")?.ticker).toBe("AZZA3");

    // Prefix fallback e.g. BBSE4 -> BBSE3
    const searchRes = searchB3Assets("BBSE4");
    expect(searchRes.some((a) => a.ticker === "BBSE3")).toBe(true);
  });

  test("assetClassFromSymbol classifies stocks, FIIs, ETFs, BDRs, and indices", () => {
    expect(assetClassFromSymbol("PETR4")).toBe("Acao");
    expect(assetClassFromSymbol("HGLG11")).toBe("Fii");
    expect(assetClassFromSymbol("BOVA11")).toBe("Etf");
    expect(assetClassFromSymbol("NVDC34")).toBe("Bdr");
    expect(assetClassFromSymbol("IBOV")).toBe("Indice");
  });

  test("BRT time helpers and presentation captions", () => {
    // 2026-10-30 15:30:00 UTC -> 12:30 BRT
    const ts = Date.parse("2026-10-30T15:30:00Z");
    const [h, m] = brtHourMinute(ts);
    expect(h).toBe(12);
    expect(m).toBe(30);
    expect(priceCaption(ts)).toBe("Mercado · 12:30");
    expect(priceCaption(null)).toBe("Cotação local");
    expect(fundamentalsCaption(2025)).toBe("CVM 2025");
    expect(fundamentalsCaption(null)).toBe("Cenário local");
    expect(changePercent(110, 100)).toBeCloseTo(10.0, 6);

    const dayStart = brtDayStartMs(ts);
    expect(brtHourMinute(dayStart)).toEqual([0, 0]);
  });

  test("newsForReader filters, sorts, and truncates news items", () => {
    const items: NewsItem[] = [
      {
        ticker: "PETR4",
        title: "Dividendo extraordinário",
        url: "https://example.com/1",
        source: "CVM",
        publishedAtMs: 1000,
        summary: "A".repeat(700),
        externalId: null,
      },
      {
        ticker: "VALE3",
        title: "Produção de minério",
        url: "https://example.com/2",
        source: "RSS",
        publishedAtMs: 2000,
        summary: "Resumo curto",
        externalId: null,
      },
    ];
    const briefs = newsForReader(items, ["PETR4"], 8, 100);
    expect(briefs.length).toBe(1);
    expect(briefs[0]!.ticker).toBe("PETR4");
    expect(briefs[0]!.text.endsWith("…")).toBe(true);
    expect(Array.from(briefs[0]!.text).length).toBe(101);
  });
});

describe("b3-analytics: valuation, TTM, screener & trending", () => {
  test("coreMultiples computes Graham and Bazin accurately", () => {
    const multiples = coreMultiples(20.0, 5.0, 30.0, -5.0, 1.0);
    expect(multiples.grahamFairValue).toBeCloseTo(Math.sqrt(22.5 * 5.0 * 30.0), 9);
    expect(multiples.bazinCeilingPrice).toBeCloseTo(5.0 / 0.06, 9);
    expect(multiples.pl).toBeCloseTo(4.0, 9);
    expect(multiples.pvp).toBeCloseTo(20.0 / 30.0, 9);
    expect(multiples.dividendYield).toBeCloseTo(25.0, 9);
    expect(formatMetric(multiples, "Pl")).toBe("4.00");
  });

  test("TtmAggregator sums flow accounts and keeps latest balance sheet accounts", () => {
    const q1 = new FinancialPeriod(2025, "Q1", "2025-01-01", "2025-03-31", {
      [cvmCodes.PATRIMONIO_LIQUIDO]: 100,
      [cvmCodes.LUCRO_LIQUIDO_CONSOLIDADO]: 10,
    });
    const q2 = new FinancialPeriod(2025, "Q2", "2025-04-01", "2025-06-30", {
      [cvmCodes.PATRIMONIO_LIQUIDO]: 110,
      [cvmCodes.LUCRO_LIQUIDO_CONSOLIDADO]: 12,
    });
    const q3 = new FinancialPeriod(2025, "Q3", "2025-07-01", "2025-09-30", {
      [cvmCodes.PATRIMONIO_LIQUIDO]: 120,
      [cvmCodes.LUCRO_LIQUIDO_CONSOLIDADO]: 14,
    });
    const q4 = new FinancialPeriod(2025, "Q4", "2025-10-01", "2025-12-31", {
      [cvmCodes.PATRIMONIO_LIQUIDO]: 150,
      [cvmCodes.LUCRO_LIQUIDO_CONSOLIDADO]: 16,
    });

    const ttm = TtmAggregator.aggregateQuarters([q1, q2, q3, q4])!;
    expect(ttm.kind).toBe("TTM");
    expect(ttm.patrimonioLiquido()).toBe(150);
    expect(ttm.lucroLiquido()).toBe(52);
  });

  test("ScreenerEngine filters and ranks Magic Formula", () => {
    const items = ["PETR4", "VALE3", "WEGE3", "ITUB4"].map((s) =>
      defaultStockIndicators(s)
    );
    const filtered = ScreenerEngine.filter(items, { minDy: 5.0, maxPl: 10.0 });
    expect(filtered.some((i) => i.symbol === "PETR4")).toBe(true);
    expect(filtered.some((i) => i.symbol === "WEGE3")).toBe(false);

    const magic = ScreenerEngine.magicFormula(items);
    expect(magic.length).toBeGreaterThan(0);
  });

  test("TrendingEngine SMA, RSI, Bollinger Bands & composite stock analysis", () => {
    expect(calculateSma([10, 11, 12, 13, 14], 3)).toBeCloseTo(13.0, 9);
    expect(calculateSma([10, 11, 12, 13, 14], 5)).toBeCloseTo(12.0, 9);
    expect(calculateSma([10, 11], 10)).toBeNull();

    const upPrices = Array.from({ length: 20 }, (_, i) => 10 + i);
    expect(calculateRsi(upPrices, 14)!).toBeGreaterThan(90);

    const bbPrices = [...Array.from({ length: 20 }, () => 20), 22];
    const [upper, middle, lower, pctB] = calculateBollingerBands(bbPrices, 20, 2.0)!;
    expect(upper).toBeGreaterThan(middle);
    expect(middle).toBeGreaterThan(lower);
    expect(pctB).toBeGreaterThan(0);
    expect(calculateMomentumPct(upPrices, 10)!).toBeGreaterThan(0);

    const ind = defaultStockIndicators("PETR4");
    const analysis = TrendingEngine.analyzeStock(
      "PETR4",
      "Petrobras PN",
      "Petróleo e Gás",
      36.4,
      [],
      ind,
      null
    );
    expect(analysis.ticker).toBe("PETR4");
    expect(analysis.overallScore).toBeGreaterThan(0);
    expect(analysis.overallScore).toBeLessThanOrEqual(100);
    expect(analysis.buyCatalysts.length).toBeGreaterThan(0);
  });
});

describe("b3-portfolio: average cost (PM) & Brazilian tax / DARF 6015", () => {
  test("PortfolioTracker & TaxCalculator handle R$ 20k exemption and taxable months", () => {
    const tracker = new PortfolioTracker();
    const taxCalc = new TaxCalculator();

    const trades: TradeOperation[] = [
      {
        id: "T1",
        date: "2026-01-05",
        symbol: "PETR4",
        side: "Buy",
        kind: "SwingTrade",
        quantity: 1000,
        unitPrice: 30.0,
        totalGross: 30000.0,
        liquidationFee: 0,
        registrationFee: 0,
        brokerageFee: 0,
        issTax: 0,
        irrf: 0,
        brokerName: "XP",
        noteNumber: "1",
      },
      // Sell R$ 15,000 (< R$ 20,000 -> exempt!)
      {
        id: "T2",
        date: "2026-01-15",
        symbol: "PETR4",
        side: "Sell",
        kind: "SwingTrade",
        quantity: 400,
        unitPrice: 37.5,
        totalGross: 15000.0,
        liquidationFee: 0,
        registrationFee: 0,
        brokerageFee: 0,
        issTax: 0,
        irrf: 0.75,
        brokerName: "XP",
        noteNumber: "2",
      },
      // Next month: Sell R$ 25,000 (> R$ 20,000 -> taxable 15%!)
      {
        id: "T3",
        date: "2026-02-10",
        symbol: "PETR4",
        side: "Sell",
        kind: "SwingTrade",
        quantity: 500,
        unitPrice: 50.0,
        totalGross: 25000.0,
        liquidationFee: 0,
        registrationFee: 0,
        brokerageFee: 0,
        issTax: 0,
        irrf: 1.25,
        brokerName: "XP",
        noteNumber: "3",
      },
    ];

    const reports = taxCalc.computeMonthlyTaxes(trades, tracker);
    expect(reports.length).toBe(2);

    // Jan 2026: exempt
    expect(reports[0]!.isAcoesExempt).toBe(true);
    expect(reports[0]!.taxDueAcoesSwing).toBe(0);
    expect(reports[0]!.totalDarfDue).toBe(0);

    // Feb 2026: profit = 25000 - (500 * 30) = 10000 -> 15% = 1500 - 1.25 IRRF = 1498.75
    expect(reports[1]!.isAcoesExempt).toBe(false);
    expect(reports[1]!.netResultAcoesSwing).toBeCloseTo(10000.0, 5);
    expect(reports[1]!.taxDueAcoesSwing).toBeCloseTo(1500.0, 5);
    expect(reports[1]!.totalDarfDue).toBeCloseTo(1498.75, 5);

    // Remaining position: 100 shares @ PM 30.0
    const positions = tracker.getPositions({ PETR4: 45.0 });
    expect(positions.length).toBe(1);
    expect(positions[0]!.quantity).toBe(100);
    expect(positions[0]!.averagePrice).toBeCloseTo(30.0, 6);
    expect(dueDateFor(2026, 1)).toBe("2026-02-27");
  });
});

describe("b3-storage & b3-ingest: SQLite DB and parsers", () => {
  test("in-memory SQLite database stores watchlist, quotes, CVM accounts, trades, and news", () => {
    const db = BrokerDatabase.inMemory();
    db.setUserTickers(["petr4", "vale3", "PETR4"]);
    expect(db.getUserTickers()).toEqual(["PETR4", "VALE3"]);

    db.replaceCvmYear("PETR4", 2025, "2025-12-31", [
      ["3.11", 100_000],
      ["2.03", 500_000],
      ["6.03.05", -50_000],
      ["SHARES", 10_000],
    ]);
    const loaded = db.loadCvmAccounts("PETR4");
    expect(loaded?.[0]).toBe(2025);
    const mult = multiplesFromAccounts(20.0, loaded![1]);
    expect(mult.lpa).toBeCloseTo(10.0, 6);
    expect(mult.vpa).toBeCloseTo(50.0, 6);
    expect(mult.dpa).toBeCloseTo(5.0, 6);

    const parsedTrades = SinacorParser.parseTradeLines(SAMPLE_SINACOR_NOTE);
    expect(parsedTrades.length).toBe(6);
    for (const t of parsedTrades) {
      db.insertTrade(t);
    }
    expect(db.getAllTrades().length).toBe(6);
    db.close();
  });

  test("existing broker.db file is readable and preserves stored records", () => {
    const db = BrokerDatabase.open("broker.db");
    const trades = db.getAllTrades();
    expect(trades.length).toBeGreaterThanOrEqual(6);
    const petrCvm = db.loadCvmAccounts("PETR4");
    expect(petrCvm?.[0]).toBe(2025);
    db.close();
  });

  test("CVM statement, capital, IPE, RSS, StatusInvest, and COTAHIST parsers", () => {
    expect(issuerForTicker("PETR4")?.cnpjDigits).toBe("33000167000101");
    expect(tickersForCnpj("33.000.167/0001-01")).toContain("PETR4");

    const rssXml = `
      <rss><channel>
        <item>
          <title>Petrobras anuncia resultados do trimestre</title>
          <link>https://www.infomoney.com.br/mercados/petrobras-resultados</link>
          <description>Lucro líquido da companhia superou expectativas.</description>
          <pubDate>Fri, 30 Oct 2026 14:00:00 GMT</pubDate>
        </item>
      </channel></rss>
    `;
    const rssNews = filterRss(rssXml, ["PETR4"]);
    expect(rssNews.length).toBe(1);
    expect(rssNews[0]!.ticker).toBe("PETR4");

    const sampleHtml = `
      <html><head><title>PETR4 - PETROBRAS - Status Invest</title></head>
      <body>
        ${"x".repeat(3100)}
        <h3 class="title">Valor atual</h3><strong class="value">R$ 52,71</strong>
        <h3 class="title">D.Y</h3><strong class="value">14,80%</strong>
        <h3 class="title">P/L</h3><strong class="value">4,85</strong>
        <h3 class="title">P/VP</h3><strong class="value">1,15</strong>
      </body></html>
    `;
    const si = StatusInvestClient.parseHtml("PETR4", "acoes", sampleHtml);
    expect(si).not.toBeNull();
    expect(si!.price).toBeCloseTo(52.71, 2);
    expect(si!.dividendYield).toBeCloseTo(14.8, 2);

    const tick = LiveFeedClient.simulateTick("PETR4", 52.71, 1793390000000);
    expect(tick.price).toBeGreaterThan(0);
    expect(tick.quantity).toBeGreaterThan(0);

    // ValuationCalculator test
    const prof = defaultCompanyProfile("PETR4");
    const period = CvmIngestor.buildFinancialPeriod(2025, "Annual", []);
    const computed = ValuationCalculator.computeIndicators(prof, period, 52.71, 5.38);
    expect(computed.symbol).toBe("PETR4");
    expect(defaultFiiDetails("HGLG11").segment).toBe("Logistica");
  });
});

describe("web server: Bun HTTP API, responsive frontend bundle & tick simulation", () => {
  test("serves responsive HTML, bundled JS/CSS, and full REST API", async () => {
    const { createBrokerServer } = await import("../src/server/index.ts");
    const running = await createBrokerServer({
      port: 0,
      dbPath: ":memory:",
      enableIntervals: false,
    });
    const baseUrl = `http://localhost:${running.server.port}`;

    try {
      // 1. HTML & responsive viewport check
      const htmlRes = await fetch(`${baseUrl}/`);
      expect(htmlRes.status).toBe(200);
      const html = await htmlRes.text();
      expect(html).toContain("viewport-fit=cover");
      expect(html).toContain("/assets/main.js");

      // 2. Bundled JS & CSS check
      const jsRes = await fetch(`${baseUrl}/assets/main.js`);
      expect(jsRes.status).toBe(200);
      const cssRes = await fetch(`${baseUrl}/assets/styles.css`);
      expect(cssRes.status).toBe(200);

      // 3. Initial state endpoint
      const stateRes = await fetch(`${baseUrl}/api/state`);
      expect(stateRes.status).toBe(200);
      const stateJson = await stateRes.json();
      expect(stateJson.visibleTickers.length).toBeGreaterThan(0);
      expect(stateJson.catalog.length).toBe(187);
      expect(stateJson.trending.length).toBe(187);
      expect(stateJson.heatmapNodes.length).toBe(12);
      expect(stateJson.portfolio.trades.length).toBe(6);

      // 4. Watchlist update
      const wlRes = await fetch(`${baseUrl}/api/watchlist`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tickers: ["BBAS3", "TAEE11", "HGLG11"] }),
      });
      const wlJson = await wlRes.json();
      expect(wlJson.visibleTickers).toEqual(["BBAS3", "TAEE11", "HGLG11"]);

      // 5. Simulation step
      running.state.stepSimulationTick();
      expect(running.state.recentTicks.get("BBAS3")?.length).toBe(1);
    } finally {
      running.stop();
    }
  });

  test("loads existing broker.db with legacy snake_case statusinvest_assets and computes /api/state cleanly", async () => {
    const { createBrokerServer } = await import("../src/server/index.ts");
    const running = await createBrokerServer({
      port: 0,
      dbPath: "broker.db",
      enableIntervals: false,
    });
    const baseUrl = `http://localhost:${running.server.port}`;

    try {
      const stateRes = await fetch(`${baseUrl}/api/state`);
      expect(stateRes.status).toBe(200);
      const stateJson = await stateRes.json();
      expect(stateJson.trending.length).toBe(187);

      // Verify KNCR11 (cached in statusinvest_assets with snake_case keys and null vacancy)
      const kncr = stateJson.assets["KNCR11"];
      expect(kncr).toBeDefined();
      expect(kncr.indicators.dividendYield).toBeCloseTo(13.04, 2);
      expect(typeof kncr.fiiDetails.physicalVacancyPercent).toBe("number");
    } finally {
      running.stop();
    }
  });
});

