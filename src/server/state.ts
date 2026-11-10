import {
  B3_ASSET_CATALOG,
  assetClassFromSymbol,
  createSyntheticCandles,
  createSyntheticOrderBook,
  defaultCompanyProfile,
  defaultFiiDetails,
  defaultStockIndicators,
  findB3Asset,
  type Candle,
  type NewsItem,
  type OrderBook,
  type Tick,
  type TradeOperation,
} from "../core/index.ts";
import {
  multiplesFromAccounts,
  TrendingEngine,
  type StockTrendAnalysis,
} from "../analytics/index.ts";
import { PortfolioTracker, TaxCalculator } from "../portfolio/index.ts";
import { BrokerDatabase } from "../storage/index.ts";
import {
  candidateDfpYears,
  CotahistParser,
  downloadDfpAccounts,
  downloadIpeNews,
  downloadRssNews,
  isB3Session,
  LiveFeedClient,
  loadFeedUrls,
  nextDfpYear,
  QUOTE_POLL_INTERVAL_MS,
  SAMPLE_SINACOR_NOTE,
  SinacorParser,
  StatusInvestClient,
  type StatusInvestAsset,
} from "../ingest/index.ts";
import {
  type AssetSnapshot,
  type HeatmapNode,
  type InitialStatePayload,
  type PortfolioSnapshot,
  type WsServerMessage,
} from "../shared/types.ts";

export const MAX_VISIBLE_ASSETS = 4;

const PRELOAD_TICKERS = [
  "PETR4",
  "VALE3",
  "WEGE3",
  "ITUB4",
  "BBDC4",
  "BBAS3",
  "TAEE11",
  "EGIE3",
  "HGLG11",
  "KNCR11",
  "MXRF11",
  "XPML11",
  "BTLG11",
  "BBSE3",
  "ITSA4",
  "CXSE3",
  "AZZA3",
];

const HEATMAP_TICKERS = [
  "PETR4",
  "VALE3",
  "ITUB4",
  "BBDC4",
  "BBAS3",
  "WEGE3",
  "TAEE11",
  "EGIE3",
  "HGLG11",
  "KNCR11",
  "MXRF11",
  "XPML11",
];

const CORE_FEED_TICKERS = [
  "PETR4",
  "VALE3",
  "ITUB4",
  "BBAS3",
  "WEGE3",
  "BBSE3",
];

const STATUSINVEST_TTL_MS = 60 * 60 * 1000; // 60 minutes

export class BrokerServerState {
  readonly db: BrokerDatabase;
  readonly liveClient = new LiveFeedClient();
  readonly statusInvestClient = new StatusInvestClient();

  visibleTickers: string[] = [];
  selectedTicker = "PETR4";
  isLiveSimRunning = true;

  assets = new Map<string, AssetSnapshot>();
  cvmAccountsCache = new Map<string, [number, [string, number][]]>();
  orderBooks = new Map<string, OrderBook>();
  recentTicks = new Map<string, Tick[]>();
  heatmapNodes: HeatmapNode[] = [];
  statusInvestInFlight = new Set<string>();

  private lastQuotePollMs = 0;
  private lastNewsPollMs = 0;
  private tickCounter = 0;
  private broadcastFn: ((msg: WsServerMessage) => void) | null = null;

  constructor(dbPath = process.env.BROKER_DB_PATH || "broker.db") {
    let database: BrokerDatabase;
    try {
      database = BrokerDatabase.open(dbPath);
    } catch {
      database = BrokerDatabase.inMemory();
    }
    this.db = database;
    this.init();
  }

  setBroadcaster(fn: (msg: WsServerMessage) => void): void {
    this.broadcastFn = fn;
  }

  private broadcast(msg: WsServerMessage): void {
    this.broadcastFn?.(msg);
  }

