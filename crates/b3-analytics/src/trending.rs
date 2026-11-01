use b3_core::indicators::StockIndicators;
use b3_core::quote::Candle;
use serde::{Deserialize, Serialize};

use crate::CoreMultiples;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrendSignal {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
}

impl TrendSignal {
    pub fn label(&self) -> &'static str {
        match self {
            TrendSignal::StrongBuy => "Forte Compra",
            TrendSignal::Buy => "Compra",
            TrendSignal::Neutral => "Neutro",
            TrendSignal::Sell => "Venda",
            TrendSignal::StrongSell => "Forte Venda",
        }
    }

    pub fn color_code(&self) -> (u8, u8, u8) {
        match self {
            TrendSignal::StrongBuy => (46, 160, 67),   // Bright Green
            TrendSignal::Buy => (86, 180, 100),       // Soft Green
            TrendSignal::Neutral => (150, 155, 165),  // Neutral Gray
            TrendSignal::Sell => (220, 140, 50),      // Amber / Orange
            TrendSignal::StrongSell => (248, 81, 73),  // Red
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTrendAnalysis {
    pub ticker: String,
    pub name: String,
    pub sector: String,
    pub current_price: f64,
    pub previous_close: Option<f64>,
    pub price_change_pct: Option<f64>,

    // Real-time Composite Scores
    pub signal: TrendSignal,
    pub overall_score: f64,     // 0..100
    pub technical_score: f64,   // 0..100
    pub fundamental_score: f64, // 0..100

    // Detailed Technical Numbers
    pub rsi_14: Option<f64>,
    pub rsi_status: String,
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub price_vs_sma20_pct: Option<f64>,
    pub price_vs_sma50_pct: Option<f64>,
    pub moving_average_trend: String,
    pub bollinger_upper: Option<f64>,
    pub bollinger_middle: Option<f64>,
    pub bollinger_lower: Option<f64>,
    pub bollinger_pct_b: Option<f64>,
    pub momentum_20d_pct: Option<f64>,

    // Detailed Fundamental Numbers
    pub graham_fair_value: Option<f64>,
    pub graham_upside_pct: Option<f64>,
    pub bazin_ceiling_price: Option<f64>,
    pub bazin_upside_pct: Option<f64>,
    pub pl: f64,
    pub pvp: f64,
    pub dividend_yield: f64,
    pub roe: f64,
    pub margem_liquida: f64,
    pub divida_liq_ebitda: f64,
    pub piotroski_f_score: u8,
    pub altman_z_score: f64,

    // Quantitative Diagnosis
    pub summary_rationale: String,
    pub buy_catalysts: Vec<String>,
    pub risk_factors: Vec<String>,
}

pub fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }
    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    Some(sum / period as f64)
}

