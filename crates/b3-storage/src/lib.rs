use b3_core::news::NewsItem;
use b3_core::quote::Candle;
use b3_core::tax::{OperationKind, OperationSide, TradeOperation};
use chrono::NaiveDate;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DailyQuoteWrite {
    Inserted,
    Updated,
    Unchanged,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    pub fn in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            r#"
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
            "#,
        )?;
        Self::ensure_column(&conn, "quotes", "source", "ALTER TABLE quotes ADD COLUMN source TEXT")?;
        Self::ensure_column(
            &conn,
            "quotes",
            "fetched_at_ms",
            "ALTER TABLE quotes ADD COLUMN fetched_at_ms INTEGER",
        )?;
        Ok(())
    }

    fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) -> anyhow::Result<()> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let names = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for name in names {
            if name? == column {
                return Ok(());
            }
        }
        conn.execute(ddl, [])?;
        Ok(())
    }

    pub fn insert_quotes(&self, symbol: &str, candles: &[Candle]) -> anyhow::Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO quotes (symbol, timestamp_ms, open, high, low, close, volume, trades, adjusted_close)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            )?;
            for c in candles {
                stmt.execute(params![
                    symbol,
                    c.timestamp_ms,
                    c.open,
                    c.high,
                    c.low,
                    c.close,
                    c.volume,
                    c.trades,
                    c.adjusted_close,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_quotes(&self, symbol: &str, limit: usize) -> anyhow::Result<Vec<Candle>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT timestamp_ms, open, high, low, close, volume, trades, adjusted_close
             FROM quotes WHERE symbol = ?1 ORDER BY timestamp_ms ASC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![symbol, limit as i64], |row| {
            Ok(Candle {
                timestamp_ms: row.get(0)?,
                open: row.get(1)?,
                high: row.get(2)?,
                low: row.get(3)?,
                close: row.get(4)?,
                volume: row.get(5)?,
                trades: row.get(6)?,
                adjusted_close: row.get(7)?,
            })
        })?;

        let mut candles = Vec::new();
        for r in rows {
            candles.push(r?);
        }
        Ok(candles)
    }

    pub const MAX_DAILY_QUOTES: usize = 180;

    /// Uma linha por ticker por dia de pregão. Sem escrita se close e volume não mudaram.
    pub fn upsert_daily_quote(
        &self,
        symbol: &str,
        candle: &Candle,
        fetched_at_ms: i64,
    ) -> anyhow::Result<DailyQuoteWrite> {
        let symbol = symbol.trim().to_uppercase();
        let day_ms = b3_core::quote::brt_day_start_ms(candle.timestamp_ms);
        let conn = self.conn.lock();

        let existing = conn
            .query_row(
                "SELECT open, high, low, close, volume FROM quotes WHERE symbol = ?1 AND timestamp_ms = ?2 AND source = 'market'",
                params![symbol, day_ms],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, f64>(4)?,
                    ))
                },
            )
            .ok();

        if let Some((open, high, low, close, volume)) = existing {
            if (close - candle.close).abs() < 1e-6 && (volume - candle.volume).abs() < 1e-6 {
                return Ok(DailyQuoteWrite::Unchanged);
            }
            let high = high.max(candle.high).max(candle.close);
            let low = low.min(candle.low).min(candle.close);
            conn.execute(
                "UPDATE quotes SET high = ?1, low = ?2, close = ?3, volume = ?4, adjusted_close = ?5, fetched_at_ms = ?6
                 WHERE symbol = ?7 AND timestamp_ms = ?8 AND source = 'market'",
                params![
                    high,
                    low,
                    candle.close,
                    candle.volume,
                    candle.close,
                    fetched_at_ms,
                    symbol,
                    day_ms
                ],
            )?;
            let _ = (open, low);
            Ok(DailyQuoteWrite::Updated)
        } else {
            conn.execute(
                "INSERT INTO quotes (symbol, timestamp_ms, open, high, low, close, volume, trades, adjusted_close, source, fetched_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'market', ?10)",
                params![
                    symbol,
                    day_ms,
                    candle.close,
                    candle.high.max(candle.close),
                    candle.low.min(candle.close),
                    candle.close,
                    candle.volume,
                    candle.trades,
                    candle.close,
                    fetched_at_ms,
                ],
            )?;
            Ok(DailyQuoteWrite::Inserted)
        }
    }

    pub fn latest_market_quote(&self, symbol: &str) -> anyhow::Result<Option<(Candle, i64)>> {
        let symbol = symbol.trim().to_uppercase();
        let conn = self.conn.lock();
        let row = conn.query_row(
            "SELECT timestamp_ms, open, high, low, close, volume, trades, adjusted_close, fetched_at_ms
             FROM quotes WHERE symbol = ?1 AND source = 'market'
             ORDER BY timestamp_ms DESC LIMIT 1",
            params![symbol],
            |row| {
                Ok((
                    Candle {
                        timestamp_ms: row.get(0)?,
                        open: row.get(1)?,
                        high: row.get(2)?,
                        low: row.get(3)?,
                        close: row.get(4)?,
                        volume: row.get(5)?,
                        trades: row.get(6)?,
                        adjusted_close: row.get(7)?,
                    },
                    row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                ))
            },
        );
        match row {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Remove dias além do teto. O chamador deve invocar no máximo uma vez por dia.
    pub fn prune_daily_quotes(&self, symbol: &str) -> anyhow::Result<()> {
        let symbol = symbol.trim().to_uppercase();
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM quotes WHERE symbol = ?1 AND source = 'market' AND timestamp_ms NOT IN (
                SELECT timestamp_ms FROM quotes WHERE symbol = ?1 AND source = 'market'
                ORDER BY timestamp_ms DESC LIMIT ?2
            )",
            params![symbol, Self::MAX_DAILY_QUOTES as i64],
        )?;
        Ok(())
    }

    pub fn cvm_latest_year(&self, ticker: &str) -> anyhow::Result<Option<i32>> {
        let ticker = ticker.trim().to_uppercase();
        let conn = self.conn.lock();
        let year = conn.query_row(
            "SELECT MAX(year) FROM cvm_accounts WHERE ticker = ?1",
            params![ticker],
            |row| row.get::<_, Option<i32>>(0),
        )?;
        Ok(year)
    }

    pub fn replace_cvm_year(
        &self,
        ticker: &str,
        year: i32,
        reference_date: NaiveDate,
        accounts: &[(String, f64)],
    ) -> anyhow::Result<()> {
        let ticker = ticker.trim().to_uppercase();
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM cvm_accounts WHERE ticker = ?1", params![ticker])?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO cvm_accounts (ticker, year, account_code, amount, reference_date)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            let date = reference_date.format("%Y-%m-%d").to_string();
            for (code, amount) in accounts {
                stmt.execute(params![ticker, year, code, amount, date])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_cvm_accounts(&self, ticker: &str) -> anyhow::Result<Option<(i32, Vec<(String, f64)>)>> {
        let ticker = ticker.trim().to_uppercase();
        let conn = self.conn.lock();
        let year: Option<i32> = conn.query_row(
            "SELECT MAX(year) FROM cvm_accounts WHERE ticker = ?1",
            params![ticker],
            |row| row.get(0),
        )?;
        let Some(year) = year else {
            return Ok(None);
        };
        let mut stmt = conn.prepare(
            "SELECT account_code, amount FROM cvm_accounts WHERE ticker = ?1 AND year = ?2",
        )?;
        let rows = stmt.query_map(params![ticker, year], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;
        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(row?);
        }
        if accounts.is_empty() {
            Ok(None)
        } else {
            Ok(Some((year, accounts)))
        }
    }

    pub fn insert_trade(&self, trade: &TradeOperation) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO trades (id, date, symbol, side, kind, quantity, unit_price, total_gross, liquidation_fee, registration_fee, brokerage_fee, iss_tax, irrf, broker_name, note_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                trade.id,
                trade.date.format("%Y-%m-%d").to_string(),
                trade.symbol,
                match trade.side { OperationSide::Buy => "BUY", OperationSide::Sell => "SELL" },
                match trade.kind { OperationKind::SwingTrade => "SWING", OperationKind::DayTrade => "DAY" },
                trade.quantity,
                trade.unit_price,
                trade.total_gross,
                trade.liquidation_fee,
                trade.registration_fee,
                trade.brokerage_fee,
                trade.iss_tax,
                trade.irrf,
                trade.broker_name,
                trade.note_number,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_trades(&self) -> anyhow::Result<Vec<TradeOperation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, date, symbol, side, kind, quantity, unit_price, total_gross, liquidation_fee, registration_fee, brokerage_fee, iss_tax, irrf, broker_name, note_number
             FROM trades ORDER BY date ASC, id ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            let date_str: String = row.get(1)?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or_default();
            let side_str: String = row.get(3)?;
            let side = if side_str == "BUY" { OperationSide::Buy } else { OperationSide::Sell };
            let kind_str: String = row.get(4)?;
            let kind = if kind_str == "DAY" { OperationKind::DayTrade } else { OperationKind::SwingTrade };

            Ok(TradeOperation {
                id: row.get(0)?,
                date,
                symbol: row.get(2)?,
                side,
                kind,
                quantity: row.get(5)?,
                unit_price: row.get(6)?,
                total_gross: row.get(7)?,
                liquidation_fee: row.get(8)?,
                registration_fee: row.get(9)?,
                brokerage_fee: row.get(10)?,
                iss_tax: row.get(11)?,
                irrf: row.get(12)?,
                broker_name: row.get(13)?,
                note_number: row.get(14)?,
            })
        })?;

        let mut trades = Vec::new();
        for r in rows {
            trades.push(r?);
        }
        Ok(trades)
    }

    pub fn get_user_tickers(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT ticker FROM user_watchlist ORDER BY slot_index ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut tickers = Vec::new();
        for r in rows {
            let t = r?.trim().to_uppercase();
            if !t.is_empty() && !tickers.contains(&t) {
                tickers.push(t);
            }
        }
        Ok(tickers)
    }

    pub fn set_user_tickers<T: AsRef<str>>(&self, tickers: &[T]) -> anyhow::Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        {
            tx.execute("DELETE FROM user_watchlist", [])?;
            let mut stmt = tx.prepare_cached(
                "INSERT INTO user_watchlist (slot_index, ticker) VALUES (?1, ?2)",
            )?;
            for (idx, ticker) in tickers.iter().enumerate() {
                let sym = ticker.as_ref().trim().to_uppercase();
                if !sym.is_empty() {
                    stmt.execute(params![idx as i64, sym])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn add_user_ticker(&self, ticker: &str) -> anyhow::Result<()> {
        let mut tickers = self.get_user_tickers()?;
        let sym = ticker.trim().to_uppercase();
        if !sym.is_empty() && !tickers.contains(&sym) {
            tickers.push(sym);
            self.set_user_tickers(&tickers)?;
        }
        Ok(())
    }

    pub fn remove_user_ticker(&self, ticker: &str) -> anyhow::Result<()> {
        let mut tickers = self.get_user_tickers()?;
        let sym = ticker.trim().to_uppercase();
        if let Some(pos) = tickers.iter().position(|t| t == &sym) {
            tickers.remove(pos);
            self.set_user_tickers(&tickers)?;
        }
        Ok(())
    }

    /// Insere notícias. A mesma URL permanece uma única linha. Item sem URL http não entra.
    pub fn insert_news(&self, items: &[NewsItem]) -> anyhow::Result<usize> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        let mut inserted = 0usize;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR IGNORE INTO news (url, ticker, title, source, published_at_ms, summary, external_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for item in items {
                let url = item.url.trim();
                if !(url.starts_with("https://") || url.starts_with("http://")) {
                    continue;
                }
                if item.title.trim().is_empty() || item.ticker.trim().is_empty() {
                    continue;
                }
                let changed = stmt.execute(params![
                    url,
                    item.ticker.trim().to_uppercase(),
                    item.title.trim(),
                    item.source.trim(),
                    item.published_at_ms,
                    item.summary.as_deref(),
                    item.external_id.as_deref(),
                ])?;
                inserted += changed;
            }
        }
        tx.commit()?;
        Ok(inserted)
    }

    pub fn search_news(
        &self,
        ticker: Option<&str>,
        text: Option<&str>,
        from_ms: Option<i64>,
        to_ms: Option<i64>,
        limit: usize,
    ) -> anyhow::Result<Vec<NewsItem>> {
        let conn = self.conn.lock();
        let mut sql = String::from(
            "SELECT ticker, title, url, source, published_at_ms, summary, external_id
             FROM news WHERE 1 = 1",
        );
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(ticker) = ticker.map(|ticker| ticker.trim().to_uppercase()).filter(|ticker| !ticker.is_empty()) {
            sql.push_str(" AND ticker = ?");
            values.push(Box::new(ticker));
        }
        if let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) {
            sql.push_str(" AND LOWER(title) LIKE ?");
            values.push(Box::new(format!("%{}%", text.to_lowercase())));
        }
        if let Some(from_ms) = from_ms {
            sql.push_str(" AND published_at_ms >= ?");
            values.push(Box::new(from_ms));
        }
        if let Some(to_ms) = to_ms {
            sql.push_str(" AND published_at_ms <= ?");
            values.push(Box::new(to_ms));
        }
        sql.push_str(" ORDER BY published_at_ms DESC, title ASC LIMIT ?");
        values.push(Box::new(limit as i64));
        let params = rusqlite::params_from_iter(values.iter());
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params, |row| {
            Ok(NewsItem {
                ticker: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                source: row.get(3)?,
                published_at_ms: row.get(4)?,
                summary: row.get(5)?,
                external_id: row.get(6)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn mark_news_attempt(&self, attempted_at_ms: i64) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO news_sync (id, attempted_at_ms) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET attempted_at_ms = excluded.attempted_at_ms",
            params![attempted_at_ms],
        )?;
        Ok(())
    }

    pub fn latest_news_attempt(&self) -> anyhow::Result<Option<i64>> {
        let conn = self.conn.lock();
        let value = conn.query_row("SELECT attempted_at_ms FROM news_sync WHERE id = 1", [], |row| row.get(0));
        match value {
            Ok(ms) => Ok(Some(ms)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_insert_and_get_quotes() {
        let db = Database::in_memory().unwrap();
        let candles = vec![
            Candle {
                timestamp_ms: 1000,
                open: 10.0,
                high: 11.0,
                low: 9.5,
                close: 10.8,
                volume: 50000.0,
                trades: 120,
                adjusted_close: 10.8,
            },
            Candle {
                timestamp_ms: 2000,
                open: 10.8,
                high: 12.0,
                low: 10.5,
                close: 11.5,
                volume: 60000.0,
                trades: 150,
                adjusted_close: 11.5,
            },
        ];

        db.insert_quotes("PETR4", &candles).unwrap();
        let fetched = db.get_quotes("PETR4", 10).unwrap();
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].open, 10.0);
        assert_eq!(fetched[1].close, 11.5);
    }

    #[test]
    fn test_user_watchlist_persistence() {
        let db = Database::in_memory().unwrap();
        assert!(db.get_user_tickers().unwrap().is_empty());

        let initial = vec!["PETR4", "VALE3", "WEGE3", "ITUB4"];
        db.set_user_tickers(&initial).unwrap();

        let loaded = db.get_user_tickers().unwrap();
        assert_eq!(loaded, vec!["PETR4", "VALE3", "WEGE3", "ITUB4"]);

        // Add ticker
        db.add_user_ticker("BBAS3").unwrap();
        let loaded2 = db.get_user_tickers().unwrap();
        assert_eq!(loaded2, vec!["PETR4", "VALE3", "WEGE3", "ITUB4", "BBAS3"]);

        // Remove ticker
        db.remove_user_ticker("VALE3").unwrap();
        let loaded3 = db.get_user_tickers().unwrap();
        assert_eq!(loaded3, vec!["PETR4", "WEGE3", "ITUB4", "BBAS3"]);

        // Replace all tickers with user-defined set
        let user_defined = vec!["TAEE11", "EGIE3", "HGLG11", "XPML11"];
        db.set_user_tickers(&user_defined).unwrap();
        assert_eq!(db.get_user_tickers().unwrap(), user_defined);
    }

    #[test]
    fn test_user_watchlist_file_persistence() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_broker_{}.db", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));

        // Step 1: Open file database and save custom user tickers
        {
            let db = Database::open(&db_path).unwrap();
            let custom_tickers = vec!["BBAS3", "TAEE11", "XPML11", "MXRF11"];
            db.set_user_tickers(&custom_tickers).unwrap();
        }

        // Step 2: Reopen the database from file and verify they persist
        {
            let db = Database::open(&db_path).unwrap();
            let loaded = db.get_user_tickers().unwrap();
            assert_eq!(loaded, vec!["BBAS3", "TAEE11", "XPML11", "MXRF11"]);
        }

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    fn candle_at(timestamp_ms: i64, close: f64, volume: f64) -> Candle {
        Candle {
            timestamp_ms,
            open: close,
            high: close,
            low: close,
            close,
            volume,
            trades: 1,
            adjusted_close: close,
        }
    }

    #[test]
    fn test_daily_quote_is_one_row_per_day_and_capped() {
        let db = Database::in_memory().unwrap();
        let morning = 1_717_416_000_000; // 2024-06-03 12:00 UTC = 09:00 BRT, mesmo dia que a tarde
        let afternoon = morning + 3 * 3_600_000;
        assert_eq!(
            b3_core::quote::brt_day_start_ms(morning),
            b3_core::quote::brt_day_start_ms(afternoon)
        );

        assert_eq!(
            db.upsert_daily_quote("petr4", &candle_at(morning, 36.0, 10.0), morning).unwrap(),
            DailyQuoteWrite::Inserted
        );
        assert_eq!(
            db.upsert_daily_quote("PETR4", &candle_at(afternoon, 36.0, 10.0), afternoon).unwrap(),
            DailyQuoteWrite::Unchanged
        );
        assert_eq!(
            db.upsert_daily_quote("PETR4", &candle_at(afternoon, 36.5, 12.0), afternoon).unwrap(),
            DailyQuoteWrite::Updated
        );

        let market = db.latest_market_quote("PETR4").unwrap().unwrap();
        assert!((market.0.close - 36.5).abs() < 1e-9);
        assert_eq!(market.1, afternoon);
        let stored = db.get_quotes("PETR4", 10).unwrap();
        assert_eq!(stored.len(), 1);

        let day = 86_400_000;
        for i in 0..181 {
            let ts = morning + i * day;
            db.upsert_daily_quote("PETR4", &candle_at(ts, 10.0 + i as f64, 1.0), ts).unwrap();
        }
        db.prune_daily_quotes("PETR4").unwrap();
        let capped = db.get_quotes("PETR4", 500).unwrap();
        let market_rows = capped.iter().filter(|c| c.trades == 1).count();
        assert!(market_rows <= Database::MAX_DAILY_QUOTES);
        assert_eq!(market_rows, Database::MAX_DAILY_QUOTES);
    }

    #[test]
    fn test_cvm_accounts_replace_without_duplicates() {
        let db = Database::in_memory().unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        db.replace_cvm_year("PETR4", 2024, date, &[("3.11".into(), 10.0), ("1".into(), 99.0)])
            .unwrap();
        db.replace_cvm_year("PETR4", 2024, date, &[("3.11".into(), 12.0), ("2.03".into(), 20.0)])
            .unwrap();
        let (year, accounts) = db.load_cvm_accounts("PETR4").unwrap().unwrap();
        assert_eq!(year, 2024);
        assert_eq!(accounts.len(), 2);
        assert!(accounts.iter().any(|(c, v)| c == "3.11" && (*v - 12.0).abs() < 1e-9));
        assert!(!accounts.iter().any(|(c, _)| c == "1"));
        assert_eq!(db.cvm_latest_year("PETR4").unwrap(), Some(2024));
    }

    #[test]
    fn test_news_url_is_unique_and_query_is_per_ticker() {
        let db = Database::in_memory().unwrap();
        let petr = NewsItem {
            ticker: "PETR4".into(),
            title: "Petrobras paga dividendos".into(),
            url: "https://exemplo.test/petr".into(),
            source: "CVM".into(),
            published_at_ms: 2_000,
            summary: None,
            external_id: Some("p1".into()),
        };
        let older = NewsItem {
            ticker: "petr4".into(),
            title: "Petrobras antigo".into(),
            url: "https://exemplo.test/petr-antigo".into(),
            source: "CVM".into(),
            published_at_ms: 1_000,
            summary: None,
            external_id: None,
        };
        let vale = NewsItem {
            ticker: "VALE3".into(),
            title: "Vale produz".into(),
            url: "https://exemplo.test/vale".into(),
            source: "RSS".into(),
            published_at_ms: 3_000,
            summary: None,
            external_id: None,
        };
        assert_eq!(db.insert_news(&[petr.clone(), vale.clone()]).unwrap(), 2);
        assert_eq!(db.insert_news(&[petr, older.clone()]).unwrap(), 1);
        let rows = db.search_news(Some("PETR4"), None, None, None, 10).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "Petrobras paga dividendos");
        assert_eq!(rows[1].title, "Petrobras antigo");
        assert!(rows.iter().all(|row| row.ticker == "PETR4"));
        let filtered = db.search_news(Some("PETR4"), Some("dividendos"), None, None, 10).unwrap();
        assert_eq!(filtered.len(), 1);
        assert!(db.latest_news_attempt().unwrap().is_none());
        db.mark_news_attempt(50).unwrap();
        assert_eq!(db.latest_news_attempt().unwrap(), Some(50));
    }
}