  private init(): void {
    // 1. Load watchlist
    let userTickers = this.db.getUserTickers();
    if (userTickers.length === 0) {
      userTickers = ["PETR4", "VALE3", "WEGE3", "ITUB4"];
      this.db.setUserTickers(userTickers);
    }
    this.visibleTickers = userTickers.slice(0, MAX_VISIBLE_ASSETS);
    this.selectedTicker = this.visibleTickers[0] ?? "PETR4";

    // 2. Seed sample SINACOR trades if empty
    const existingTrades = this.db.getAllTrades();
    if (existingTrades.length === 0) {
      const sample = SinacorParser.parseTradeLines(
        SAMPLE_SINACOR_NOTE,
        "XP Investimentos"
      );
      for (const t of sample) {
        this.db.insertTrade(t);
      }
    }

    // 3. Preload core + user tickers + cached statusinvest tickers
    const initialTickers = new Set<string>([
      ...PRELOAD_TICKERS,
      ...this.visibleTickers,
    ]);
    const siCached = this.db.loadAllStatusInvestRaw();
    for (const [sym] of siCached) {
      initialTickers.add(sym.toUpperCase());
    }

    for (const sym of initialTickers) {
      this.ensureAssetLoaded(sym);
    }

    // 4. Overlay cached StatusInvest data
    for (const [sym, jsonStr, fetchedAtMs] of siCached) {
      const parsed = StatusInvestClient.deserializeRaw(jsonStr);
      if (parsed) {
        this.applyStatusInvestAsset(sym.toUpperCase(), parsed, fetchedAtMs);
      }
    }

    // 5. Initialize 4x3 Heatmap Nodes
    this.initHeatmapNodes();
  }

  ensureAssetLoaded(rawSymbol: string): AssetSnapshot {
    const found = findB3Asset(rawSymbol);
    const sym = (found?.ticker ?? rawSymbol).trim().toUpperCase();

    const existing = this.assets.get(sym);
    if (existing) return existing;

    const assetClass = found?.assetClass ?? assetClassFromSymbol(sym);
    const profile = defaultCompanyProfile(sym);
    const indicators = defaultStockIndicators(sym);
    const fiiDetails = assetClass === "Fii" ? defaultFiiDetails(sym) : null;

    // Load quotes from DB or generate in-memory synthetic candles
    let candles = this.db.getQuotes(sym, 180);
    if (candles.length < 30) {
      const synthetics = createSyntheticCandles(indicators.price, 180);
      if (candles.length > 0) {
        const latestDb = candles[candles.length - 1]!;
        synthetics[synthetics.length - 1] = latestDb;
      }
      candles = synthetics;
    }

    let marketPrice: number | null = null;
    let marketPreviousClose: number | null = null;
    let marketFetchedAtMs: number | null = null;

    const latestMarket = this.db.latestMarketQuote(sym);
    if (latestMarket) {
      const [mCandle, fetchedAt] = latestMarket;
      marketPrice = mCandle.close;
      marketPreviousClose = mCandle.open > 0 ? mCandle.open : mCandle.close;
      marketFetchedAtMs = fetchedAt;
      indicators.price = mCandle.close;
      if (candles.length > 0) {
        candles[candles.length - 1] = mCandle;
      } else {
        candles.push(mCandle);
      }
    }

    let cvmYear: number | null = null;
    let coreMultiples = null;
    const cvmData = this.db.loadCvmAccounts(sym);
    if (cvmData) {
      const [year, accounts] = cvmData;
      cvmYear = year;
      this.cvmAccountsCache.set(sym, [year, accounts]);
      const refPrice = marketPrice ?? indicators.price;
      coreMultiples = multiplesFromAccounts(refPrice, accounts);
    }

    const snapshot: AssetSnapshot = {
      ticker: sym,
      assetClass,
      profile,
      indicators,
      fiiDetails,
      candles,
      marketPrice,
      marketPreviousClose,
      marketFetchedAtMs,
      cvmYear,
      coreMultiples,
      statusInvestAtMs: null,
      statusInvestLoading: false,
      statusInvestOffline: false,
    };

    this.assets.set(sym, snapshot);
    this.orderBooks.set(
      sym,
      createSyntheticOrderBook(sym, snapshot.indicators.price)
    );
    this.recentTicks.set(sym, []);

    // Check if StatusInvest raw exists in DB
    const siRaw = this.db.getStatusInvestRaw(sym);
    if (siRaw) {
      const parsed = StatusInvestClient.deserializeRaw(siRaw[0]);
      if (parsed) {
        this.applyStatusInvestAsset(sym, parsed, siRaw[1]);
      }
    }

    return snapshot;
  }

