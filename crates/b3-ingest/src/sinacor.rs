use b3_core::tax::{OperationKind, OperationSide, TradeOperation};
use chrono::NaiveDate;
use regex::Regex;

pub struct SinacorParser;

impl SinacorParser {
    pub fn parse_trade_lines(text: &str, broker: &str) -> Vec<TradeOperation> {
        let mut trades = Vec::new();

        // Regex for trading line in SINACOR note:
        // Example: "1-BOVESPA C VISTA PETR4 PETROBRAS PN 100 36,40 3.640,00 C"
        // Or "C VISTA PETR4 100 36,40 3.640,00"
        let re_trade = Regex::new(
            r"(?i)\b([CV])\s+(?:VISTA|FRACIONARIO|FRAC)\s+([A-Z0-9]{4,6})\s+.*?\s+(\d+)\s+([\d.,]+)\s+([\d.,]+)\s+([CD])"
        ).unwrap();

        let re_date = Regex::new(r"(\d{2}/\d{2}/\d{4})").unwrap();
        let mut note_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        for cap in re_date.captures_iter(text) {
            if let Ok(d) = NaiveDate::parse_from_str(&cap[1], "%d/%m/%Y") {
                note_date = d;
                break;
            }
        }

        let mut idx = 1;
        for cap in re_trade.captures_iter(text) {
            let side = if &cap[1] == "C" || &cap[1] == "c" {
                OperationSide::Buy
            } else {
                OperationSide::Sell
            };
            let symbol = cap[2].to_uppercase();
            let qty = cap[3].parse::<f64>().unwrap_or(0.0);
            let price_str = cap[4].replace('.', "").replace(',', ".");
            let price = price_str.parse::<f64>().unwrap_or(0.0);
            let total = qty * price;

            // Approximate prorated fees for typical B3 trade (0.025% liquidação + 0.005% emolumentos)
            let liquidation_fee = (total * 0.00025).max(0.01);
            let registration_fee = (total * 0.00005).max(0.01);

            trades.push(TradeOperation {
                id: format!("TRD-{}-{}", note_date.format("%Y%m%d"), idx),
                date: note_date,
                symbol,
                side,
                kind: OperationKind::SwingTrade,
                quantity: qty,
                unit_price: price,
                total_gross: total,
                liquidation_fee,
                registration_fee,
                brokerage_fee: 0.0,
                iss_tax: 0.0,
                irrf: if side == OperationSide::Sell { total * 0.00005 } else { 0.0 },
                broker_name: broker.to_string(),
                note_number: format!("NC-{}", idx),
            });
            idx += 1;
        }

        trades
    }
}
