use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use b3_ingest::companies::issuer_for_ticker;
use b3_ingest::dfp::{self, KeptAccount};
use b3_ingest::ipe;
use b3_ingest::live::LiveFeedClient;
use b3_ingest::rss;
use b3_storage::Database;
use chrono::{Datelike, Utc};
use tokio::time::sleep;

const NEWS_POLL_INTERVAL_MS: i64 = 30 * 60 * 1000;

#[derive(Debug, Clone)]
pub struct QuoteTick {
    pub symbol: String,
    pub close: f64,
    pub fetched_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct CvmLoaded {
    pub ticker: String,
    pub year: i32,
    pub accounts: Vec<(String, f64)>,
}

#[derive(Debug, Clone)]
pub enum MarketEvent {
    Quotes(Vec<QuoteTick>),
    Cvm(Vec<CvmLoaded>),
    News,
}

pub fn spawn(db: Arc<Database>) -> crossbeam_channel::Receiver<MarketEvent> {
    let (tx, rx) = crossbeam_channel::unbounded();
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime,
            Err(err) => {
                log::error!("Falha ao iniciar a coleta de mercado: {err}");
                return;
            }
        };
        runtime.block_on(run(db, tx));
    });
    rx
}

async fn run(db: Arc<Database>, tx: crossbeam_channel::Sender<MarketEvent>) {
    let quotes = LiveFeedClient::new();
    let cvm_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let mut last_quote_ms: Option<i64> = None;
    let mut last_prune_day: Option<i64> = None;
    let mut known_tickers: Vec<String> = Vec::new();
    let mut cvm_attempted_for: Vec<String> = Vec::new();
    let mut last_news_ms: Option<i64> = None;

    loop {
        let tickers = db.get_user_tickers().unwrap_or_default();
        if tickers != known_tickers {
            last_quote_ms = None;
            last_news_ms = None;
            if tickers != cvm_attempted_for {
                cvm_attempted_for.clear();
            }
            known_tickers = tickers.clone();
        }

        let now = Utc::now().timestamp_millis();
        let due = match last_quote_ms {
            None => true,
            Some(previous) => {
                LiveFeedClient::is_b3_session(now)
                    && now.saturating_sub(previous) >= LiveFeedClient::QUOTE_POLL_INTERVAL_MS
            }
        };
        if due && !tickers.is_empty() {
            let mut updates = Vec::new();
            for ticker in &tickers {
                match quotes.fetch_brapi_quote(ticker).await {
                    Ok(Some(candle)) => {
                        let fetched_at = candle.timestamp_ms;
                        match db.upsert_daily_quote(ticker, &candle, fetched_at) {
                            Ok(_) => updates.push(QuoteTick {
                                symbol: ticker.clone(),
                                close: candle.close,
                                fetched_at_ms: fetched_at,
                            }),
                            Err(err) => log::warn!("Falha ao gravar cotação de {ticker}: {err}"),
                        }
                    }
                    Ok(None) => log::warn!("Brapi não devolveu preço para {ticker}"),
                    Err(err) => log::warn!("Falha na cotação de {ticker}: {err}"),
                }
            }
            let day = b3_core::quote::brt_day_start_ms(now);
            if last_prune_day != Some(day) {
                for ticker in &tickers {
                    if let Err(err) = db.prune_daily_quotes(ticker) {
                        log::warn!("Falha ao podar cotações de {ticker}: {err}");
                    }
                }
                last_prune_day = Some(day);
            }
            if !updates.is_empty() {
                let _ = tx.send(MarketEvent::Quotes(updates));
            }
            last_quote_ms = Some(now);
        }

        if cvm_attempted_for != tickers {
            match sync_cvm(&db, &cvm_client, &tickers).await {
                Ok(loaded) => {
                    if !loaded.is_empty() {
                        let _ = tx.send(MarketEvent::Cvm(loaded));
                    }
                    cvm_attempted_for = tickers.clone();
                }
                Err(err) => log::warn!("Falha na carga CVM: {err}"),
            }
        }

        let news_due = last_news_ms
            .map(|previous| now.saturating_sub(previous) >= NEWS_POLL_INTERVAL_MS)
            .unwrap_or(true);
        if news_due && !tickers.is_empty() {
            sync_news(&db, &cvm_client, &tickers).await;
            last_news_ms = Some(Utc::now().timestamp_millis());
            let _ = tx.send(MarketEvent::News);
        }

        sleep(Duration::from_secs(20)).await;
    }
}