  private applyStatusInvestAsset(
    sym: string,
    si: StatusInvestAsset,
    fetchedAtMs: number
  ): void {
    const snap = this.ensureAssetLoaded(sym);
    snap.indicators = StatusInvestClient.intoStockIndicators(si);
    snap.profile = StatusInvestClient.intoCompanyProfile(si);
    if (snap.assetClass === "Fii" || si.category === "fundos-imobiliarios") {
      snap.fiiDetails = StatusInvestClient.intoFiiDetails(si);
    }
    if (si.price > 0) {
      snap.marketPrice = si.price;
      snap.marketPreviousClose =
        si.previousClose > 0 ? si.previousClose : si.price;
      snap.marketFetchedAtMs = fetchedAtMs;
      if (snap.candles.length > 0) {
        const last = snap.candles[snap.candles.length - 1]!;
        last.close = si.price;
        last.adjustedClose = si.price;
        if (si.previousClose > 0) last.open = si.previousClose;
        last.high = Math.max(last.high, si.price);
        last.low = Math.min(last.low, si.price);
      }
      const cvm = this.cvmAccountsCache.get(sym);
      if (cvm) {
        snap.coreMultiples = multiplesFromAccounts(si.price, cvm[1]);
      }
    }
    snap.statusInvestAtMs = fetchedAtMs;
    snap.statusInvestLoading = false;
    snap.statusInvestOffline = false;
  }

  async queryStatusInvest(
    rawTicker: string,
    force = false
  ): Promise<AssetSnapshot> {
    const snap = this.ensureAssetLoaded(rawTicker);
    const sym = snap.ticker;

    if (
      !force &&
      snap.statusInvestAtMs != null &&
      Date.now() - snap.statusInvestAtMs < STATUSINVEST_TTL_MS
    ) {
      return snap;
    }

    if (this.statusInvestInFlight.has(sym)) {
      return snap;
    }

    this.statusInvestInFlight.add(sym);
    snap.statusInvestLoading = true;
    this.broadcast({ type: "asset_updated", asset: snap });

    try {
      const fetched = await this.statusInvestClient.fetchAsset(sym);
      if (fetched) {
        const now = Date.now();
        this.db.saveStatusInvestRaw(sym, JSON.stringify(fetched), now);
        this.applyStatusInvestAsset(sym, fetched, now);
      } else {
        snap.statusInvestLoading = false;
        snap.statusInvestOffline = snap.statusInvestAtMs == null;
      }
    } catch {
      snap.statusInvestLoading = false;
      snap.statusInvestOffline = snap.statusInvestAtMs == null;
    } finally {
      this.statusInvestInFlight.delete(sym);
      this.broadcast({ type: "asset_updated", asset: snap });
    }

    return snap;
  }

  private initHeatmapNodes(): void {
    const cols = 4;
    const rows = 3;
    const cellW = 1.0 / cols;
    const cellH = 1.0 / rows;

    this.heatmapNodes = HEATMAP_TICKERS.map((sym, idx) => {
      const snap = this.ensureAssetLoaded(sym);
      const col = idx % cols;
      const row = Math.floor(idx / cols);
      const price = snap.indicators.price;
      const prev =
        snap.marketPreviousClose ??
        (snap.candles.length >= 2
          ? snap.candles[snap.candles.length - 2]!.close
          : price * 0.99);
      const changePct = prev > 0 ? ((price - prev) / prev) * 100.0 : 0.0;

      return {
        symbol: sym,
        name: snap.profile.name,
        sector: snap.profile.subsector,
        price,
        changePct,
        flashIntensity: 0.0,
        x: col * cellW,
        y: row * cellH,
        w: cellW,
        h: cellH,
      };
    });
  }

  computePortfolioSnapshot(): PortfolioSnapshot {
    const trades = this.db.getAllTrades();
    const tracker = new PortfolioTracker();
    const taxCalc = new TaxCalculator();
    const monthlyReports = taxCalc.computeMonthlyTaxes(trades, tracker);

    const currentPrices = new Map<string, number>();
    for (const [sym, snap] of this.assets.entries()) {
      currentPrices.set(sym, snap.marketPrice ?? snap.indicators.price);
    }
    for (const t of trades) {
      const sym = t.symbol.toUpperCase();
      if (!currentPrices.has(sym)) {
        const snap = this.ensureAssetLoaded(sym);
        currentPrices.set(sym, snap.marketPrice ?? snap.indicators.price);
      }
    }

    const positions = tracker.getPositions(currentPrices);
    const totalCustodyValue = positions.reduce(
      (acc, p) => acc + p.currentValue,
      0.0
    );
    const totalInvested = positions.reduce(
      (acc, p) => acc + p.totalInvested,
      0.0
    );
    const unrealizedPnl = totalCustodyValue - totalInvested;
    const unrealizedPnlPercent =
      totalInvested > 0.0 ? (unrealizedPnl / totalInvested) * 100.0 : 0.0;

    return {
      trades,
      positions,
      monthlyReports,
      totalCustodyValue,
      totalInvested,
      unrealizedPnl,
      unrealizedPnlPercent,
    };
  }

