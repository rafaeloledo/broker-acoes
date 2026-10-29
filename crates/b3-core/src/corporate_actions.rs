use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorporateActionType {
    Dividendo,
    JurosSobreCapitalProprio, // JCP (15% IRRF na fonte)
    Bonificacao,
    Desdobramento, // Stock Split (e.g. 1 -> 4)
    Grupamento,    // Reverse Split (e.g. 10 -> 1)
    Subscricao,
}

impl CorporateActionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            CorporateActionType::Dividendo => "Dividendo",
            CorporateActionType::JurosSobreCapitalProprio => "JCP",
            CorporateActionType::Bonificacao => "Bonificação",
            CorporateActionType::Desdobramento => "Desdobramento",
            CorporateActionType::Grupamento => "Grupamento",
            CorporateActionType::Subscricao => "Subscrição",
        }
    }

    pub fn is_cash_dividend(&self) -> bool {
        matches!(self, CorporateActionType::Dividendo | CorporateActionType::JurosSobreCapitalProprio)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporateAction {
    pub symbol: String,
    pub action_type: CorporateActionType,
    pub date_com: NaiveDate,
    pub date_ex: NaiveDate,
    pub date_payment: Option<NaiveDate>,
    pub value_per_share: f64, // Valor em R$ por ação/cota (líquido ou bruto)
    pub split_ratio_from: f64, // Para split/grupamento
    pub split_ratio_to: f64,
}
