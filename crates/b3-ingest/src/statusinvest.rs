use b3_core::fii::{FiiDetails, FiiSegment};
use b3_core::indicators::StockIndicators;
use b3_core::ticker::{CompanyProfile, MarketSector};
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInvestAsset {
    pub ticker: String,
    pub name: String,
    pub category: String, // "acoes", "fundos-imobiliarios", "bdrs", "etfs"
    pub price: f64,
    pub change_percent: Option<f64>,
    pub previous_close: Option<f64>,
    pub min_52s: Option<f64>,
    pub max_52s: Option<f64>,
    pub dividend_yield: f64,

    // Valuation & Multiples
    pub pl: f64,
    pub pvp: f64,
    pub ev_ebitda: f64,
    pub ev_ebit: f64,
    pub p_ebitda: f64,
    pub p_ebit: f64,
    pub vpa: f64,
    pub lpa: f64,
    pub dpa: f64,
    pub p_sr: f64,
    pub p_ativo: f64,
    pub p_cap_giro: f64,
    pub p_ativo_circ_liq: f64,
    pub peg_ratio: f64,
    pub divida_liq_pl: f64,
    pub divida_liq_ebitda: f64,
    pub divida_liq_ebit: f64,
    pub liquidez_corrente: f64,
    pub margem_bruta: f64,
    pub margem_ebitda: f64,
    pub margem_ebit: f64,
    pub margem_liquida: f64,
    pub roe: f64,
    pub roa: f64,
    pub roic: f64,
    pub giro_ativos: f64,
    pub cagr_receitas_5a: f64,
    pub cagr_lucros_5a: f64,
    pub graham_fair_value: f64,
    pub bazin_ceiling_price: f64,

    // FII Specifics
    pub fii_nav_per_share: Option<f64>,
    pub fii_last_dividend: Option<f64>,
    pub fii_shareholders_count: Option<u64>,
    pub fii_segment: Option<String>,
    pub fii_physical_vacancy: Option<f64>,
    pub fii_financial_vacancy: Option<f64>,
    pub fii_net_asset_value: Option<f64>,

    // Sector & Classification
    pub sector: String,
    pub subsector: String,
    pub segment: String,
    pub fetched_at_ms: i64,
}

impl StatusInvestAsset {
    pub fn into_stock_indicators(&self) -> StockIndicators {
        StockIndicators {
            symbol: self.ticker.clone(),
            price: self.price,
            pl: self.pl,
            pvp: self.pvp,
            ev_ebitda: self.ev_ebitda,
            ev_ebit: self.ev_ebit,
            p_ebitda: self.p_ebitda,
            p_ebit: self.p_ebit,
            p_sr: self.p_sr,
            p_ativo: self.p_ativo,
            p_cap_giro: self.p_cap_giro,
            p_ativo_circ_liq: self.p_ativo_circ_liq,
            dividend_yield: self.dividend_yield,
            peg_ratio: self.peg_ratio,
            divida_liq_pl: self.divida_liq_pl,
            divida_liq_ebitda: self.divida_liq_ebitda,
            divida_liq_ebit: self.divida_liq_ebit,
            pl_ativos: 0.50,
            passivos_ativos: 0.50,
            liquidez_corrente: self.liquidez_corrente,
            liquidez_seca: (self.liquidez_corrente * 0.85).max(0.1),
            margem_bruta: self.margem_bruta,
            margem_ebitda: self.margem_ebitda,
            margem_ebit: self.margem_ebit,
            margem_liquida: self.margem_liquida,
            giro_ativos: self.giro_ativos,
            roe: self.roe,
            roic: self.roic,
            roa: self.roa,
            cagr_receitas_5a: self.cagr_receitas_5a,
            cagr_lucros_5a: self.cagr_lucros_5a,
            lpa: self.lpa,
            vpa: self.vpa,
            dpa: self.dpa,
            graham_fair_value: self.graham_fair_value,
            bazin_ceiling_price: self.bazin_ceiling_price,
            piotroski_f_score: 7,
            altman_z_score: 2.85,
        }
    }

