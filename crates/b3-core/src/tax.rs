use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationKind {
    SwingTrade,
    DayTrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaxCategory {
    AcoesSwing,
    DayTrade,
    Fii,
    EtfBdr,
}

impl TaxCategory {
    pub fn tax_rate(&self) -> f64 {
        match self {
            TaxCategory::AcoesSwing => 0.15, // 15%
            TaxCategory::DayTrade => 0.20,   // 20%
            TaxCategory::Fii => 0.20,        // 20%
            TaxCategory::EtfBdr => 0.15,     // 15%
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TaxCategory::AcoesSwing => "Ações (Operações Comuns / Swing Trade)",
            TaxCategory::DayTrade => "Day Trade (Todas as Classes)",
            TaxCategory::Fii => "Fundos Imobiliários / Fiagros (FII)",
            TaxCategory::EtfBdr => "ETFs & BDRs",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOperation {
    pub id: String,
    pub date: NaiveDate,
    pub symbol: String,
    pub side: OperationSide,
    pub kind: OperationKind,
    pub quantity: f64,
    pub unit_price: f64,
    pub total_gross: f64,
    pub liquidation_fee: f64, // Taxa de Liquidação B3
    pub registration_fee: f64, // Emolumentos B3
    pub brokerage_fee: f64,    // Corretagem
    pub iss_tax: f64,          // Imposto sobre serviços
    pub irrf: f64,             // IRRF retido na fonte ("dedo-duro")
    pub broker_name: String,
    pub note_number: String,
}

impl TradeOperation {
    pub fn total_costs(&self) -> f64 {
        self.liquidation_fee + self.registration_fee + self.brokerage_fee + self.iss_tax
    }

    pub fn effective_total(&self) -> f64 {
        match self.side {
            // No Buy, custos somam ao desembolso (aumentam o Preço Médio)
            OperationSide::Buy => self.total_gross + self.total_costs(),
            // No Sell, custos diminuem o recebimento (reduzem o ganho tributável)
            OperationSide::Sell => self.total_gross - self.total_costs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub symbol: String,
    pub quantity: f64,
    pub average_price: f64, // Preço Médio (PM) em R$
    pub total_invested: f64, // Quantidade * PM
    pub current_price: f64,
    pub current_value: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyTaxReport {
    pub year: i32,
    pub month: u32,
    pub acoes_swing_sales: f64,
    pub is_acoes_exempt: bool, // Regra dos R$ 20.000,00 de vendas no mês
    pub net_result_acoes_swing: f64,
    pub net_result_day_trade: f64,
    pub net_result_fii: f64,
    pub net_result_etf_bdr: f64,

    // Prejuízos acumulados anteriores compensados
    pub loss_offset_acoes_swing: f64,
    pub loss_offset_day_trade: f64,
    pub loss_offset_fii: f64,
    pub loss_offset_etf_bdr: f64,

    // Prejuízos acumulados restantes para carregar para o próximo mês
    pub carryover_loss_acoes_swing: f64,
    pub carryover_loss_day_trade: f64,
    pub carryover_loss_fii: f64,
    pub carryover_loss_etf_bdr: f64,

    // Base de cálculo tributável
    pub tax_base_acoes_swing: f64,
    pub tax_base_day_trade: f64,
    pub tax_base_fii: f64,
    pub tax_base_etf_bdr: f64,

    // Imposto apurado
    pub tax_due_acoes_swing: f64,
    pub tax_due_day_trade: f64,
    pub tax_due_fii: f64,
    pub tax_due_etf_bdr: f64,

    // Deduções de IRRF na fonte
    pub irrf_deducted: f64,

    // Total a pagar na DARF
    pub darf_code: &'static str, // "6015"
    pub total_darf_due: f64,
    pub due_date: NaiveDate,
}

impl MonthlyTaxReport {
    pub fn due_date_for(year: i32, month: u32) -> NaiveDate {
        // Vencimento: último dia útil do mês subsequente
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let days_in_month = match next_month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                let is_leap = (next_year % 4 == 0 && next_year % 100 != 0) || (next_year % 400 == 0);
                if is_leap { 29 } else { 28 }
            }
            _ => 30,
        };

        let mut date = NaiveDate::from_ymd_opt(next_year, next_month, days_in_month)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 30).unwrap());

        // Se cair no fim de semana, antecipa para sexta-feira
        while date.weekday() == chrono::Weekday::Sat || date.weekday() == chrono::Weekday::Sun {
            date = date.pred_opt().unwrap_or(date);
        }

        date
    }
}
