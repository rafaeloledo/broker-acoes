use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiiSegment {
    Logistica,
    LajesCorporativas,
    Shoppings,
    PapeisCRIs,
    FoF, // Fund of Funds
    Agro, // Fiagro
    EducacionalHospitalar,
    Residencial,
    Hibrido,
    Outros,
}

impl FiiSegment {
    pub fn display_name(&self) -> &'static str {
        match self {
            FiiSegment::Logistica => "Logística / Galpões",
            FiiSegment::LajesCorporativas => "Lajes Corporativas / Escritórios",
            FiiSegment::Shoppings => "Shoppings Centers",
            FiiSegment::PapeisCRIs => "Títulos e Valores Mobiliários (Papel / CRI)",
            FiiSegment::FoF => "Fundos de Fundos (FoF)",
            FiiSegment::Agro => "Fiagro / Agronegócio",
            FiiSegment::EducacionalHospitalar => "Educacional & Hospitalar",
            FiiSegment::Residencial => "Residencial",
            FiiSegment::Hibrido => "Híbrido / Renda Urbana",
            FiiSegment::Outros => "Outros",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiiProperty {
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub area_m2: f64,
    pub units_count: u32,
    pub tenants_count: u32,
    pub vacancy_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiiDetails {
    pub symbol: String,
    pub segment: FiiSegment,
    pub mandate: String,
    pub manager: String,
    pub admin: String,
    pub net_asset_value: f64, // Valor Patrimonial (VP)
    pub nav_per_share: f64,   // VP por cota (VPA)
    pub total_shares: u64,
    pub physical_vacancy_percent: f64,
    pub financial_vacancy_percent: f64,
    pub shareholders_count: u64,
    pub admin_fee_percent: f64,
    pub performance_fee: String,
    pub last_dividend: f64,
    pub dividend_yield_12m: f64,
    pub properties: Vec<FiiProperty>,
}

impl FiiDetails {
    pub fn default_for(symbol: &str) -> Self {
        let sym = symbol.to_uppercase();
        let (segment, nav_per_share, shares, phys_vac, div) = match sym.as_str() {
            "HGLG11" => (FiiSegment::Logistica, 153.20, 33_787_545, 4.2, 1.10),
            "KNCR11" => (FiiSegment::PapeisCRIs, 101.45, 57_094_570, 0.0, 1.02),
            "MXRF11" => (FiiSegment::PapeisCRIs, 9.85, 334_655_903, 0.0, 0.10),
            "XPML11" => (FiiSegment::Shoppings, 112.50, 60_796_275, 4.8, 0.92),
            "BTLG11" => (FiiSegment::Logistica, 99.80, 42_150_000, 2.1, 0.78),
            _ => (FiiSegment::Hibrido, 100.0, 10_000_000, 5.0, 0.85),
        };

        Self {
            symbol: sym.clone(),
            segment,
            mandate: "Renda".to_string(),
            manager: "Gestora B3 Credenciada".to_string(),
            admin: "Administrador Fiduciário B3".to_string(),
            net_asset_value: nav_per_share * (shares as f64),
            nav_per_share,
            total_shares: shares,
            physical_vacancy_percent: phys_vac,
            financial_vacancy_percent: phys_vac * 0.9,
            shareholders_count: 150_000,
            admin_fee_percent: 0.75,
            performance_fee: "20% do que exceder CDI / IPCA+6%".to_string(),
            last_dividend: div,
            dividend_yield_12m: (div * 12.0 / nav_per_share) * 100.0,
            properties: vec![
                FiiProperty {
                    name: "Centro Logístico Bandeirantes".to_string(),
                    address: "Rodovia dos Bandeirantes, km 38".to_string(),
                    city: "Cajamar".to_string(),
                    state: "SP".to_string(),
                    area_m2: 85_400.0,
                    units_count: 8,
                    tenants_count: 6,
                    vacancy_percent: 0.0,
                },
                FiiProperty {
                    name: "Parque Industrial Betim".to_string(),
                    address: "Av. das Indústrias, 1200".to_string(),
                    city: "Betim".to_string(),
                    state: "MG".to_string(),
                    area_m2: 62_100.0,
                    units_count: 4,
                    tenants_count: 3,
                    vacancy_percent: 6.5,
                },
            ],
        }
    }
}