  computeTrending(onlyStocks = false): StockTrendAnalysis[] {
    const out: StockTrendAnalysis[] = [];
    for (const catalogItem of B3_ASSET_CATALOG) {
      if (onlyStocks && catalogItem.assetClass !== "Acao") continue;
      const snap = this.assets.get(catalogItem.ticker);
      const indicators =
        snap?.indicators ?? defaultStockIndicators(catalogItem.ticker);
      const price =
        snap?.marketPrice ?? snap?.indicators.price ?? catalogItem.basePrice;
      const candles = snap?.candles ?? [];
      const coreMultiples = snap?.coreMultiples ?? null;

      out.push(
        TrendingEngine.analyzeStock(
          catalogItem.ticker,
          catalogItem.name,
          catalogItem.sector,
          price,
          candles,
          indicators,
          coreMultiples
        )
      );
    }
    out.sort((a, b) => b.overallScore - a.overallScore);
    return out;
  }

  getInitialStatePayload(): InitialStatePayload {
    const assetsObj: Record<string, AssetSnapshot> = {};
    for (const [sym, snap] of this.assets.entries()) {
      assetsObj[sym] = snap;
    }

    const orderBooksObj: Record<string, OrderBook> = {};
    for (const [sym, ob] of this.orderBooks.entries()) {
      orderBooksObj[sym] = ob;
    }

    const recentTicksObj: Record<string, Tick[]> = {};
    for (const [sym, ticks] of this.recentTicks.entries()) {
      recentTicksObj[sym] = ticks;
    }

    return {
      visibleTickers: this.visibleTickers,
      selectedTicker: this.selectedTicker,
      assets: assetsObj,
      orderBooks: orderBooksObj,
      recentTicks: recentTicksObj,
      heatmapNodes: this.heatmapNodes,
      portfolio: this.computePortfolioSnapshot(),
      recentNews: this.db.searchNews(null, null, null, null, 200),
      trending: this.computeTrending(false),
      catalog: B3_ASSET_CATALOG,
      isLiveSimRunning: this.isLiveSimRunning,
    };
  }

  updateWatchlist(tickers: readonly string[]): string[] {
    const clean: string[] = [];
    const seen = new Set<string>();
    for (const raw of tickers) {
      const found = findB3Asset(raw);
      const sym = (found?.ticker ?? raw).trim().toUpperCase();
      if (sym && !seen.has(sym)) {
        seen.add(sym);
        clean.push(sym);
        this.ensureAssetLoaded(sym);
      }
      if (clean.length >= MAX_VISIBLE_ASSETS) break;
    }

    if (clean.length === 0) {
      clean.push("PETR4");
      this.ensureAssetLoaded("PETR4");
    }

    this.visibleTickers = clean;
    if (!this.visibleTickers.includes(this.selectedTicker)) {
      this.selectedTicker = this.visibleTickers[0]!;
    }
    this.db.setUserTickers(this.visibleTickers);

    const assetsObj: Record<string, AssetSnapshot> = {};
    for (const sym of this.visibleTickers) {
      assetsObj[sym] = this.ensureAssetLoaded(sym);
    }

    this.broadcast({
      type: "watchlist_updated",
      visibleTickers: this.visibleTickers,
      assets: assetsObj,
    });

    // Trigger async market feed check for new tickers
    void this.syncMarketData(false);
    return this.visibleTickers;
  }

  addTrades(newTrades: readonly TradeOperation[]): PortfolioSnapshot {
    for (const t of newTrades) {
      this.db.insertTrade(t);
      this.ensureAssetLoaded(t.symbol);
    }
    const snap = this.computePortfolioSnapshot();
    this.broadcast({ type: "portfolio_updated", portfolio: snap });
    return snap;
  }

