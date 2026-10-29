use b3_core::financials::{FinancialPeriod, PeriodKind};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvmAccountRecord {
    pub cnpj_cia: String,
    pub denom_cia: String,
    pub cd_cvm: u32,
    pub dt_refer: NaiveDate,
    pub cd_conta: String,
    pub ds_conta: String,
    pub vl_conta: f64,
}

pub struct CvmIngestor;

impl CvmIngestor {
    pub fn parse_cvm_csv_line(line: &str) -> Option<CvmAccountRecord> {
        // CVM CSVs are semicolon-separated
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() < 12 {
            return None;
        }

        let cnpj = parts.get(0)?.trim_matches('"').trim().to_string();
        let dt_str = parts.get(1)?.trim_matches('"').trim();
        let dt_refer = NaiveDate::parse_from_str(dt_str, "%Y-%m-%d").ok()?;
        let denom_cia = parts.get(3)?.trim_matches('"').trim().to_string();
        let cd_cvm = parts.get(4)?.trim_matches('"').trim().parse::<u32>().ok()?;
        let cd_conta = parts.get(8)?.trim_matches('"').trim().to_string();
        let ds_conta = parts.get(9)?.trim_matches('"').trim().to_string();
        let vl_str = parts.get(10)?.trim_matches('"').trim();
        let vl_conta = vl_str.parse::<f64>().unwrap_or(0.0);

        Some(CvmAccountRecord {
            cnpj_cia: cnpj,
            denom_cia,
            cd_cvm,
            dt_refer,
            cd_conta,
            ds_conta,
            vl_conta,
        })
    }

    pub fn build_financial_period(
        year: i32,
        kind: PeriodKind,
        records: &[CvmAccountRecord],
    ) -> FinancialPeriod {
        let mut accounts = BTreeMap::new();
        let date_start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let date_end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

        for rec in records {
            accounts.insert(rec.cd_conta.clone(), rec.vl_conta);
        }

        FinancialPeriod {
            year,
            kind,
            date_start,
            date_end,
            accounts,
        }
    }
}
