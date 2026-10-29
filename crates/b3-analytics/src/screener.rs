use b3_core::indicators::StockIndicators;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenerCriteria {
    pub min_dy: Option<f64>,
    pub max_pl: Option<f64>,
    pub min_pl: Option<f64>,
    pub max_pvp: Option<f64>,
    pub min_roe: Option<f64>,
    pub min_roic: Option<f64>,
    pub min_margem_liquida: Option<f64>,
    pub max_divida_liq_ebitda: Option<f64>,
    pub min_liquidez_corrente: Option<f64>,
}

impl Default for ScreenerCriteria {
    fn default() -> Self {
        Self {
            min_dy: None,
            max_pl: None,
            min_pl: None,
            max_pvp: None,
            min_roe: None,
            min_roic: None,
            min_margem_liquida: None,
            max_divida_liq_ebitda: None,
            min_liquidez_corrente: None,
        }
    }
}

pub struct ScreenerEngine;

impl ScreenerEngine {
    pub fn filter<'a>(
        items: &'a [StockIndicators],
        criteria: &ScreenerCriteria,
    ) -> Vec<&'a StockIndicators> {
        items
            .iter()
            .filter(|i| {
                if let Some(min_dy) = criteria.min_dy {
                    if i.dividend_yield < min_dy { return false; }
                }
                if let Some(max_pl) = criteria.max_pl {
                    if i.pl > max_pl || i.pl <= 0.0 { return false; }
                }
                if let Some(min_pl) = criteria.min_pl {
                    if i.pl < min_pl { return false; }
                }
                if let Some(max_pvp) = criteria.max_pvp {
                    if i.pvp > max_pvp || i.pvp <= 0.0 { return false; }
                }
                if let Some(min_roe) = criteria.min_roe {
                    if i.roe < min_roe { return false; }
                }
                if let Some(min_roic) = criteria.min_roic {
                    if i.roic < min_roic { return false; }
                }
                if let Some(min_ml) = criteria.min_margem_liquida {
                    if i.margem_liquida < min_ml { return false; }
                }
                if let Some(max_div) = criteria.max_divida_liq_ebitda {
                    if i.divida_liq_ebitda > max_div { return false; }
                }
                if let Some(min_liq) = criteria.min_liquidez_corrente {
                    if i.liquidez_corrente < min_liq { return false; }
                }
                true
            })
            .collect()
    }

    /// Magic Formula (Joel Greenblatt):
    /// Rank by Earnings Yield (EBIT / EV or 1 / (EV/EBIT)) + Rank by ROIC
    pub fn magic_formula<'a>(items: &'a [StockIndicators]) -> Vec<&'a StockIndicators> {
        let mut valid: Vec<&StockIndicators> = items
            .iter()
            .filter(|i| i.ev_ebit > 0.0 && i.roic > 0.0)
            .collect();

        // Sort by EY descending (or EV/EBIT ascending)
        valid.sort_by(|a, b| a.ev_ebit.partial_cmp(&b.ev_ebit).unwrap());
        let mut scores: Vec<(&StockIndicators, usize)> = valid.into_iter().enumerate().map(|(r, item)| (item, r)).collect();

        // Sort by ROIC descending
        scores.sort_by(|a, b| b.0.roic.partial_cmp(&a.0.roic).unwrap());
        let mut final_scores: Vec<(&StockIndicators, usize)> = scores.into_iter().enumerate().map(|(r, (item, prev_rank))| (item, prev_rank + r)).collect();

        // Sort by combined rank ascending
        final_scores.sort_by_key(|(_, rank)| *rank);
        final_scores.into_iter().map(|(item, _)| item).collect()
    }

    /// Bazin Strategy: DY >= 6%, NetDebt/EBITDA <= 2.5, positive margins
    pub fn bazin_preset() -> ScreenerCriteria {
        ScreenerCriteria {
            min_dy: Some(6.0),
            max_pl: Some(15.0),
            min_pl: Some(1.0),
            max_pvp: None,
            min_roe: Some(10.0),
            min_roic: None,
            min_margem_liquida: Some(8.0),
            max_divida_liq_ebitda: Some(2.5),
            min_liquidez_corrente: Some(1.0),
        }
    }

    /// Graham Deep Value: P/L <= 15, P/VP <= 1.5, Liquidez Corrente >= 1.5
    pub fn graham_preset() -> ScreenerCriteria {
        ScreenerCriteria {
            min_dy: None,
            max_pl: Some(15.0),
            min_pl: Some(1.0),
            max_pvp: Some(1.5),
            min_roe: Some(8.0),
            min_roic: None,
            min_margem_liquida: Some(5.0),
            max_divida_liq_ebitda: Some(3.0),
            min_liquidez_corrente: Some(1.5),
        }
    }
}