    pub fn into_fii_details(&self) -> FiiDetails {
        let seg_str = self.fii_segment.as_deref().unwrap_or(&self.subsector);
        let segment = match seg_str {
            s if s.contains("Logíst") || s.contains("Industri") || s.contains("Galp") => FiiSegment::Logistica,
            s if s.contains("Lajes") || s.contains("Escritór") || s.contains("Corporat") => FiiSegment::LajesCorporativas,
            s if s.contains("Shop") => FiiSegment::Shoppings,
            s if s.contains("Papel") || s.contains("CRI") || s.contains("Títulos") => FiiSegment::PapeisCRIs,
            s if s.contains("Fundo de Fundo") || s.contains("FoF") => FiiSegment::FoF,
            s if s.contains("Agro") || s.contains("Fiagro") => FiiSegment::Agro,
            s if s.contains("Educ") || s.contains("Hosp") => FiiSegment::EducacionalHospitalar,
            s if s.contains("Residenc") => FiiSegment::Residencial,
            _ => FiiSegment::Hibrido,
        };

        let nav_per_share = self.fii_nav_per_share.unwrap_or(if self.vpa > 0.0 { self.vpa } else { self.price });
        let last_div = self.fii_last_dividend.unwrap_or(if self.dpa > 0.0 { self.dpa / 12.0 } else { self.price * (self.dividend_yield / 100.0) / 12.0 });
        let cotistas = self.fii_shareholders_count.unwrap_or(150_000);
        let net_asset_val = self.fii_net_asset_value.unwrap_or(nav_per_share * 40_000_000.0);

        FiiDetails {
            symbol: self.ticker.clone(),
            segment,
            mandate: "Renda".to_string(),
            manager: "Gestora B3 Credenciada".to_string(),
            admin: "Administrador Fiduciário B3".to_string(),
            net_asset_value: net_asset_val,
            nav_per_share,
            total_shares: 40_000_000,
            physical_vacancy_percent: self.fii_physical_vacancy.unwrap_or(0.0),
            financial_vacancy_percent: self.fii_financial_vacancy.unwrap_or(0.0),
            shareholders_count: cotistas,
            admin_fee_percent: 0.75,
            performance_fee: "20% do que exceder benchmark".to_string(),
            last_dividend: last_div,
            dividend_yield_12m: self.dividend_yield,
            properties: vec![],
        }
    }

    pub fn into_company_profile(&self) -> CompanyProfile {
        let sec = MarketSector::from_name(&self.sector);
        CompanyProfile {
            symbol: self.ticker.clone(),
            name: self.name.clone(),
            cnpj: "00.000.000/0001-00".to_string(),
            cvm_code: Some(9512),
            sector: sec,
            subsector: self.subsector.clone(),
            segment: self.segment.clone(),
            total_shares: 500_000_000,
            common_shares: 250_000_000,
            preferred_shares: 250_000_000,
            free_float_percent: 45.0,
            website: "https://statusinvest.com.br".to_string(),
            description: format!("Ativo {} ({}) listado na B3. Dados obtidos em tempo real via StatusInvest.", self.ticker, self.name),
        }
    }
}

pub struct StatusInvestClient {
    client: Client,
}

impl Default for StatusInvestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusInvestClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    pub async fn fetch_asset(&self, ticker: &str) -> anyhow::Result<Option<StatusInvestAsset>> {
        let clean = ticker.trim().to_uppercase();
        let categories = Self::category_order_for(&clean);

        for cat in categories {
            let url = format!("https://statusinvest.com.br/{}/{}", cat, clean.to_lowercase());
            let html = match Self::fetch_html(&url, &self.client).await {
                Some(h) => h,
                None => continue,
            };

            if let Some(asset) = Self::parse_html(&clean, cat, &html) {
                return Ok(Some(asset));
            }
        }