async fn sync_cvm(
    db: &Database,
    client: &reqwest::Client,
    tickers: &[String],
) -> anyhow::Result<Vec<CvmLoaded>> {
    let mut stored = HashMap::new();
    let mut cnpjs = HashSet::new();
    for ticker in tickers {
        if let Some(issuer) = issuer_for_ticker(ticker) {
            cnpjs.insert(issuer.cnpj_digits.to_string());
            if let Some((year, accounts)) = db.load_cvm_accounts(ticker)? {
                let has_shares = accounts.iter().any(|(code, amount)| {
                    code == b3_core::financials::cvm_codes::SHARES_TOTAL && *amount > 0.0
                });
                if has_shares {
                    stored.insert(ticker.clone(), year);
                }
            }
        }
    }
    if cnpjs.is_empty() {
        return Ok(Vec::new());
    }

    let today_year = Utc::now().date_naive().year();
    let candidates = dfp::candidate_dfp_years(today_year);
    let mut loaded = Vec::new();
    for year in candidates {
        if dfp::next_dfp_year(&stored, &[year], tickers).is_none() {
            continue;
        }
        let rows = dfp::download_dfp_accounts(client, year, &cnpjs).await?;
        if rows.is_empty() {
            continue;
        }
        loaded = persist_rows(db, tickers, &rows)?;
        break;
    }
    Ok(loaded)
}

fn persist_rows(db: &Database, tickers: &[String], rows: &[KeptAccount]) -> anyhow::Result<Vec<CvmLoaded>> {
    let mut loaded = Vec::new();
    for ticker in tickers {
        let Some(issuer) = issuer_for_ticker(ticker) else {
            continue;
        };
        let mine: Vec<&KeptAccount> = rows
            .iter()
            .filter(|row| row.cnpj_digits == issuer.cnpj_digits)
            .collect();
        if mine.is_empty() {
            continue;
        }
        let year = mine.iter().map(|row| row.year).max().unwrap();
        let reference = mine
            .iter()
            .find(|row| row.year == year)
            .map(|row| row.reference_date)
            .unwrap();
        let mut accounts: Vec<(String, f64)> = Vec::new();
        for row in mine.iter().filter(|row| row.year == year) {
            if let Some(existing) = accounts.iter_mut().find(|(code, _)| code == &row.account_code) {
                existing.1 = row.amount;
            } else {
                accounts.push((row.account_code.clone(), row.amount));
            }
        }
        db.replace_cvm_year(ticker, year, reference, &accounts)?;
        loaded.push(CvmLoaded {
            ticker: ticker.clone(),
            year,
            accounts,
        });
    }
    Ok(loaded)
}

async fn sync_news(db: &Database, client: &reqwest::Client, tickers: &[String]) {
    let mut items = Vec::new();
    let year = Utc::now().date_naive().year();
    for candidate in [year, year - 1] {
        match ipe::download_ipe_news(client, candidate, tickers).await {
            Ok(mut rows) => items.append(&mut rows),
            Err(err) => log::warn!("Falha nos fatos relevantes de {candidate}: {err}"),
        }
    }
    for feed in rss::load_feed_urls(std::path::Path::new("news_feeds.txt")) {
        match rss::download_rss_news(client, &feed, tickers).await {
            Ok(mut rows) => items.append(&mut rows),
            Err(err) => log::warn!("Falha no RSS: {err}"),
        }
    }
    if let Err(err) = db.insert_news(&items) {
        log::warn!("Falha ao gravar notícias: {err}");
    }
    if let Err(err) = db.mark_news_attempt(Utc::now().timestamp_millis()) {
        log::warn!("Falha ao registrar a coleta de notícias: {err}");
    }
}
