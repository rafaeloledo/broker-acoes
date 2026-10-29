use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetClass {
    Acao,
    Fii,
    Bdr,
    Etf,
    Opcao,
    Tesouro,
    Indice,
}

impl AssetClass {
    pub fn from_symbol(symbol: &str) -> Self {
        let s = symbol.to_uppercase();
        if s.starts_with("IBOV") || s.starts_with("IFIX") || s.starts_with("SMLL") {
            AssetClass::Indice
        } else if s.ends_with("11") {
            // Can be FII, ETF or BDR/Unit
            // Typically FIIs or ETFs
            if s.starts_with("BOVA") || s.starts_with("IVVB") || s.starts_with("SMAL") || s.starts_with("HASH") {
                AssetClass::Etf
            } else if s.starts_with("HGLG") || s.starts_with("KNCR") || s.starts_with("XPML") || s.starts_with("BTLG") || s.starts_with("MXRF") || s.starts_with("VISC") {
                AssetClass::Fii
            } else {
                AssetClass::Fii
            }
        } else if s.ends_with("33") || s.ends_with("34") || s.ends_with("35") {
            AssetClass::Bdr
        } else if s.ends_with("3") || s.ends_with("4") || s.ends_with("5") || s.ends_with("6") {
            AssetClass::Acao
        } else {
            AssetClass::Acao
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketSector {
    Financeiro,
    PetroleoGasBiocombustiveis,
    MateriaisBasicos,
    UtilidadePublica,
    ConsumoNaoCiclico,
    ConsumoCiclico,
    BensIndustriais,
    Saude,
    TecnologiaInformacao,
    Comunicacoes,
    Imobiliario,
    Outros,
}

impl MarketSector {
    pub fn display_name(&self) -> &'static str {
        match self {
            MarketSector::Financeiro => "Financeiro & Seguros",
            MarketSector::PetroleoGasBiocombustiveis => "Petróleo, Gás e Biocombustíveis",
            MarketSector::MateriaisBasicos => "Materiais Básicos (Mineração/Siderurgia)",
            MarketSector::UtilidadePublica => "Utilidade Pública (Energia/Saneamento)",
            MarketSector::ConsumoNaoCiclico => "Consumo Não Cíclico (Alimentos/Bebidas)",
            MarketSector::ConsumoCiclico => "Consumo Cíclico (Varejo/Construção)",
            MarketSector::BensIndustriais => "Bens Industriais (Maquinário/Transporte)",
            MarketSector::Saude => "Saúde (Hospitais/Farmácias)",
            MarketSector::TecnologiaInformacao => "Tecnologia da Informação",
            MarketSector::Comunicacoes => "Comunicações e Telecom",
            MarketSector::Imobiliario => "Imobiliário (FIIs e Shoppings)",
            MarketSector::Outros => "Outros Setores",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub symbol: String,
    pub name: String,
    pub cnpj: String,
    pub cvm_code: Option<u32>,
    pub sector: MarketSector,
    pub subsector: String,
    pub segment: String,
    pub total_shares: u64,
    pub common_shares: u64,
    pub preferred_shares: u64,
    pub free_float_percent: f64,
    pub website: String,
    pub description: String,
}

impl CompanyProfile {
    pub fn default_for(symbol: &str) -> Self {
        let sym = symbol.to_uppercase();
        let (name, sector, subsector, shares) = match sym.as_str() {
            "PETR4" | "PETR3" => (
                "Petróleo Brasileiro S.A. - Petrobras",
                MarketSector::PetroleoGasBiocombustiveis,
                "Exploração, Refino e Distribuição",
                13_044_496_930,
            ),
            "VALE3" => (
                "Vale S.A.",
                MarketSector::MateriaisBasicos,
                "Minerais Metálicos",
                4_539_007_568,
            ),
            "WEGE3" => (
                "WEG S.A.",
                MarketSector::BensIndustriais,
                "Motores, Compressores e Geradores",
                4_197_297_600,
            ),
            "ITUB4" | "ITUB3" => (
                "Itaú Unibanco Holding S.A.",
                MarketSector::Financeiro,
                "Bancos Comerciais e Múltiplos",
                9_804_134_442,
            ),
            "BBAS3" => (
                "Banco do Brasil S.A.",
                MarketSector::Financeiro,
                "Bancos Comerciais e Múltiplos",
                5_737_928_828,
            ),
            "TAEE11" => (
                "Transmissora Aliança de Energia Elétrica S.A. - Taesa",
                MarketSector::UtilidadePublica,
                "Transmissão de Energia Elétrica",
                1_033_496_721,
            ),
            "EGIE3" => (
                "Engie Brasil Energia S.A.",
                MarketSector::UtilidadePublica,
                "Geração de Energia Elétrica",
                815_927_740,
            ),
            "HGLG11" => (
                "CSHG Logística Fundo de Investimento Imobiliário",
                MarketSector::Imobiliario,
                "Logística e Galpões Industriais",
                33_787_545,
            ),
            "KNCR11" => (
                "Kinea Rendimentos Imobiliários FII",
                MarketSector::Imobiliario,
                "Títulos e Valores Mobiliários (Papel)",
                57_094_570,
            ),
            "MXRF11" => (
                "Maxi Renda FII",
                MarketSector::Imobiliario,
                "Híbrido e CRIs",
                334_655_903,
            ),
            "XPML11" => (
                "XP Malls FII",
                MarketSector::Imobiliario,
                "Shoppings",
                60_796_275,
            ),
            _ => (
                "Ativo B3",
                MarketSector::Outros,
                "Mercado à Vista",
                100_000_000,
            ),
        };

        Self {
            symbol: sym.clone(),
            name: name.to_string(),
            cnpj: "00.000.000/0001-00".to_string(),
            cvm_code: Some(9512),
            sector,
            subsector: subsector.to_string(),
            segment: "Novo Mercado".to_string(),
            total_shares: shares,
            common_shares: shares / 2,
            preferred_shares: shares / 2,
            free_float_percent: 45.0,
            website: "https://ri.b3.com.br".to_string(),
            description: format!("Empresa listada na B3 ({}) com alta liquidez e cobertura de analistas.", sym),
        }
    }
}
