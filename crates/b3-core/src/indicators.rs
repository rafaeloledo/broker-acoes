use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockIndicators {
    pub symbol: String,
    pub price: f64,

    // Valuation
    pub pl: f64,              // P/L (Preço / Lucro)
    pub pvp: f64,             // P/VP (Preço / Valor Patrimonial)
    pub ev_ebitda: f64,       // EV / EBITDA (Enterprise Value / EBITDA)
    pub ev_ebit: f64,         // EV / EBIT
    pub p_ebitda: f64,        // P / EBITDA
    pub p_ebit: f64,          // P / EBIT
    pub p_sr: f64,            // PSR (Preço / Receita Líquida por ação)
    pub p_ativo: f64,         // P / Ativo Total
    pub p_cap_giro: f64,      // P / Capital de Giro (Ativo Circulante - Passivo Circulante)
    pub p_ativo_circ_liq: f64,// P / Ativo Circulante Líquido
    pub dividend_yield: f64,  // DY % (Proventos LTM / Preço)
    pub peg_ratio: f64,       // PEG Ratio (P/L / Crescimento de Lucros)

    // Endividamento & Liquidez
    pub divida_liq_pl: f64,     // Dívida Líquida / Patrimônio Líquido
    pub divida_liq_ebitda: f64, // Dívida Líquida / EBITDA
    pub divida_liq_ebit: f64,   // Dívida Líquida / EBIT
    pub pl_ativos: f64,         // PL / Ativos Totais
    pub passivos_ativos: f64,   // Passivo Total / Ativo Total
    pub liquidez_corrente: f64, // Ativo Circulante / Passivo Circulante
    pub liquidez_seca: f64,     // (Ativo Circulante - Estoques) / Passivo Circulante

    // Eficiência Operacional
    pub margem_bruta: f64,   // Lucro Bruto / Receita Líquida %
    pub margem_ebitda: f64,  // EBITDA / Receita Líquida %
    pub margem_ebit: f64,    // EBIT / Receita Líquida %
    pub margem_liquida: f64, // Lucro Líquido / Receita Líquida %
    pub giro_ativos: f64,    // Receita Líquida / Ativo Total

    // Rentabilidade
    pub roe: f64,  // Retorno sobre Patrimônio Líquido % (Lucro / PL)
    pub roic: f64, // Retorno sobre Capital Investido % (EBIT * (1 - t) / Capital Investido)
    pub roa: f64,  // Retorno sobre Ativos % (Lucro / Ativo Total)

    // Crescimento (5 anos)
    pub cagr_receitas_5a: f64, // Taxa de Crescimento Anual Composta de Receita %
    pub cagr_lucros_5a: f64,   // Taxa de Crescimento Anual Composta de Lucros %

    // Dados por Ação
    pub lpa: f64, // Lucro por Ação
    pub vpa: f64, // Valor Patrimonial por Ação
    pub dpa: f64, // Dividendo por Ação (LTM)

    // Modelos Quantitativos & Valuation
    pub graham_fair_value: f64,    // sqrt(22.5 * LPA * VPA)
    pub bazin_ceiling_price: f64,  // DPA / 0.06 (Preço teto Bazin a 6% yield)
    pub piotroski_f_score: u8,     // 0 a 9
    pub altman_z_score: f64,       // Score de solvência (> 2.99 seguro, < 1.81 distress)
}

