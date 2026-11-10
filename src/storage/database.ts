import { Database } from "bun:sqlite";
import { type Candle, brtDayStartMs } from "../core/quote.ts";
import { type TradeOperation } from "../core/tax.ts";
import { type NewsItem } from "../core/news.ts";

export type DailyQuoteWrite = "Inserted" | "Updated" | "Unchanged";

export const MAX_DAILY_QUOTES = 180;

export class BrokerDatabase {
  readonly db: Database;

  constructor(db: Database) {
    this.db = db;
    this.initSchema();
  }

  static open(path: string): BrokerDatabase {
    const db = new Database(path, { create: true });
    db.exec("PRAGMA journal_mode = WAL;");
    db.exec("PRAGMA busy_timeout = 5000;");
    return new BrokerDatabase(db);
  }

  static inMemory(): BrokerDatabase {
    const db = new Database(":memory:");
    return new BrokerDatabase(db);
  }

  close(): void {
    this.db.close();
  }

  private ensureColumn(table: string, column: string, definition: string): void {
    const cols = this.db
      .query<{ name: string }, []>(`PRAGMA table_info(${table})`)
      .all();
    const exists = cols.some((c) => c.name === column);
    if (!exists) {
      this.db.exec(`ALTER TABLE ${table} ADD COLUMN ${column} ${definition}`);
    }
  }