  deleteTrade(id: string): PortfolioSnapshot {
    this.db.deleteTrade(id);
    const snap = this.computePortfolioSnapshot();
    this.broadcast({ type: "portfolio_updated", portfolio: snap });
    return snap;
  }

  importCotahist(content: string): { importedCount: number; symbols: string[] } {
    const parsed = CotahistParser.parseCotahistContent(content);
    const bySymbol = new Map<string, Candle[]>();
    for (const [sym, candle] of parsed) {
      const list = bySymbol.get(sym) ?? [];
      list.push(candle);
      bySymbol.set(sym, list);
    }
    for (const [sym, candles] of bySymbol.entries()) {
      this.db.insertQuotes(sym, candles);
      const snap = this.ensureAssetLoaded(sym);
      snap.candles = this.db.getQuotes(sym, 180);
      if (snap.candles.length > 0) {
        const last = snap.candles[snap.candles.length - 1]!;
        snap.indicators.price = last.close;
      }
      this.broadcast({ type: "asset_updated", asset: snap });
    }
    return {
      importedCount: parsed.length,
      symbols: Array.from(bySymbol.keys()),
    };
  }

  stepSimulationTick(): void {
    if (!this.isLiveSimRunning) return;
    this.tickCounter += 1;
    const now = Date.now();

    const activeSymbols = new Set<string>([
      ...this.visibleTickers,
      this.selectedTicker,
    ]);

    const ticksOut: Record<string, Tick> = {};
    const booksOut: Record<string, OrderBook> = {};
    const pricesOut: Record<
      string,
      { price: number; changePct: number; lastCandle: Candle }
    > = {};

    for (const sym of activeSymbols) {
      const snap = this.ensureAssetLoaded(sym);
      const lastPrice = snap.indicators.price;
      const tick = LiveFeedClient.simulateTick(sym, lastPrice, now + this.tickCounter);
      snap.indicators.price = tick.price;

      // Update last candle
      if (snap.candles.length > 0) {
        const lastCandle = snap.candles[snap.candles.length - 1]!;
        lastCandle.close = tick.price;
        lastCandle.adjustedClose = tick.price;
        lastCandle.high = Math.max(lastCandle.high, tick.price);
        lastCandle.low = Math.min(lastCandle.low, tick.price);
        lastCandle.volume += tick.quantity;
        lastCandle.trades += 1;

        const prevClose =
          snap.marketPreviousClose ??
          (lastCandle.open > 0 ? lastCandle.open : tick.price);
        const changePct =
          prevClose > 0 ? ((tick.price - prevClose) / prevClose) * 100.0 : 0.0;

        pricesOut[sym] = {
          price: tick.price,
          changePct,
          lastCandle: { ...lastCandle },
        };
      }

      // Update recent ticks (keep max 25)
      const list = this.recentTicks.get(sym) ?? [];
      list.unshift(tick);
      if (list.length > 25) list.length = 25;
      this.recentTicks.set(sym, list);
      ticksOut[sym] = tick;

      // Update order book around new price
      const ob = createSyntheticOrderBook(sym, tick.price);
      this.orderBooks.set(sym, ob);
      booksOut[sym] = ob;
    }

    // Decay heatmap flash intensity and pulse one node
    for (const node of this.heatmapNodes) {
      node.flashIntensity = Number((node.flashIntensity * 0.82).toFixed(3));
      const snap = this.assets.get(node.symbol);
      if (snap) {
        node.price = snap.indicators.price;
        const prev =
          snap.marketPreviousClose ??
          (snap.candles.length > 0 ? snap.candles[snap.candles.length - 1]!.open : node.price);
        if (prev > 0) {
          node.changePct = ((node.price - prev) / prev) * 100.0;
        }
      }
    }

    if (this.heatmapNodes.length > 0) {
      const pulseIdx = this.tickCounter % this.heatmapNodes.length;
      const pulsed = this.heatmapNodes[pulseIdx]!;
      pulsed.flashIntensity = 1.0;
      const tick = LiveFeedClient.simulateTick(
        pulsed.symbol,
        pulsed.price,
        now + this.tickCounter * 7
      );
      pulsed.price = tick.price;
      const snap = this.assets.get(pulsed.symbol);
      if (snap) {
        snap.indicators.price = tick.price;
        const prev =
          snap.marketPreviousClose ??
          (snap.candles.length > 0
            ? snap.candles[snap.candles.length - 1]!.open
            : tick.price);
        if (prev > 0) {
          pulsed.changePct = ((tick.price - prev) / prev) * 100.0;
        }
      }
    }

    this.broadcast({
      type: "tick_batch",
      ticks: ticksOut,
      orderBooks: booksOut,
      heatmapNodes: this.heatmapNodes,
      prices: pricesOut,
    });
  }

