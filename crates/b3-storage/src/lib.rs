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
            "#,
        )?;
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
}