impl StockIndicators {
    pub fn default_for(symbol: &str) -> Self {
        let sym = symbol.to_uppercase();
        match sym.as_str() {
            "PETR4" => Self {
                symbol: "PETR4".to_string(),
                price: 36.40,
                pl: 4.85,
                pvp: 1.15,
                ev_ebitda: 2.80,
                ev_ebit: 3.40,
                p_ebitda: 2.45,
                p_ebit: 3.10,
                p_sr: 0.95,
                p_ativo: 0.48,
                p_cap_giro: 4.10,
                p_ativo_circ_liq: -1.2,
                dividend_yield: 14.80,
                peg_ratio: 0.42,
                divida_liq_pl: 0.85,
                divida_liq_ebitda: 0.95,
                divida_liq_ebit: 1.25,
                pl_ativos: 0.42,
                passivos_ativos: 0.58,
                liquidez_corrente: 1.45,
                liquidez_seca: 1.20,
                margem_bruta: 51.2,
                margem_ebitda: 42.5,
                margem_ebit: 33.8,
                margem_liquida: 21.4,
                giro_ativos: 0.52,
                roe: 24.2,
                roic: 21.8,
                roa: 10.4,
                cagr_receitas_5a: 14.2,
                cagr_lucros_5a: 18.5,
                lpa: 7.50,
                vpa: 31.65,
                dpa: 5.38,
                graham_fair_value: (22.5 * 7.50 * 31.65_f64).sqrt(),
                bazin_ceiling_price: 5.38 / 0.06,
                piotroski_f_score: 7,
                altman_z_score: 2.65,
            },
            "VALE3" => Self {
                symbol: "VALE3".to_string(),
                price: 58.90,
                pl: 5.60,
                pvp: 1.40,
                ev_ebitda: 3.85,
                ev_ebit: 4.60,
                p_ebitda: 3.20,
                p_ebit: 4.10,
                p_sr: 1.35,
                p_ativo: 0.62,
                p_cap_giro: 3.80,
                p_ativo_circ_liq: -2.1,
                dividend_yield: 8.90,
                peg_ratio: 0.68,
                divida_liq_pl: 0.42,
                divida_liq_ebitda: 0.58,
                divida_liq_ebit: 0.72,
                pl_ativos: 0.48,
                passivos_ativos: 0.52,
                liquidez_corrente: 1.62,
                liquidez_seca: 1.30,
                margem_bruta: 42.0,
                margem_ebitda: 38.2,
                margem_ebit: 29.5,
                margem_liquida: 24.1,
                giro_ativos: 0.45,
                roe: 25.1,
                roic: 19.4,
                roa: 12.0,
                cagr_receitas_5a: 9.8,
                cagr_lucros_5a: 12.4,
                lpa: 10.51,
                vpa: 42.07,
                dpa: 5.24,
                graham_fair_value: (22.5 * 10.51 * 42.07_f64).sqrt(),
                bazin_ceiling_price: 5.24 / 0.06,
                piotroski_f_score: 6,
                altman_z_score: 3.12,
            },
            "WEGE3" => Self {
                symbol: "WEGE3".to_string(),
                price: 52.30,
                pl: 35.80,
                pvp: 10.20,
                ev_ebitda: 24.50,
                ev_ebit: 27.80,
                p_ebitda: 23.90,
                p_ebit: 27.10,
                p_sr: 6.20,
                p_ativo: 5.10,
                p_cap_giro: 15.40,
                p_ativo_circ_liq: -8.5,
                dividend_yield: 1.85,
                peg_ratio: 1.82,
                divida_liq_pl: -0.12, // Caixa líquido
                divida_liq_ebitda: -0.25,
                divida_liq_ebit: -0.28,
                pl_ativos: 0.54,
                passivos_ativos: 0.46,
                liquidez_corrente: 2.15,
                liquidez_seca: 1.48,
                margem_bruta: 34.5,
                margem_ebitda: 22.8,
                margem_ebit: 19.6,
                margem_liquida: 16.4,
                giro_ativos: 0.88,
                roe: 30.5,
                roic: 32.8,
                roa: 16.2,
                cagr_receitas_5a: 22.4,
                cagr_lucros_5a: 24.1,
                lpa: 1.46,
                vpa: 5.12,
                dpa: 0.97,
                graham_fair_value: (22.5 * 1.46 * 5.12_f64).sqrt(),
                bazin_ceiling_price: 0.97 / 0.06,
                piotroski_f_score: 8,
                altman_z_score: 5.80,
            },
            "ITUB4" => Self {
                symbol: "ITUB4".to_string(),
                price: 34.20,
                pl: 8.90,
                pvp: 1.75,
                ev_ebitda: 0.0, // Não aplicável a bancos
                ev_ebit: 0.0,
                p_ebitda: 0.0,
                p_ebit: 0.0,
                p_sr: 1.85,
                p_ativo: 0.12,
                p_cap_giro: 0.0,
                p_ativo_circ_liq: 0.0,
                dividend_yield: 7.20,
                peg_ratio: 0.75,
                divida_liq_pl: 0.0,
                divida_liq_ebitda: 0.0,
                divida_liq_ebit: 0.0,
                pl_ativos: 0.07,
                passivos_ativos: 0.93,
                liquidez_corrente: 1.0,
                liquidez_seca: 1.0,
                margem_bruta: 48.0,
                margem_ebitda: 0.0,
                margem_ebit: 0.0,
                margem_liquida: 24.5,
                giro_ativos: 0.08,
                roe: 21.4,
                roic: 0.0,
                roa: 1.5,
                cagr_receitas_5a: 11.2,
                cagr_lucros_5a: 14.8,
                lpa: 3.84,
                vpa: 19.54,
                dpa: 2.46,
                graham_fair_value: (22.5 * 3.84 * 19.54_f64).sqrt(),
                bazin_ceiling_price: 2.46 / 0.06,
                piotroski_f_score: 8,
                altman_z_score: 0.0,
            },
            "BBAS3" => Self {
                symbol: "BBAS3".to_string(),
                price: 28.50,
                pl: 4.20,
                pvp: 0.85,
                ev_ebitda: 0.0,
                ev_ebit: 0.0,
                p_ebitda: 0.0,
                p_ebit: 0.0,
                p_sr: 1.10,
                p_ativo: 0.08,
                p_cap_giro: 0.0,
                p_ativo_circ_liq: 0.0,
                dividend_yield: 10.40,
                peg_ratio: 0.40,
                divida_liq_pl: 0.0,
                divida_liq_ebitda: 0.0,
                divida_liq_ebit: 0.0,
                pl_ativos: 0.08,
                passivos_ativos: 0.92,
                liquidez_corrente: 1.0,
                liquidez_seca: 1.0,
                margem_bruta: 45.0,
                margem_ebitda: 0.0,
                margem_ebit: 0.0,
                margem_liquida: 22.0,
                giro_ativos: 0.07,
                roe: 21.8,
                roic: 0.0,
                roa: 1.6,
                cagr_receitas_5a: 13.5,
                cagr_lucros_5a: 16.2,
                lpa: 6.78,
                vpa: 33.52,
                dpa: 2.96,
                graham_fair_value: (22.5 * 6.78 * 33.52_f64).sqrt(),
                bazin_ceiling_price: 2.96 / 0.06,
                piotroski_f_score: 7,
                altman_z_score: 0.0,
            },
            _ => Self {
                symbol: sym.to_string(),
                price: 25.00,
                pl: 10.0,
                pvp: 1.5,
                ev_ebitda: 6.0,
                ev_ebit: 7.5,
                p_ebitda: 5.5,
                p_ebit: 7.0,
                p_sr: 1.5,
                p_ativo: 0.8,
                p_cap_giro: 4.0,
                p_ativo_circ_liq: -1.0,
                dividend_yield: 6.0,
                peg_ratio: 1.0,
                divida_liq_pl: 0.5,
                divida_liq_ebitda: 1.2,
                divida_liq_ebit: 1.5,
                pl_ativos: 0.5,
                passivos_ativos: 0.5,
                liquidez_corrente: 1.8,
                liquidez_seca: 1.3,
                margem_bruta: 35.0,
                margem_ebitda: 20.0,
                margem_ebit: 15.0,
                margem_liquida: 10.0,
                giro_ativos: 0.6,
                roe: 15.0,
                roic: 14.0,
                roa: 7.5,
                cagr_receitas_5a: 10.0,
                cagr_lucros_5a: 10.0,
                lpa: 2.50,
                vpa: 16.66,
                dpa: 1.50,
                graham_fair_value: (22.5 * 2.50 * 16.66_f64).sqrt(),
                bazin_ceiling_price: 1.50 / 0.06,
                piotroski_f_score: 6,
                altman_z_score: 2.5,
            },
        }
    }
}