  async syncMarketData(force = false): Promise<void> {
    const now = Date.now();
    const watched = Array.from(
      new Set([...this.visibleTickers, ...CORE_FEED_TICKERS])
    );

    // 1. Quotes poll
    const missingAnyQuote = watched.some(
      (sym) => this.db.latestMarketQuote(sym) == null
    );
    const shouldPollQuotes =
      force ||
      missingAnyQuote ||
      (isB3Session(now) && now - this.lastQuotePollMs >= QUOTE_POLL_INTERVAL_MS);

    if (shouldPollQuotes) {
      this.lastQuotePollMs = now;
      for (const sym of watched) {
        try {
          const candle = await this.liveClient.fetchBrapiQuote(sym);
          const fetchedAt = Date.now();
          this.db.upsertDailyQuote(sym, candle, fetchedAt);
          this.db.pruneDailyQuotes(sym);

          const snap = this.ensureAssetLoaded(sym);
          snap.marketPrice = candle.close;
          snap.marketPreviousClose =
            candle.open > 0 ? candle.open : candle.close;
          snap.marketFetchedAtMs = fetchedAt;
          snap.indicators.price = candle.close;
          if (snap.candles.length > 0) {
            snap.candles[snap.candles.length - 1] = candle;
          } else {
            snap.candles.push(candle);
          }

          const cvm = this.cvmAccountsCache.get(sym);
          if (cvm) {
            snap.coreMultiples = multiplesFromAccounts(candle.close, cvm[1]);
          }
          this.broadcast({ type: "asset_updated", asset: snap });
        } catch {
          // Keep existing quote/local scenario if offline
        }
      }
    }

    // 2. CVM DFP sync if missing
    try {
      const todayYear = new Date().getUTCFullYear();
      const candidates = candidateDfpYears(todayYear);
      const storedYears = new Map<string, number | null>();
      for (const sym of watched) {
        storedYears.set(sym, this.db.cvmLatestYear(sym));
      }
      const targetYear = nextDfpYear(storedYears, candidates, watched);
      if (targetYear != null && force) {
        const extracted = await downloadDfpAccounts(targetYear, watched);
        for (const item of extracted) {
          this.db.replaceCvmYear(
            item.ticker,
            item.year,
            item.referenceDate,
            item.accounts
          );
          this.cvmAccountsCache.set(item.ticker, [item.year, item.accounts]);
          const snap = this.ensureAssetLoaded(item.ticker);
          snap.cvmYear = item.year;
          const refPrice = snap.marketPrice ?? snap.indicators.price;
          snap.coreMultiples = multiplesFromAccounts(refPrice, item.accounts);
          this.broadcast({ type: "asset_updated", asset: snap });
        }
      }
    } catch {
      // Ignore network/CVM errors
    }

    // 3. News poll (RSS + IPE)
    if (force || now - this.lastNewsPollMs >= 30 * 60 * 1000) {
      this.lastNewsPollMs = now;
      await this.syncNews(watched);
    }
  }

  async syncNews(tickers = this.visibleTickers): Promise<NewsItem[]> {
    const watched = Array.from(
      new Set([...tickers, ...this.visibleTickers, ...CORE_FEED_TICKERS])
    );
    try {
      const feedUrls = loadFeedUrls("news_feeds.txt");
      const rssItems = await downloadRssNews(feedUrls, watched);
      if (rssItems.length > 0) {
        this.db.insertNews(rssItems);
      }

      const year = new Date().getUTCFullYear();
      const ipeItems = await downloadIpeNews(year, watched).catch(() => []);
      if (ipeItems.length > 0) {
        this.db.insertNews(ipeItems);
      }
      this.db.markNewsAttempt(Date.now());
    } catch {
      // Ignore offline errors
    }

    const recent = this.db.searchNews(null, null, null, null, 200);
    this.broadcast({ type: "news_updated", recentNews: recent });
    return recent;
  }
}