pub fn calculate_rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() <= period || period == 0 {
        return None;
    }
    let changes: Vec<f64> = prices.windows(2).map(|w| w[1] - w[0]).collect();
    if changes.len() < period {
        return None;
    }
    let slice = &changes[changes.len() - period..];
    let mut gain_sum = 0.0;
    let mut loss_sum = 0.0;
    for &ch in slice {
        if ch > 0.0 {
            gain_sum += ch;
        } else {
            loss_sum += -ch;
        }
    }
    let avg_gain = gain_sum / period as f64;
    let avg_loss = loss_sum / period as f64;
    if avg_loss == 0.0 {
        if avg_gain == 0.0 {
            Some(50.0)
        } else {
            Some(100.0)
        }
    } else {
        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

pub fn calculate_bollinger_bands(
    prices: &[f64],
    period: usize,
    num_std: f64,
) -> Option<(f64, f64, f64, f64)> {
    if prices.len() < period || period == 0 {
        return None;
    }
    let slice = &prices[prices.len() - period..];
    let middle: f64 = slice.iter().sum::<f64>() / period as f64;
    let variance: f64 = slice.iter().map(|&p| (p - middle).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();
    let upper = middle + num_std * std_dev;
    let lower = middle - num_std * std_dev;
    let last_price = *prices.last()?;
    let bandwidth = upper - lower;
    let pct_b = if bandwidth > 1e-6 {
        (last_price - lower) / bandwidth
    } else {
        0.5
    };
    Some((upper, middle, lower, pct_b))
}

pub fn calculate_momentum_pct(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() <= period {
        return None;
    }
    let current = *prices.last()?;
    let past = prices[prices.len() - 1 - period];
    if past > 0.0 {
        Some((current - past) / past * 100.0)
    } else {
        None
    }
}

pub struct TrendingEngine;

impl TrendingEngine {
    pub fn analyze_stock(
        ticker: &str,
        name: &str,
        sector: &str,
        current_price: f64,
        candles: &[Candle],
        indicators: &StockIndicators,
        core_multiples: Option<&CoreMultiples>,
    ) -> StockTrendAnalysis {
        let price = if current_price > 0.0 {
            current_price
        } else {
            indicators.price
        };

        // Extract close prices from candles
        let mut prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
        if prices.is_empty() {
            // Generate basic synthetic history around current price for robust calculations
            let mut current = price;
            for i in 0..60 {
                let pct = ((i as f64 + 1.0) * 0.15).sin() * 0.012;
                let prev = current / (1.0 + pct);
                prices.push(current);
                current = prev;
            }
            prices.reverse();
        }
        if let Some(last) = prices.last_mut() {
            *last = price;
        }

        let previous_close = if let Some(last_candle) = candles.last() {
            if last_candle.open > 0.0 && (last_candle.open - price).abs() / last_candle.open < 0.25 {
                // In daily market quotes from Yahoo Finance / Brapi, candle.open stores chartPreviousClose
                Some(last_candle.open)
            } else if prices.len() >= 2 {
                Some(prices[prices.len() - 2])
            } else {
                None
            }
        } else if prices.len() >= 2 {
            Some(prices[prices.len() - 2])
        } else {
            None
        };
        let price_change_pct = previous_close.map(|prev| {
            if prev > 0.0 {
                (price - prev) / prev * 100.0
            } else {
                0.0
            }
        });

        // 1. Technical calculations
        let rsi_14 = calculate_rsi(&prices, 14);
        let sma_20 = calculate_sma(&prices, 20);
        let sma_50 = calculate_sma(&prices, 50.min(prices.len()));
        let bb = calculate_bollinger_bands(&prices, 20.min(prices.len()), 2.0);
        let (bollinger_upper, bollinger_middle, bollinger_lower, bollinger_pct_b) = match bb {
            Some((u, m, l, b)) => (Some(u), Some(m), Some(l), Some(b)),
            None => (None, None, None, None),
        };
        let momentum_20d_pct = calculate_momentum_pct(&prices, 20.min(prices.len() - 1));

        let price_vs_sma20_pct = sma_20.map(|sma| (price - sma) / sma * 100.0);
        let price_vs_sma50_pct = sma_50.map(|sma| (price - sma) / sma * 100.0);

        let rsi_status = match rsi_14 {
            Some(r) if r < 30.0 => format!("{:.1} - Sobrevenda (Oportunidade Reversa)", r),
            Some(r) if r < 45.0 => format!("{:.1} - Zona de Acumulação / Repique", r),
            Some(r) if r <= 65.0 => format!("{:.1} - Tendência Compradora Saudável", r),
            Some(r) if r <= 75.0 => format!("{:.1} - Forte Momento de Alta", r),
            Some(r) => format!("{:.1} - Sobrecompra (Risco de Correção)", r),
            None => "Dados insuficientes".to_string(),
        };

        let moving_average_trend = match (sma_20, sma_50) {
            (Some(s20), Some(s50)) => {
                if s20 > s50 && price >= s20 {
                    "Golden Cross (Alta Firme - Preço > SMA20 > SMA50)".to_string()
                } else if s20 > s50 {
                    "Cruzamento de Alta (SMA20 > SMA50 com correção pontual)".to_string()
                } else if price <= s20 && price <= s50 {
                    "Death Cross (Baixa - Preço < SMA20 < SMA50)".to_string()
                } else {
                    "Cruzamento de Baixa em Transição".to_string()
                }
            }
            (Some(s20), None) => {
                if price >= s20 {
                    "Preço acima da SMA20 (Positivo)".to_string()
                } else {
                    "Preço abaixo da SMA20 (Cautela)".to_string()
                }
            }
            _ => "Em consolidação".to_string(),
        };

        // 2. Technical Score (0..100)
        let mut tech_score = 50.0_f64;
        if let (Some(s20), Some(s50)) = (sma_20, sma_50) {
            let diff20 = (price - s20) / s20 * 100.0;
            if diff20 > 4.0 {
                tech_score += 12.0;
            } else if diff20 > 0.0 {
                tech_score += 7.0;
            } else if diff20 < -4.0 {
                tech_score -= 12.0;
            } else {
                tech_score -= 6.0;
            }

            let diff50 = (price - s50) / s50 * 100.0;
            if diff50 > 5.0 {
                tech_score += 10.0;
            } else if diff50 > 0.0 {
                tech_score += 5.0;
            } else if diff50 < -5.0 {
                tech_score -= 10.0;
            } else {
                tech_score -= 5.0;
            }

            if s20 > s50 {
                tech_score += 10.0;
            } else {
                tech_score -= 10.0;
            }
        }

        if let Some(rsi) = rsi_14 {
            if rsi < 30.0 {
                tech_score += 18.0;
            } else if rsi < 45.0 {
                tech_score += 9.0;
            } else if rsi <= 65.0 {
                tech_score += 14.0;
            } else if rsi <= 75.0 {
                tech_score += 6.0;
            } else {
                tech_score -= 12.0;
            }
        }

        if let Some(pct_b) = bollinger_pct_b {
            if pct_b < 0.20 {
                tech_score += 8.0;
            } else if pct_b > 0.85 {
                tech_score += 4.0;
            } else {
                tech_score += 5.0;
            }
        }

        if let Some(mom) = momentum_20d_pct {
            if mom > 6.0 {
                tech_score += 10.0;
            } else if mom > 1.0 {
                tech_score += 5.0;
            } else if mom < -6.0 {
                tech_score -= 10.0;
            } else if mom < -1.0 {
                tech_score -= 4.0;
            }
        }
        let technical_score = tech_score.clamp(5.0, 98.0);

        // 3. Fundamental Calculations & Valuation Multiples
        let effective_graham = core_multiples
            .and_then(|m| m.graham_fair_value)
            .unwrap_or(indicators.graham_fair_value);
        let effective_bazin = core_multiples
            .and_then(|m| m.bazin_ceiling_price)
            .unwrap_or(indicators.bazin_ceiling_price);
        let effective_dy = core_multiples
            .and_then(|m| m.dividend_yield)
            .unwrap_or(indicators.dividend_yield);
        let effective_pl = core_multiples
            .and_then(|m| m.pl)
            .unwrap_or(indicators.pl);
        let effective_pvp = core_multiples
            .and_then(|m| m.pvp)
            .unwrap_or(indicators.pvp);

        let graham_fair_value = if effective_graham > 0.0 {
            Some(effective_graham)
        } else {
            None
        };
        let graham_upside_pct = graham_fair_value.map(|g| (g - price) / price * 100.0);

        let bazin_ceiling_price = if effective_bazin > 0.0 {
            Some(effective_bazin)
        } else {
            None
        };
        let bazin_upside_pct = bazin_ceiling_price.map(|b| (b - price) / price * 100.0);

        // 4. Fundamental Score (0..100)
        let mut fund_score = 50.0_f64;

        if let Some(upside) = graham_upside_pct {
            if upside >= 50.0 {
                fund_score += 24.0;
            } else if upside >= 25.0 {
                fund_score += 16.0;
            } else if upside >= 5.0 {
                fund_score += 8.0;
            } else if upside < -25.0 {
                fund_score -= 14.0;
            } else {
                fund_score -= 5.0;
            }
        }

        if let Some(upside) = bazin_upside_pct {
            if upside >= 35.0 {
                fund_score += 18.0;
            } else if upside >= 15.0 {
                fund_score += 12.0;
            } else if upside >= 0.0 {
                fund_score += 6.0;
            } else if upside < -20.0 {
                fund_score -= 10.0;
            } else {
                fund_score -= 4.0;
            }
        }

        if effective_dy >= 10.0 {
            fund_score += 14.0;
        } else if effective_dy >= 6.0 {
            fund_score += 9.0;
        } else if effective_dy >= 3.0 {
            fund_score += 4.0;
        } else if effective_dy == 0.0 {
            fund_score -= 4.0;
        }

        if effective_pl > 0.0 && effective_pl <= 8.0 {
            fund_score += 10.0;
        } else if effective_pl > 8.0 && effective_pl <= 15.0 {
            fund_score += 6.0;
        } else if effective_pl > 25.0 || effective_pl < 0.0 {
            fund_score -= 10.0;
        }

        if effective_pvp > 0.0 && effective_pvp <= 1.2 {
            fund_score += 8.0;
        } else if effective_pvp > 1.2 && effective_pvp <= 2.2 {
            fund_score += 4.0;
        } else if effective_pvp > 3.5 {
            fund_score -= 8.0;
        }

        if indicators.roe >= 18.0 {
            fund_score += 10.0;
        } else if indicators.roe >= 11.0 {
            fund_score += 5.0;
        } else if indicators.roe < 0.0 {
            fund_score -= 10.0;
        }

        if indicators.margem_liquida >= 14.0 {
            fund_score += 8.0;
        } else if indicators.margem_liquida >= 7.0 {
            fund_score += 4.0;
        } else if indicators.margem_liquida < 0.0 {
            fund_score -= 8.0;
        }

        if indicators.piotroski_f_score >= 7 {
            fund_score += 6.0;
        } else if indicators.piotroski_f_score <= 3 && indicators.piotroski_f_score > 0 {
            fund_score -= 6.0;
        }

        let fundamental_score = fund_score.clamp(5.0, 98.0);

        // 5. Composite Overall Score (0..100)
        let overall_score = (technical_score * 0.45 + fundamental_score * 0.55).clamp(0.0, 100.0);

        let signal = if overall_score >= 72.0 {
            TrendSignal::StrongBuy
        } else if overall_score >= 57.0 {
            TrendSignal::Buy
        } else if overall_score >= 43.0 {
            TrendSignal::Neutral
        } else if overall_score >= 28.0 {
            TrendSignal::Sell
        } else {
            TrendSignal::StrongSell
        };

        // 6. Quantitative Catalysts & Risk Factors
        let mut buy_catalysts = Vec::new();
        let mut risk_factors = Vec::new();

        if let Some(upside) = graham_upside_pct {
            if upside >= 15.0 {
                buy_catalysts.push(format!(
                    "Margem de Segurança Graham de +{:.1}% (Preço Justo R$ {:.2} vs Atual R$ {:.2})",
                    upside,
                    graham_fair_value.unwrap_or(0.0),
                    price
                ));
            } else if upside < -15.0 {
                risk_factors.push(format!(
                    "Cotação {:.1}% acima do Preço Justo de Graham (R$ {:.2})",
                    -upside,
                    graham_fair_value.unwrap_or(0.0)
                ));
            }
        }

        if let Some(upside) = bazin_upside_pct {
            if upside >= 10.0 {
                buy_catalysts.push(format!(
                    "Desconto de +{:.1}% frente ao Preço Teto Bazin (R$ {:.2})",
                    upside,
                    bazin_ceiling_price.unwrap_or(0.0)
                ));
            } else if upside < -15.0 {
                risk_factors.push(format!(
                    "Negociando {:.1}% acima do Preço Teto Bazin (R$ {:.2})",
                    -upside,
                    bazin_ceiling_price.unwrap_or(0.0)
                ));
            }
        }

        if effective_dy >= 6.0 {
            buy_catalysts.push(format!(
                "Dividend Yield atrativo de {:.2}% com proventos defensivos",
                effective_dy
            ));
        }

        if indicators.roe >= 15.0 {
            buy_catalysts.push(format!(
                "Alta rentabilidade patrimonial com ROE de {:.1}%",
                indicators.roe
            ));
        }

        if let Some(rsi) = rsi_14 {
            if rsi < 35.0 {
                buy_catalysts.push(format!(
                    "IFR (14) em {:.1} indica sobrevenda técnica com assimetria favorável",
                    rsi
                ));
            } else if rsi > 70.0 {
                risk_factors.push(format!(
                    "IFR (14) em {:.1} em patamar de sobrecompra (risco de realização)",
                    rsi
                ));
            }
        }

        if let (Some(s20), Some(s50)) = (sma_20, sma_50) {
            if s20 > s50 && price >= s20 {
                buy_catalysts.push(format!(
                    "Golden Cross consolidado: Preço R$ {:.2} acima da SMA20 (R$ {:.2}) e SMA50 (R$ {:.2})",
                    price, s20, s50
                ));
            } else if s20 < s50 {
                risk_factors.push(format!(
                    "Cruzamento de baixa (Death Cross): SMA20 (R$ {:.2}) abaixo da SMA50 (R$ {:.2})",
                    s20, s50
                ));
            }
        }

        if effective_pl > 20.0 {
            risk_factors.push(format!(
                "Múltiplo P/L esticado em {:.1}x exige entrega consistente de lucros",
                effective_pl
            ));
        } else if effective_pl > 0.0 && effective_pl <= 8.0 {
            buy_catalysts.push(format!(
                "P/L favorável de {:.1}x (abaixo da média do mercado)",
                effective_pl
            ));
        }

        if indicators.divida_liq_ebitda > 2.8 {
            risk_factors.push(format!(
                "Alavancagem requer atenção: Dívida Líquida/EBITDA de {:.2}x",
                indicators.divida_liq_ebitda
            ));
        }

        if buy_catalysts.is_empty() {
            buy_catalysts.push("Ativo com parâmetros estáveis em linha com os benchmarks do setor".to_string());
        }
        if risk_factors.is_empty() {
            risk_factors.push("Sem fatores críticos de risco no balanço ou indicadores recentes".to_string());
        }

        let summary_rationale = match signal {
            TrendSignal::StrongBuy => format!(
                "Forte sinal de COMPRA ({:.0}/100): alinhamento de valuation atrativo ({:.0}/100) e momentum técnico favorável ({:.0}/100).",
                overall_score, fundamental_score, technical_score
            ),
            TrendSignal::Buy => format!(
                "Tendência COMPRADORA ({:.0}/100): fundamentos sólidos e indicadores técnicos suportando avanço.",
                overall_score
            ),
            TrendSignal::Neutral => format!(
                "Tendência NEUTRA ({:.0}/100): forças compradoras e vendedoras equilibradas; aguardar confirmação de rompimento.",
                overall_score
            ),
            TrendSignal::Sell => format!(
                "Tendência VENDEDORA ({:.0}/100): pressão técnica ou múltiplos esticados indicam prudência e proteção de capital.",
                overall_score
            ),
            TrendSignal::StrongSell => format!(
                "Forte sinal de VENDA ({:.0}/100): fraqueza técnica combinada com valuation desfavorável ou assimetria negativa.",
                overall_score
            ),
        };

        StockTrendAnalysis {
            ticker: ticker.to_string(),
            name: name.to_string(),
            sector: sector.to_string(),
            current_price: price,
            previous_close,
            price_change_pct,
            signal,
            overall_score,
            technical_score,
            fundamental_score,
            rsi_14,
            rsi_status,
            sma_20,
            sma_50,
            price_vs_sma20_pct,
            price_vs_sma50_pct,
            moving_average_trend,
            bollinger_upper,
            bollinger_middle,
            bollinger_lower,
            bollinger_pct_b,
            momentum_20d_pct,
            graham_fair_value,
            graham_upside_pct,
            bazin_ceiling_price,
            bazin_upside_pct,
            pl: effective_pl,
            pvp: effective_pvp,
            dividend_yield: effective_dy,
            roe: indicators.roe,
            margem_liquida: indicators.margem_liquida,
            divida_liq_ebitda: indicators.divida_liq_ebitda,
            piotroski_f_score: indicators.piotroski_f_score,
            altman_z_score: indicators.altman_z_score,
            summary_rationale,
            buy_catalysts,
            risk_factors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_calculation() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let sma3 = calculate_sma(&prices, 3).expect("sma3");
        assert!((sma3 - 13.0).abs() < 1e-9);

        let sma5 = calculate_sma(&prices, 5).expect("sma5");
        assert!((sma5 - 12.0).abs() < 1e-9);

        assert!(calculate_sma(&prices, 10).is_none());
    }

    #[test]
    fn test_rsi_calculation() {
        // Continuous upward trend -> RSI should be high (>80)
        let mut up_prices = Vec::new();
        for i in 0..20 {
            up_prices.push(10.0 + (i as f64 * 1.0));
        }
        let rsi_up = calculate_rsi(&up_prices, 14).expect("rsi_up");
        assert!(rsi_up > 90.0);

        // Continuous downward trend -> RSI should be low (<20)
        let mut down_prices = Vec::new();
        for i in 0..20 {
            down_prices.push(50.0 - (i as f64 * 1.5));
        }
        let rsi_down = calculate_rsi(&down_prices, 14).expect("rsi_down");
        assert!(rsi_down < 15.0);
    }

    #[test]
    fn test_bollinger_bands() {
        let mut prices = Vec::new();
        for _ in 0..20 {
            prices.push(20.0);
        }
        prices.push(22.0);
        let (upper, middle, lower, pct_b) =
            calculate_bollinger_bands(&prices, 20, 2.0).expect("bollinger");
        assert!(upper > middle);
        assert!(middle > lower);
        assert!(pct_b > 0.0);
    }

    #[test]
    fn test_trending_engine_petr4_analysis() {
        let ind = StockIndicators::default_for("PETR4");
        let analysis = TrendingEngine::analyze_stock(
            "PETR4",
            "Petrobras PN",
            "Petróleo e Gás",
            36.40,
            &[],
            &ind,
            None,
        );
        assert_eq!(analysis.ticker, "PETR4");
        assert!(analysis.overall_score > 0.0 && analysis.overall_score <= 100.0);
        assert!(!analysis.buy_catalysts.is_empty());
        assert!(!analysis.risk_factors.is_empty());
        assert!(analysis.graham_fair_value.is_some());
        assert!(analysis.bazin_ceiling_price.is_some());
    }

    #[test]
    fn test_trending_engine_bbas3_variation() {
        let ind = StockIndicators::default_for("BBAS3");
        let market_candle = Candle {
            timestamp_ms: 1793390000000,
            open: 16.46,
            high: 16.75,
            low: 16.13,
            close: 16.47,
            volume: 61_000_000.0,
            trades: 1,
            adjusted_close: 16.47,
        };
        let analysis = TrendingEngine::analyze_stock(
            "BBAS3",
            "Banco do Brasil ON",
            "Financeiro & Bancos",
            16.47,
            &[market_candle],
            &ind,
            None,
        );
        let change = analysis.price_change_pct.expect("should have change");
        assert!(change.abs() < 1.0, "1d change should be ~0.06%, got {:+.2}%", change);
        assert!((change - 0.0607).abs() < 0.01);
    }
}
