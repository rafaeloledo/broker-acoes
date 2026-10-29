use b3_core::quote::Candle;
use chrono::NaiveDate;
use std::io::BufRead;

pub struct CotahistParser;

impl CotahistParser {
    pub fn parse_line(line: &str) -> Option<(String, Candle)> {
        // Must be a 245-character record starting with "01"
        if line.len() < 240 || !line.starts_with("01") {
            return None;
        }

        let date_str = line.get(2..10)?;
        let date = NaiveDate::parse_from_str(date_str, "%Y%m%d").ok()?;
        let timestamp_ms = date.and_hms_opt(18, 0, 0)?.and_utc().timestamp_millis();

        let codneg = line.get(12..24)?.trim().to_string();
        if codneg.is_empty() {
            return None;
        }

        // BDI code (02 = standard equities, 12 = FIIs, 14 = ETFs/BDRs)
        let bdi = line.get(10..12)?.trim();
        if bdi != "02" && bdi != "12" && bdi != "14" && bdi != "96" {
            // Filter out options, forwards unless wanted
            return None;
        }

        let preabe = line.get(56..69)?.trim().parse::<f64>().ok()? / 100.0;
        let premax = line.get(69..82)?.trim().parse::<f64>().ok()? / 100.0;
        let premin = line.get(82..95)?.trim().parse::<f64>().ok()? / 100.0;
        let preult = line.get(108..121)?.trim().parse::<f64>().ok()? / 100.0;
        let quatot = line.get(152..170)?.trim().parse::<f64>().unwrap_or(0.0);
        let voltot = line.get(170..188)?.trim().parse::<f64>().unwrap_or(0.0) / 100.0;

        let candle = Candle {
            timestamp_ms,
            open: preabe,
            high: premax,
            low: premin,
            close: preult,
            volume: voltot,
            trades: quatot as u32,
            adjusted_close: preult,
        };

        Some((codneg, candle))
    }

    pub fn parse_cotahist_content<R: BufRead>(reader: R) -> Vec<(String, Candle)> {
        let mut results = Vec::new();
        for line in reader.lines().flatten() {
            if let Some(entry) = Self::parse_line(&line) {
                results.push(entry);
            }
        }
        results
    }
}