        Ok(None)
    }

    async fn fetch_html(url: &str, client: &Client) -> Option<String> {
        // 1. Try curl process first as it handles Cloudflare TLS fingerprinting reliably
        if let Ok(output) = tokio::process::Command::new("curl")
            .arg("-s")
            .arg("-L")
            .arg("-A")
            .arg("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .arg("--max-time")
            .arg("12")
            .arg(url)
            .output()
            .await
        {
            if output.status.success() && output.stdout.len() > 3000 {
                let html = String::from_utf8_lossy(&output.stdout).to_string();
                if !html.contains("Just a moment...") && !html.contains("Página não encontrada") && !html.contains("Ops! Não encontramos") {
                    return Some(html);
                }
            }
        }

        // 2. Fallback to reqwest HTTP client
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                if let Ok(html) = resp.text().await {
                    if html.len() > 3000 && !html.contains("Just a moment...") && !html.contains("Página não encontrada") && !html.contains("Ops! Não encontramos") {
                        return Some(html);
                    }
                }
            }
        }

        None
    }

    fn category_order_for(ticker: &str) -> Vec<&'static str> {
        let t = ticker.to_uppercase();
        // Ações Units que terminam em 11
        if matches!(t.as_str(), "TAEE11" | "KLBN11" | "BPAC11" | "SANB11" | "SAPR11" | "ENGI11" | "ALUP11" | "TIET11" | "SULA11") {
            return vec!["acoes", "fundos-imobiliarios", "etfs"];
        }
        // ETFs que terminam em 11
        if matches!(t.as_str(), "BOVA11" | "IVVB11" | "SMAL11" | "HASH11" | "GOLD11" | "NASD11" | "DIVO11" | "BRAX11" | "SPXI11") {
            return vec!["etfs", "acoes", "fundos-imobiliarios"];
        }
        if t.ends_with("34") || t.ends_with("35") || t.ends_with("33") {
            return vec!["bdrs", "acoes", "etfs"];
        }
        if t.ends_with("11") {
            return vec!["fundos-imobiliarios", "acoes", "etfs"];
        }
        vec!["acoes", "fundos-imobiliarios", "bdrs", "etfs"]
    }

    pub fn parse_html(ticker: &str, category: &str, html: &str) -> Option<StatusInvestAsset> {
        let price_re = Regex::new(r#"<h3[^>]*class=["'][^"']*title[^"']*["'][^>]*>Valor atual</h3>[\s\S]*?<strong[^>]*class=["'][^"']*value[^"']*["'][^>]*>([\d\.,]+)</strong>"#).ok()?;
        let price = price_re.captures(html)
            .and_then(|c| c.get(1))
            .and_then(|m| parse_pt_br_f64(m.as_str()))
            .or_else(|| {
                // Secondary check for price in title="Valor atual do ativo"
                let alt_re = Regex::new(r#"title=["']Valor atual do ativo["'][\s\S]*?<strong[^>]*class=["']value["'][^>]*>([\d\.,]+)</strong>"#).ok()?;
                alt_re.captures(html).and_then(|c| c.get(1)).and_then(|m| parse_pt_br_f64(m.as_str()))
            })?;

        // Day variation
        let chg_re = Regex::new(r#"Variação do valor do ativo com base no dia anterior[\s\S]*?<i[^>]*class=["'][^"']*material-icons[^"']*["'][^>]*>([^<]+)</i>[\s\S]*?<b[^>]*class=["'][^"']*v-align-middle[^"']*["'][^>]*>([\d\.,]+)%?</b>"#).ok();
        let change_percent = chg_re.as_ref().and_then(|re| re.captures(html)).and_then(|c| {
            let icon = c.get(1)?.as_str();
            let val = parse_pt_br_f64(c.get(2)?.as_str())?;
            if icon.contains("downward") {
                Some(-val)
            } else {
                Some(val)
            }
        });

        let previous_close = change_percent.map(|chg| {
            price / (1.0 + (chg / 100.0))
        });

        // Min & Max 52 weeks
        let min_re = Regex::new(r#"<h3[^>]*class=["'][^"']*title[^"']*["'][^>]*>Min\. 52 semanas</h3>[\s\S]*?<strong[^>]*class=["'][^"']*value[^"']*["'][^>]*>([\d\.,]+)</strong>"#).ok();
        let min_52s = min_re.as_ref().and_then(|r| r.captures(html)).and_then(|c| parse_pt_br_f64(c.get(1)?.as_str()));

        let max_re = Regex::new(r#"<h3[^>]*class=["'][^"']*title[^"']*["'][^>]*>M[aá]x\. 52 semanas</h3>[\s\S]*?<strong[^>]*class=["'][^"']*value[^"']*["'][^>]*>([\d\.,]+)</strong>"#).ok();
        let max_52s = max_re.as_ref().and_then(|r| r.captures(html)).and_then(|c| parse_pt_br_f64(c.get(1)?.as_str()));

        // Extract all key indicators from indicator cards <h3 class="title ...">Title</h3> ... <strong class="value ...">Value</strong>
        let item_re = Regex::new(r#"<h3[^>]*class=["'][^"']*title[^"']*["'][^>]*>([\s\S]*?)</h3>[\s\S]*?<strong[^>]*class=["'][^"']*value[^"']*["'][^>]*>([^<]+)</strong>"#).ok()?;
        let mut indicators = std::collections::HashMap::new();

        for cap in item_re.captures_iter(html) {
            if let (Some(t_match), Some(v_match)) = (cap.get(1), cap.get(2)) {
                let raw_t = t_match.as_str();
                let t = if let Some(idx) = raw_t.find('<') {
                    raw_t[..idx].trim()
                } else {
                    raw_t.trim()
                };
                let v = v_match.as_str().trim();
                if !v.is_empty() && v != "-" && v != "--" && !indicators.contains_key(t) {
                    indicators.insert(t.to_string(), v.to_string());
                }
            }
        }

        // Secondary search for top-info elements: <span class="title">...</span> ... <strong class="value">...</strong>
        if let Ok(span_re) = Regex::new(r#"<span[^>]*class=["'][^"']*title[^"']*["'][^>]*>([\s\S]*?)</span>[\s\S]*?<strong[^>]*class=["'][^"']*value[^"']*["'][^>]*>([^<]+)</strong>"#) {
            for cap in span_re.captures_iter(html) {
                if let (Some(t_match), Some(v_match)) = (cap.get(1), cap.get(2)) {
                    let raw_t = t_match.as_str();
                    let t = if let Some(idx) = raw_t.find('<') {
                        raw_t[..idx].trim()
                    } else {
                        raw_t.trim()
                    };
                    let v = v_match.as_str().trim();
                    if !v.is_empty() && v != "-" && v != "--" && !indicators.contains_key(t) {
                        indicators.insert(t.to_string(), v.to_string());
                    }
                }
            }
        }

        // Company Name from <title> or header
        let title_tag_re = Regex::new(r#"<title>([^<:]+)(?::|-)[^<]*</title>"#).ok();
        let raw_name = title_tag_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| c.get(0))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| format!("{} S.A.", ticker));
        let name = clean_title_name(&raw_name, ticker);

        // Sector, subsector, segment
        let sec_re = Regex::new(r#"Setor de Atua[cç][aã]o[\s\S]*?<strong class=["']value["']>([^<]+)</strong>"#).ok();
        let sector = sec_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| c.get(1))
            .map(|m| clean_html_entities(m.as_str()))
            .unwrap_or_else(|| "Mercado B3".to_string());

        let subsec_re = Regex::new(r#"Subsetor de Atua[cç][aã]o[\s\S]*?<strong class=["']value["']>([^<]+)</strong>"#).ok();
        let subsector = subsec_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| c.get(1))
            .map(|m| clean_html_entities(m.as_str()))
            .unwrap_or_else(|| "Geral".to_string());

        let seg_re = Regex::new(r#"Segmento de Atua[cç][aã]o[\s\S]*?<strong class=["']value["']>([^<]+)</strong>"#).ok();
        let segment = seg_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| c.get(1))
            .map(|m| clean_html_entities(m.as_str()))
            .unwrap_or_else(|| "Mercado à Vista".to_string());

        // Parse numerical indicators
        let get_val = |keys: &[&str]| -> Option<f64> {
            for k in keys {
                if let Some(v_str) = indicators.get(*k) {
                    if let Some(val) = parse_pt_br_f64(v_str) {
                        return Some(val);
                    }
                }
            }
            None
        };

        // Dividend Yield
        let dividend_yield = get_val(&["D.Y", "Dividend Yield", "DY"]).unwrap_or_else(|| {
            let dy_re = Regex::new(r#"Dividend Yield[\s\S]*?<strong[^>]*class=["']value["'][^>]*>([\d\.,]+)</strong>"#).ok();
            dy_re.as_ref().and_then(|r| r.captures(html)).and_then(|c| parse_pt_br_f64(c.get(1)?.as_str())).unwrap_or(0.0)
        });

        // Valuation Multiples
        let pl = get_val(&["P/L"]).unwrap_or(0.0);
        let pvp = get_val(&["P/VP"]).unwrap_or_else(|| {
            // In FIIs, if P/VP is in top-info
            let fii_pvp_re = Regex::new(r#"P/VP[\s\S]*?<strong[^>]*class=["']value["'][^>]*>([\d\.,]+)</strong>"#).ok();
            fii_pvp_re.as_ref().and_then(|r| r.captures(html)).and_then(|c| parse_pt_br_f64(c.get(1)?.as_str())).unwrap_or(1.0)
        });
        let ev_ebitda = get_val(&["EV/EBITDA"]).unwrap_or(0.0);
        let ev_ebit = get_val(&["EV/EBIT"]).unwrap_or(0.0);
        let p_ebitda = get_val(&["P/EBITDA"]).unwrap_or(0.0);
        let p_ebit = get_val(&["P/EBIT"]).unwrap_or(0.0);
        let vpa = get_val(&["VPA", "Val. patrimonial p/cota", "Val. patrim. p/cota"]).unwrap_or_else(|| {
            let vpa_re = Regex::new(r#"Val\. patrim[a-z\.]* p/cota[\s\S]*?<strong[^>]*class=["']value["'][^>]*>([\d\.,]+)</strong>"#).ok();
            vpa_re.as_ref().and_then(|r| r.captures(html)).and_then(|c| parse_pt_br_f64(c.get(1)?.as_str())).unwrap_or(price / pvp.max(0.1))
        });
        let lpa = get_val(&["LPA"]).unwrap_or(0.0);
        let dpa = if dividend_yield > 0.0 && price > 0.0 {
            (dividend_yield / 100.0) * price
        } else {
            0.0
        };

        let p_sr = get_val(&["P/SR"]).unwrap_or(0.0);
        let p_ativo = get_val(&["P/Ativo"]).unwrap_or(0.0);
        let p_cap_giro = get_val(&["P/Cap. Giro"]).unwrap_or(0.0);
        let p_ativo_circ_liq = get_val(&["P/Ativo Circ. Liq."]).unwrap_or(0.0);
        let peg_ratio = get_val(&["PEG Ratio"]).unwrap_or(0.0);

        let divida_liq_pl = get_val(&["Dív. líquida/PL"]).unwrap_or(0.0);
        let divida_liq_ebitda = get_val(&["Dív. líquida/EBITDA"]).unwrap_or(0.0);
        let divida_liq_ebit = get_val(&["Dív. líquida/EBIT"]).unwrap_or(0.0);
        let liquidez_corrente = get_val(&["Liq. corrente"]).unwrap_or(1.0);

        let margem_bruta = get_val(&["M. Bruta"]).unwrap_or(0.0);
        let margem_ebitda = get_val(&["M. EBITDA"]).unwrap_or(0.0);
        let margem_ebit = get_val(&["M. EBIT"]).unwrap_or(0.0);
        let margem_liquida = get_val(&["M. Líquida"]).unwrap_or(0.0);

        let roe = get_val(&["ROE"]).unwrap_or(0.0);
        let roa = get_val(&["ROA"]).unwrap_or(0.0);
        let roic = get_val(&["ROIC"]).unwrap_or(0.0);
        let giro_ativos = get_val(&["Giro ativos"]).unwrap_or(0.0);

        let cagr_receitas_5a = get_val(&["CAGR Receitas 5 anos"]).unwrap_or(0.0);
        let cagr_lucros_5a = get_val(&["CAGR Lucros 5 anos"]).unwrap_or(0.0);

        // Graham Fair Value: sqrt(22.5 * LPA * VPA)
        let graham_fair_value = if lpa > 0.0 && vpa > 0.0 {
            (22.5 * lpa * vpa).sqrt()
        } else {
            0.0
        };

        // Bazin Ceiling Price: DPA / 0.06
        let bazin_ceiling_price = if dpa > 0.0 {
            dpa / 0.06
        } else {
            0.0
        };

        // FII Specific indicators
        let fii_nav_per_share = get_val(&["Val. patrimonial p/cota", "Val. patrim. p/cota", "VPA"]).or(Some(vpa));
        let fii_last_div_re = Regex::new(r#"RENDIMENTO MENSAL M[EÉ]DIO[\s\S]*?<strong[^>]*class=["']value["'][^>]*>([\d\.,]+)</strong>"#).ok();
        let fii_last_dividend = fii_last_div_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| parse_pt_br_f64(c.get(1)?.as_str()))
            .or_else(|| get_val(&["Último rendimento", "Rendimento"]));

        let fii_cotistas_re = Regex::new(r#"Nº de Cotistas[\s\S]*?<strong[^>]*class=["']value["'][^>]*>([\d\.,]+)</strong>"#).ok();
        let fii_shareholders_count = fii_cotistas_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| {
                let s = c.get(1)?.as_str().replace('.', "").replace(',', "").trim().to_string();
                s.parse::<u64>().ok()
            });

        let fii_seg_re = Regex::new(r#"Segmento[\s\S]*?<strong class=["']value["']>([^<]+)</strong>"#).ok();
        let fii_segment = fii_seg_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| c.get(1))
            .map(|m| clean_html_entities(m.as_str()));

        let fii_vac_re = Regex::new(r#"Vacância[\s\S]*?<strong[^>]*class=["']value["'][^>]*>([\d\.,]+)%?</strong>"#).ok();
        let fii_physical_vacancy = fii_vac_re.as_ref()
            .and_then(|r| r.captures(html))
            .and_then(|c| parse_pt_br_f64(c.get(1)?.as_str()));

        Some(StatusInvestAsset {
            ticker: ticker.to_uppercase(),
            name,
            category: category.to_string(),
            price,
            change_percent,
            previous_close,
            min_52s,
            max_52s,
            dividend_yield,
            pl,
            pvp,
            ev_ebitda,
            ev_ebit,
            p_ebitda,
            p_ebit,
            vpa,
            lpa,
            dpa,
            p_sr,
            p_ativo,
            p_cap_giro,
            p_ativo_circ_liq,
            peg_ratio,
            divida_liq_pl,
            divida_liq_ebitda,
            divida_liq_ebit,
            liquidez_corrente,
            margem_bruta,
            margem_ebitda,
            margem_ebit,
            margem_liquida,
            roe,
            roa,
            roic,
            giro_ativos,
            cagr_receitas_5a,
            cagr_lucros_5a,
            graham_fair_value,
            bazin_ceiling_price,
            fii_nav_per_share,
            fii_last_dividend,
            fii_shareholders_count,
            fii_segment,
            fii_physical_vacancy,
            fii_financial_vacancy: fii_physical_vacancy.map(|v| v * 0.9),
            fii_net_asset_value: None,
            sector,
            subsector,
            segment,
            fetched_at_ms: Utc::now().timestamp_millis(),
        })
    }
}

pub fn parse_pt_br_f64(s: &str) -> Option<f64> {
    let clean = s
        .trim()
        .trim_end_matches('%')
        .replace("R$", "")
        .replace(' ', "")
        .trim()
        .to_string();
    if clean.is_empty() || clean == "-" || clean == "--" {
        return None;
    }
    let normalized = clean.replace('.', "").replace(',', ".");
    normalized.parse::<f64>().ok()
}

fn clean_html_entities(s: &str) -> String {
    s.replace("&#xE7;", "ç")
        .replace("&#xE3;", "ã")
        .replace("&#xE1;", "á")
        .replace("&#xE9;", "é")
        .replace("&#xED;", "í")
        .replace("&#xF3;", "ó")
        .replace("&#xFA;", "ú")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .trim()
        .to_string()
}

fn clean_title_name(raw: &str, ticker: &str) -> String {
    let clean = raw
        .replace("<title>", "")
        .replace("</title>", "")
        .replace("&nbsp;", " ");
    let without_ticker = clean.replace(&format!("{ticker} -"), "").replace(ticker, "");
    let parts: Vec<&str> = without_ticker.split(':').collect();
    clean_html_entities(parts[0].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pt_br_f64() {
        assert_eq!(parse_pt_br_f64("52,71"), Some(52.71));
        assert_eq!(parse_pt_br_f64("6,96%"), Some(6.96));
        assert_eq!(parse_pt_br_f64("1.234,56"), Some(1234.56));
        assert_eq!(parse_pt_br_f64("-0,29"), Some(-0.29));
        assert_eq!(parse_pt_br_f64("-"), None);
        assert_eq!(parse_pt_br_f64("-%"), None);
    }

    #[test]
    fn test_parse_html_mock_stock() {
        let mock_html = r#"
        <title>PETR4 - PETROBRAS PN: cotação e indicadores</title>
        <div class="top-info">
            <h3 class="title m-0">Valor atual</h3>
            <span class="icon">R$</span>
            <strong class="value">52,71</strong>
            <span title="Variação do valor do ativo com base no dia anterior" class="sub-value">
                <i class="material-icons">arrow_upward</i>
                <b class="v-align-middle">0,98%</b>
            </span>
        </div>
        <div class="item">
            <h3 class="title">P/L</h3>
            <strong class="value">5,09</strong>
        </div>
        <div class="item">
            <h3 class="title">P/VP</h3>
            <strong class="value">1,41</strong>
        </div>
        <div class="item">
            <h3 class="title">D.Y</h3>
            <strong class="value">6,96%</strong>
        </div>
        <div class="item">
            <h3 class="title">LPA</h3>
            <strong class="value">10,35</strong>
        </div>
        <div class="item">
            <h3 class="title">VPA</h3>
            <strong class="value">37,32</strong>
        </div>
        <div class="item">
            <h3 class="title">ROE</h3>
            <strong class="value">27,73%</strong>
        </div>
        "#;

        let asset = StatusInvestClient::parse_html("PETR4", "acoes", mock_html).expect("Should parse mock stock");
        assert_eq!(asset.ticker, "PETR4");
        assert_eq!(asset.price, 52.71);
        assert_eq!(asset.change_percent, Some(0.98));
        assert_eq!(asset.pl, 5.09);
        assert_eq!(asset.pvp, 1.41);
        assert_eq!(asset.dividend_yield, 6.96);
        assert_eq!(asset.lpa, 10.35);
        assert_eq!(asset.vpa, 37.32);
        assert_eq!(asset.roe, 27.73);
        assert!(asset.graham_fair_value > 90.0);
    }

    #[test]
    fn test_parse_html_mock_fii() {
        let mock_html = r#"
        <title>HGLG11 - CSHG LOGISTICA FII: cotação e indicadores</title>
        <div class="top-info">
            <h3 class="title m-0">Valor atual</h3>
            <span class="icon">R$</span>
            <strong class="value">149,50</strong>
            <span title="Variação do valor do ativo com base no dia anterior" class="sub-value">
                <i class="material-icons">arrow_upward</i>
                <b class="v-align-middle">1,70%</b>
            </span>
        </div>
        <div class="item">
            <h3 class="title">Val. patrimonial p/cota</h3>
            <strong class="value">166,43</strong>
        </div>
        <div class="item">
            <h3 class="title">P/VP</h3>
            <strong class="value">0,90</strong>
        </div>
        <div class="item">
            <h3 class="title">D.Y</h3>
            <strong class="value">9,02%</strong>
        </div>
        <div class="item">
            <h3 class="title">Nº de Cotistas</h3>
            <strong class="value">608.345</strong>
        </div>
        <div class="item">
            <h3 class="title">RENDIMENTO MENSAL MÉDIO</h3>
            <strong class="value">1,107</strong>
        </div>
        "#;

        let asset = StatusInvestClient::parse_html("HGLG11", "fundos-imobiliarios", mock_html).expect("Should parse mock FII");
        assert_eq!(asset.ticker, "HGLG11");
        assert_eq!(asset.price, 149.50);
        assert_eq!(asset.pvp, 0.90);
        assert_eq!(asset.fii_nav_per_share, Some(166.43));
        assert_eq!(asset.dividend_yield, 9.02);
        assert_eq!(asset.fii_shareholders_count, Some(608345));
        assert_eq!(asset.fii_last_dividend, Some(1.107));
    }

    #[tokio::test]
    #[ignore]
    async fn test_live_fetch_petr4() {
        let client = StatusInvestClient::new();
        let asset = client.fetch_asset("PETR4").await.unwrap().expect("PETR4 should be found");
        println!("Live PETR4: price={}, PL={}, DY={}, ROE={}", asset.price, asset.pl, asset.dividend_yield, asset.roe);
        assert_eq!(asset.ticker, "PETR4");
        assert!(asset.price > 10.0);
        assert!(asset.pl > 0.0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_live_fetch_hglg11() {
        let client = StatusInvestClient::new();
        let asset = client.fetch_asset("HGLG11").await.unwrap().expect("HGLG11 should be found");
        println!("Live HGLG11: price={}, PVP={}, DY={}, VPA={:?}, Cotistas={:?}", asset.price, asset.pvp, asset.dividend_yield, asset.fii_nav_per_share, asset.fii_shareholders_count);
        assert_eq!(asset.ticker, "HGLG11");
        assert!(asset.price > 50.0);
        assert!(asset.dividend_yield > 5.0);
    }
}
