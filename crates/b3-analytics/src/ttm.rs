use b3_core::financials::{FinancialPeriod, PeriodKind};
use std::collections::BTreeMap;

pub struct TtmAggregator;

impl TtmAggregator {
    /// Aggregates 4 consecutive quarters into a Trailing Twelve Months (TTM) period.
    /// Flow metrics (DRE/DFC) are summed; balance sheet metrics (BPA/BPP) take the latest period.
    pub fn aggregate_quarters(quarters: &[FinancialPeriod]) -> Option<FinancialPeriod> {
        if quarters.len() < 4 {
            return quarters.last().cloned();
        }

        let latest = quarters.last()?;
        let earliest = quarters.first()?;
        let mut ttm_accounts = BTreeMap::new();

        // 1. Balance sheet accounts (latest quarter balance)
        for (code, val) in &latest.accounts {
            if code.starts_with("1") || code.starts_with("2") {
                ttm_accounts.insert(code.clone(), *val);
            }
        }

        // 2. Flow accounts (DRE group 3, DFC group 6): sum over the 4 quarters
        for q in quarters.iter().rev().take(4) {
            for (code, val) in &q.accounts {
                if code.starts_with("3") || code.starts_with("6") || code.starts_with("7") {
                    let entry = ttm_accounts.entry(code.clone()).or_insert(0.0);
                    *entry += *val;
                }
            }
        }

        Some(FinancialPeriod {
            year: latest.year,
            kind: PeriodKind::TTM,
            date_start: earliest.date_start,
            date_end: latest.date_end,
            accounts: ttm_accounts,
        })
    }
}
