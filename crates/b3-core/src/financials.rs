use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

pub mod cvm_codes {
    // Balanço Patrimonial Ativo (BPA)
    pub const ATIVO_TOTAL: &str = "1";
    pub const ATIVO_CIRCULANTE: &str = "1.01";
    pub const CAIXA_EQUIVALENTES: &str = "1.01.01";
    pub const APLICACOES_FINANCEIRAS: &str = "1.01.02";
    pub const CONTAS_A_RECEBER: &str = "1.01.03";
    pub const ESTOQUES: &str = "1.01.04";
    pub const ATIVO_NAO_CIRCULANTE: &str = "1.02";
    pub const ATIVO_REALIZAVEL_LP: &str = "1.02.01";
    pub const INVESTIMENTOS: &str = "1.02.02";
    pub const IMOBILIZADO: &str = "1.02.03";
    pub const INTANGIVEL: &str = "1.02.04";

    // Balanço Patrimonial Passivo (BPP)
    pub const PASSIVO_TOTAL: &str = "2";
    pub const PASSIVO_CIRCULANTE: &str = "2.01";
    pub const OBRIGACOES_SOCIAIS_TRABALHISTAS: &str = "2.01.01";
    pub const FORNECEDORES: &str = "2.01.02";
    pub const OBRIGACOES_FISCAIS: &str = "2.01.03";
    pub const EMPRESTIMOS_FINANCIAMENTOS_CP: &str = "2.01.04";
    pub const PASSIVO_NAO_CIRCULANTE: &str = "2.02";
    pub const EMPRESTIMOS_FINANCIAMENTOS_LP: &str = "2.02.01";
    pub const PATRIMONIO_LIQUIDO: &str = "2.03";
    pub const CAPITAL_SOCIAL_REALIZADO: &str = "2.03.01";
    pub const RESERVAS_CAPITAL: &str = "2.03.02";
    pub const RESERVAS_LUCRO: &str = "2.03.04";
    pub const LUCROS_PREJUIZOS_ACUMULADOS: &str = "2.03.05";

    // Demonstração do Resultado do Exercício (DRE)
    pub const RECEITA_LIQUIDA: &str = "3.01";
    pub const CUSTOS_BENS_SERVICOS: &str = "3.02";
    pub const LUCRO_BRUTO: &str = "3.03";
    pub const DESPESAS_RECEITAS_OPERACIONAIS: &str = "3.04";
    pub const DESPESAS_VENDAS: &str = "3.04.01";
    pub const DESPESAS_GERAIS_ADMINISTRATIVAS: &str = "3.04.02";
    pub const OUTRAS_RECEITAS_DESPESAS_OPER: &str = "3.04.05";
    pub const EBIT: &str = "3.05"; // Resultado Antes do Resultado Financeiro e Tributos
    pub const RESULTADO_FINANCEIRO: &str = "3.06";
    pub const RECEITAS_FINANCEIRAS: &str = "3.06.01";
    pub const DESPESAS_FINANCEIRAS: &str = "3.06.02";
    pub const RESULTADO_ANTES_TRIBUTOS: &str = "3.07"; // LAIR / EBT
    pub const IMPOSTO_DE_RENDA_CSLL: &str = "3.08";
    pub const LUCRO_LIQUIDO_CONTINUADO: &str = "3.09";
    pub const LUCRO_LIQUIDO_CONSOLIDADO: &str = "3.11"; // Lucro Líquido do Período
    pub const DIVIDENDOS_PAGOS: &str = "6.03.05";
    pub const DIVIDENDOS_PAGOS_ALT: &str = "6.03.04";
    /// Quantidade de ações gravada pelo ingest a partir da composição de capital, não é código CVM.
    pub const SHARES_TOTAL: &str = "SHARES";

    /// Contas persistidas para o primeiro conjunto de indicadores.
    pub const CORE_ACCOUNTS: &[&str] = &[
        LUCRO_LIQUIDO_CONSOLIDADO,
        LUCRO_LIQUIDO_CONTINUADO,
        PATRIMONIO_LIQUIDO,
        DIVIDENDOS_PAGOS,
        DIVIDENDOS_PAGOS_ALT,
        SHARES_TOTAL,
    ];

    // Demonstração dos Fluxos de Caixa (DFC)
    pub const FLUXO_CAIXA_OPERACIONAL: &str = "6.01"; // FCO
    pub const FLUXO_CAIXA_INVESTIMENTO: &str = "6.02";  // FCI
    pub const FLUXO_CAIXA_FINANCIAMENTO: &str = "6.03"; // FCF
    pub const VARIACAO_LIQUIDA_CAIXA: &str = "6.05";

    // Demonstração do Valor Adicionado (DVA)
    pub const VALOR_ADICIONADO_TOTAL: &str = "7.08";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PeriodKind {
    Q1,
    Q2,
    Q3,
    Q4,
    Annual,
    TTM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialPeriod {
    pub year: i32,
    pub kind: PeriodKind,
    pub date_start: chrono::NaiveDate,
    pub date_end: chrono::NaiveDate,
    pub accounts: BTreeMap<String, f64>,
}

impl FinancialPeriod {
    pub fn get_account(&self, code: &str) -> Option<f64> {
        self.accounts.get(code).copied()
    }

    pub fn get_account_or_zero(&self, code: &str) -> f64 {
        self.accounts.get(code).copied().unwrap_or(0.0)
    }

    pub fn receita_liquida(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::RECEITA_LIQUIDA)
    }

    pub fn lucro_bruto(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::LUCRO_BRUTO)
    }

    pub fn ebit(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::EBIT)
    }

    pub fn lucro_liquido(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::LUCRO_LIQUIDO_CONSOLIDADO)
    }

    pub fn patrimonio_liquido(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::PATRIMONIO_LIQUIDO)
    }

    pub fn ativo_total(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::ATIVO_TOTAL)
    }

    pub fn ativo_circulante(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::ATIVO_CIRCULANTE)
    }

    pub fn passivo_circulante(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::PASSIVO_CIRCULANTE)
    }

    pub fn caixa_total(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::CAIXA_EQUIVALENTES)
            + self.get_account_or_zero(cvm_codes::APLICACOES_FINANCEIRAS)
    }

    pub fn divida_bruta(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::EMPRESTIMOS_FINANCIAMENTOS_CP)
            + self.get_account_or_zero(cvm_codes::EMPRESTIMOS_FINANCIAMENTOS_LP)
    }

    pub fn divida_liquida(&self) -> f64 {
        self.divida_bruta() - self.caixa_total()
    }

    pub fn fluxo_caixa_operacional(&self) -> f64 {
        self.get_account_or_zero(cvm_codes::FLUXO_CAIXA_OPERACIONAL)
    }
}