  private initSchema(): void {
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS quotes (
        symbol TEXT NOT NULL,
        timestamp_ms INTEGER NOT NULL,
        open REAL NOT NULL,
        high REAL NOT NULL,
        low REAL NOT NULL,
        close REAL NOT NULL,
        volume REAL NOT NULL,
        trades INTEGER NOT NULL,
        adjusted_close REAL NOT NULL,
        PRIMARY KEY (symbol, timestamp_ms)
      );
      CREATE INDEX IF NOT EXISTS idx_quotes_symbol_time ON quotes(symbol, timestamp_ms);

      CREATE TABLE IF NOT EXISTS trades (
        id TEXT PRIMARY KEY,
        date TEXT NOT NULL,
        symbol TEXT NOT NULL,
        side TEXT NOT NULL,
        kind TEXT NOT NULL,
        quantity REAL NOT NULL,
        unit_price REAL NOT NULL,
        total_gross REAL NOT NULL,
        liquidation_fee REAL NOT NULL,
        registration_fee REAL NOT NULL,
        brokerage_fee REAL NOT NULL,
        iss_tax REAL NOT NULL,
        irrf REAL NOT NULL,
        broker_name TEXT NOT NULL,
        note_number TEXT NOT NULL
      );
      CREATE INDEX IF NOT EXISTS idx_trades_date ON trades(date);

      CREATE TABLE IF NOT EXISTS user_watchlist (
        slot_index INTEGER PRIMARY KEY,
        ticker TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS cvm_accounts (
        ticker TEXT NOT NULL,
        year INTEGER NOT NULL,
        account_code TEXT NOT NULL,
        amount REAL NOT NULL,
        reference_date TEXT NOT NULL,
        PRIMARY KEY (ticker, year, account_code)
      );

      CREATE TABLE IF NOT EXISTS news (
        url TEXT PRIMARY KEY,
        ticker TEXT NOT NULL,
        title TEXT NOT NULL,
        source TEXT NOT NULL,
        published_at_ms INTEGER NOT NULL,
        summary TEXT,
        external_id TEXT
      );
      CREATE INDEX IF NOT EXISTS idx_news_ticker_time ON news(ticker, published_at_ms);

      CREATE TABLE IF NOT EXISTS news_sync (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        attempted_at_ms INTEGER NOT NULL
      );

      CREATE TABLE IF NOT EXISTS statusinvest_assets (
        ticker TEXT PRIMARY KEY,
        json_data TEXT NOT NULL,
        fetched_at_ms INTEGER NOT NULL
      );
    `);

    this.ensureColumn("quotes", "source", "TEXT");
    this.ensureColumn("quotes", "fetched_at_ms", "INTEGER");

    // Clean up legacy synthetic rows without source
    this.db.exec("DELETE FROM quotes WHERE source IS NULL");
  }

  insertQuotes(symbol: string, candles: readonly Candle[]): void {
    const sym = symbol.trim().toUpperCase();
    const stmt = this.db.prepare(`
      INSERT OR REPLACE INTO quotes (
        symbol, timestamp_ms, open, high, low, close, volume, trades, adjusted_close, source
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'imported')
    `);
    const tx = this.db.transaction((rows: readonly Candle[]) => {
      for (const c of rows) {
        stmt.run(
          sym,
          c.timestampMs,
          c.open,
          c.high,
          c.low,
          c.close,
          c.volume,
          c.trades,
          c.adjustedClose
        );
      }
    });
    tx(candles);
  }

  getQuotes(symbol: string, limit = 180): Candle[] {
    const sym = symbol.trim().toUpperCase();
    const rows = this.db
      .query<
        {
          timestamp_ms: number;
          open: number;
          high: number;
          low: number;
          close: number;
          volume: number;
          trades: number;
          adjusted_close: number;
        },
        [string, number]
      >(`
        SELECT timestamp_ms, open, high, low, close, volume, trades, adjusted_close
        FROM (
          SELECT timestamp_ms, open, high, low, close, volume, trades, adjusted_close
          FROM quotes
          WHERE symbol = ?1
          ORDER BY timestamp_ms DESC
          LIMIT ?2
        )
        ORDER BY timestamp_ms ASC
      `)
      .all(sym, limit);

    return rows.map((r) => ({
      timestampMs: Number(r.timestamp_ms),
      open: Number(r.open),
      high: Number(r.high),
      low: Number(r.low),
      close: Number(r.close),
      volume: Number(r.volume),
      trades: Number(r.trades),
      adjustedClose: Number(r.adjusted_close),
    }));
  }

  upsertDailyQuote(
    symbol: string,
    candle: Candle,
    fetchedAtMs: number
  ): DailyQuoteWrite {
    const sym = symbol.trim().toUpperCase();
    const dayMs = brtDayStartMs(candle.timestampMs);

    const existing = this.db
      .query<
        {
          open: number;
          high: number;
          low: number;
          close: number;
          volume: number;
        },
        [string, number]
      >(
        `SELECT open, high, low, close, volume FROM quotes WHERE symbol = ?1 AND timestamp_ms = ?2 AND source = 'market'`
      )
      .get(sym, dayMs);

    if (existing) {
      if (
        Math.abs(existing.close - candle.close) < 1e-6 &&
        Math.abs(existing.volume - candle.volume) < 1e-6 &&
        Math.abs(existing.open - candle.open) < 1e-6
      ) {
        return "Unchanged";
      }
      const newHigh = Math.max(existing.high, candle.high, candle.close);
      const newLow = Math.min(existing.low, candle.low, candle.close);
      this.db
        .prepare(
          `UPDATE quotes SET open = ?1, high = ?2, low = ?3, close = ?4, volume = ?5, adjusted_close = ?4, fetched_at_ms = ?6 WHERE symbol = ?7 AND timestamp_ms = ?8`
        )
        .run(
          candle.open,
          newHigh,
          newLow,
          candle.close,
          candle.volume,
          fetchedAtMs,
          sym,
          dayMs
        );
      return "Updated";
    }

    this.db
      .prepare(
        `INSERT OR REPLACE INTO quotes (
          symbol, timestamp_ms, open, high, low, close, volume, trades, adjusted_close, source, fetched_at_ms
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'market', ?10)`
      )
      .run(
        sym,
        dayMs,
        candle.open,
        candle.high,
        candle.low,
        candle.close,
        candle.volume,
        candle.trades,
        candle.close,
        fetchedAtMs
      );
    return "Inserted";
  }

  latestMarketQuote(symbol: string): [Candle, number] | null {
    const sym = symbol.trim().toUpperCase();
    const row = this.db
      .query<
        {
          timestamp_ms: number;
          open: number;
          high: number;
          low: number;
          close: number;
          volume: number;
          trades: number;
          adjusted_close: number;
          fetched_at_ms: number | null;
        },
        [string]
      >(
        `SELECT timestamp_ms, open, high, low, close, volume, trades, adjusted_close, fetched_at_ms
         FROM quotes
         WHERE symbol = ?1 AND source = 'market'
         ORDER BY timestamp_ms DESC
         LIMIT 1`
      )
      .get(sym);

    if (!row) return null;
    return [
      {
        timestampMs: Number(row.timestamp_ms),
        open: Number(row.open),
        high: Number(row.high),
        low: Number(row.low),
        close: Number(row.close),
        volume: Number(row.volume),
        trades: Number(row.trades),
        adjustedClose: Number(row.adjusted_close),
      },
      row.fetched_at_ms != null ? Number(row.fetched_at_ms) : Number(row.timestamp_ms),
    ];
  }

  pruneDailyQuotes(symbol: string): void {
    const sym = symbol.trim().toUpperCase();
    this.db
      .prepare(
        `DELETE FROM quotes
         WHERE symbol = ?1 AND source = 'market'
           AND timestamp_ms NOT IN (
             SELECT timestamp_ms FROM quotes
             WHERE symbol = ?1 AND source = 'market'
             ORDER BY timestamp_ms DESC
             LIMIT ?2
           )`
      )
      .run(sym, MAX_DAILY_QUOTES);
  }

  cvmLatestYear(ticker: string): number | null {
    const sym = ticker.trim().toUpperCase();
    const row = this.db
      .query<{ max_year: number | null }, [string]>(
        `SELECT MAX(year) AS max_year FROM cvm_accounts WHERE ticker = ?1`
      )
      .get(sym);
    return row?.max_year != null ? Number(row.max_year) : null;
  }

  replaceCvmYear(
    ticker: string,
    year: number,
    referenceDate: string,
    accounts: readonly [string, number][]
  ): void {
    const sym = ticker.trim().toUpperCase();
    const delStmt = this.db.prepare(`DELETE FROM cvm_accounts WHERE ticker = ?1`);
    const insStmt = this.db.prepare(
      `INSERT OR REPLACE INTO cvm_accounts (ticker, year, account_code, amount, reference_date)
       VALUES (?1, ?2, ?3, ?4, ?5)`
    );

    const tx = this.db.transaction(() => {
      delStmt.run(sym);
      for (const [code, amount] of accounts) {
        insStmt.run(sym, year, code, amount, referenceDate);
      }
    });
    tx();
  }

  loadCvmAccounts(ticker: string): [number, [string, number][]] | null {
    const sym = ticker.trim().toUpperCase();
    const year = this.cvmLatestYear(sym);
    if (year == null) return null;

    const rows = this.db
      .query<{ account_code: string; amount: number }, [string, number]>(
        `SELECT account_code, amount FROM cvm_accounts WHERE ticker = ?1 AND year = ?2 ORDER BY account_code ASC`
      )
      .all(sym, year);

    if (rows.length === 0) return null;
    return [
      year,
      rows.map((r) => [r.account_code, Number(r.amount)] as [string, number]),
    ];
  }

  insertTrade(trade: TradeOperation): void {
    const sideStr = trade.side === "Buy" ? "BUY" : "SELL";
    const kindStr = trade.kind === "SwingTrade" ? "SWING" : "DAY";
    this.db
      .prepare(
        `INSERT OR REPLACE INTO trades (
          id, date, symbol, side, kind, quantity, unit_price, total_gross,
          liquidation_fee, registration_fee, brokerage_fee, iss_tax, irrf,
          broker_name, note_number
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)`
      )
      .run(
        trade.id,
        trade.date,
        trade.symbol.trim().toUpperCase(),
        sideStr,
        kindStr,
        trade.quantity,
        trade.unitPrice,
        trade.totalGross,
        trade.liquidationFee,
        trade.registrationFee,
        trade.brokerageFee,
        trade.issTax,
        trade.irrf,
        trade.brokerName,
        trade.noteNumber
      );
  }

  deleteTrade(id: string): void {
    this.db.prepare(`DELETE FROM trades WHERE id = ?1`).run(id);
  }

  getAllTrades(): TradeOperation[] {
    const rows = this.db
      .query<
        {
          id: string;
          date: string;
          symbol: string;
          side: string;
          kind: string;
          quantity: number;
          unit_price: number;
          total_gross: number;
          liquidation_fee: number;
          registration_fee: number;
          brokerage_fee: number;
          iss_tax: number;
          irrf: number;
          broker_name: string;
          note_number: string;
        },
        []
      >(`SELECT * FROM trades ORDER BY date ASC, id ASC`)
      .all();

    return rows.map((r) => ({
      id: r.id,
      date: r.date,
      symbol: r.symbol,
      side: r.side.toUpperCase() === "SELL" ? "Sell" : "Buy",
      kind: r.kind.toUpperCase() === "DAY" ? "DayTrade" : "SwingTrade",
      quantity: Number(r.quantity),
      unitPrice: Number(r.unit_price),
      totalGross: Number(r.total_gross),
      liquidationFee: Number(r.liquidation_fee),
      registrationFee: Number(r.registration_fee),
      brokerageFee: Number(r.brokerage_fee),
      issTax: Number(r.iss_tax),
      irrf: Number(r.irrf),
      brokerName: r.broker_name,
      noteNumber: r.note_number,
    }));
  }

  getUserTickers(): string[] {
    const rows = this.db
      .query<{ ticker: string }, []>(
        `SELECT ticker FROM user_watchlist ORDER BY slot_index ASC`
      )
      .all();
    const out: string[] = [];
    const seen = new Set<string>();
    for (const r of rows) {
      const sym = r.ticker.trim().toUpperCase();
      if (sym && !seen.has(sym)) {
        seen.add(sym);
        out.push(sym);
      }
    }
    return out;
  }

  setUserTickers(tickers: readonly string[]): void {
    const unique: string[] = [];
    const seen = new Set<string>();
    for (const t of tickers) {
      const sym = t.trim().toUpperCase();
      if (sym && !seen.has(sym)) {
        seen.add(sym);
        unique.push(sym);
      }
    }

    const del = this.db.prepare(`DELETE FROM user_watchlist`);
    const ins = this.db.prepare(
      `INSERT INTO user_watchlist (slot_index, ticker) VALUES (?1, ?2)`
    );
    const tx = this.db.transaction(() => {
      del.run();
      unique.forEach((sym, idx) => {
        ins.run(idx, sym);
      });
    });
    tx();
  }

  addUserTicker(ticker: string): string[] {
    const current = this.getUserTickers();
    const sym = ticker.trim().toUpperCase();
    if (sym && !current.includes(sym)) {
      current.push(sym);
      this.setUserTickers(current);
    }
    return this.getUserTickers();
  }

  removeUserTicker(ticker: string): string[] {
    const sym = ticker.trim().toUpperCase();
    const next = this.getUserTickers().filter((t) => t !== sym);
    this.setUserTickers(next);
    return this.getUserTickers();
  }

  insertNews(items: readonly NewsItem[]): number {
    const stmt = this.db.prepare(
      `INSERT OR IGNORE INTO news (
        url, ticker, title, source, published_at_ms, summary, external_id
      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)`
    );
    let inserted = 0;
    const tx = this.db.transaction(() => {
      for (const item of items) {
        const url = item.url.trim();
        const title = item.title.trim();
        const ticker = item.ticker.trim().toUpperCase();
        if (
          !ticker ||
          !title ||
          (!url.startsWith("http://") && !url.startsWith("https://"))
        ) {
          continue;
        }
        const res = stmt.run(
          url,
          ticker,
          title,
          item.source.trim() || "CVM",
          item.publishedAtMs,
          item.summary ?? null,
          item.externalId ?? null
        );
        if (res.changes > 0) inserted += 1;
      }
    });
    tx();
    return inserted;
  }

  searchNews(
    ticker?: string | null,
    text?: string | null,
    fromMs?: number | null,
    toMs?: number | null,
    limit = 200
  ): NewsItem[] {
    const conditions: string[] = [];
    const params: (string | number)[] = [];

    if (ticker && ticker.trim() && ticker.trim().toUpperCase() !== "TODOS") {
      conditions.push(`ticker = ?`);
      params.push(ticker.trim().toUpperCase());
    }
    if (text && text.trim()) {
      conditions.push(`LOWER(title) LIKE ?`);
      params.push(`%${text.trim().toLowerCase()}%`);
    }
    if (fromMs != null) {
      conditions.push(`published_at_ms >= ?`);
      params.push(fromMs);
    }
    if (toMs != null) {
      conditions.push(`published_at_ms <= ?`);
      params.push(toMs);
    }

    const whereClause =
      conditions.length > 0 ? `WHERE ${conditions.join(" AND ")}` : "";
    params.push(limit);

    const rows = this.db
      .query<
        {
          url: string;
          ticker: string;
          title: string;
          source: string;
          published_at_ms: number;
          summary: string | null;
          external_id: string | null;
        },
        (string | number)[]
      >(
        `SELECT url, ticker, title, source, published_at_ms, summary, external_id
         FROM news
         ${whereClause}
         ORDER BY published_at_ms DESC, title ASC
         LIMIT ?`
      )
      .all(...params);

    return rows.map((r) => ({
      url: r.url,
      ticker: r.ticker,
      title: r.title,
      source: r.source,
      publishedAtMs: Number(r.published_at_ms),
      summary: r.summary,
      externalId: r.external_id,
    }));
  }

  markNewsAttempt(attemptedAtMs: number): void {
    this.db
      .prepare(
        `INSERT OR REPLACE INTO news_sync (id, attempted_at_ms) VALUES (1, ?1)`
      )
      .run(attemptedAtMs);
  }

  latestNewsAttempt(): number | null {
    const row = this.db
      .query<{ attempted_at_ms: number }, []>(
        `SELECT attempted_at_ms FROM news_sync WHERE id = 1`
      )
      .get();
    return row ? Number(row.attempted_at_ms) : null;
  }

  saveStatusInvestRaw(ticker: string, jsonData: string, fetchedAtMs: number): void {
    const sym = ticker.trim().toUpperCase();
    this.db
      .prepare(
        `INSERT OR REPLACE INTO statusinvest_assets (ticker, json_data, fetched_at_ms)
         VALUES (?1, ?2, ?3)`
      )
      .run(sym, jsonData, fetchedAtMs);
  }

  getStatusInvestRaw(ticker: string): [string, number] | null {
    const sym = ticker.trim().toUpperCase();
    const row = this.db
      .query<{ json_data: string; fetched_at_ms: number }, [string]>(
        `SELECT json_data, fetched_at_ms FROM statusinvest_assets WHERE ticker = ?1`
      )
      .get(sym);
    if (!row) return null;
    return [row.json_data, Number(row.fetched_at_ms)];
  }

  loadAllStatusInvestRaw(): [string, string, number][] {
    const rows = this.db
      .query<{ ticker: string; json_data: string; fetched_at_ms: number }, []>(
        `SELECT ticker, json_data, fetched_at_ms FROM statusinvest_assets`
      )
      .all();
    return rows.map((r) => [r.ticker, r.json_data, Number(r.fetched_at_ms)]);
  }
}
