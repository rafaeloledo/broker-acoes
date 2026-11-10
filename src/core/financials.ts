export const cvmCodes = {
  // BPA (Ativo)
  ATIVO_TOTAL: "1",
  ATIVO_CIRCULANTE: "1.01",
  CAIXA_EQUIVALENTES: "1.01.01",
  APLICACOES_FINANCEIRAS: "1.01.02",
  CONTAS_A_RECEBER: "1.01.03",
  ESTOQUES: "1.01.04",
  ATIVO_NAO_CIRCULANTE: "1.02",
  ATIVO_REALIZAVEL_LP: "1.02.01",
  INVESTIMENTOS: "1.02.02",
  IMOBILIZADO: "1.02.03",
  INTANGIVEL: "1.02.04",

  // BPP (Passivo & PL)
  PASSIVO_TOTAL: "2",
  PASSIVO_CIRCULANTE: "2.01",
  OBRIGACOES_SOCIAIS_TRABALHISTAS: "2.01.01",
  FORNECEDORES: "2.01.02",
  OBRIGACOES_FISCAIS: "2.01.03",
  EMPRESTIMOS_FINANCIAMENTOS_CP: "2.01.04",
  PASSIVO_NAO_CIRCULANTE: "2.02",
  EMPRESTIMOS_FINANCIAMENTOS_LP: "2.02.01",
  PATRIMONIO_LIQUIDO: "2.03",
  CAPITAL_SOCIAL_REALIZADO: "2.03.01",
  RESERVAS_CAPITAL: "2.03.02",
  RESERVAS_LUCRO: "2.03.04",
  LUCROS_PREJUIZOS_ACUMULADOS: "2.03.05",

  // DRE
  RECEITA_LIQUIDA: "3.01",
  CUSTOS_BENS_SERVICOS: "3.02",
  LUCRO_BRUTO: "3.03",
  DESPESAS_RECEITAS_OPERACIONAIS: "3.04",
  DESPESAS_VENDAS: "3.04.01",
  DESPESAS_GERAIS_ADMINISTRATIVAS: "3.04.02",
  OUTRAS_RECEITAS_DESPESAS_OPER: "3.04.05",
  EBIT: "3.05",
  RESULTADO_FINANCEIRO: "3.06",
  RECEITAS_FINANCEIRAS: "3.06.01",
  DESPESAS_FINANCEIRAS: "3.06.02",
  RESULTADO_ANTES_TRIBUTOS: "3.07",
  IMPOSTO_DE_RENDA_CSLL: "3.08",
  LUCRO_LIQUIDO_CONTINUADO: "3.09",
  LUCRO_LIQUIDO_CONSOLIDADO: "3.11",

  // DFC & DVA & Custom
  FLUXO_CAIXA_OPERACIONAL: "6.01",
  FLUXO_CAIXA_INVESTIMENTO: "6.02",
  FLUXO_CAIXA_FINANCIAMENTO: "6.03",
  DIVIDENDOS_PAGOS_ALT: "6.03.04",
  DIVIDENDOS_PAGOS: "6.03.05",
  VARIACAO_LIQUIDA_CAIXA: "6.05",
  DEPRECIACAO_AMORTIZACAO: "7.04",
  VALOR_ADICIONADO_TOTAL: "7.08",
  SHARES_TOTAL: "SHARES",
} as const;

export const CORE_ACCOUNTS: readonly string[] = [
  cvmCodes.LUCRO_LIQUIDO_CONSOLIDADO,
  cvmCodes.LUCRO_LIQUIDO_CONTINUADO,
  cvmCodes.PATRIMONIO_LIQUIDO,
  cvmCodes.DIVIDENDOS_PAGOS,
  cvmCodes.DIVIDENDOS_PAGOS_ALT,
  cvmCodes.SHARES_TOTAL,
];

export type PeriodKind = "Q1" | "Q2" | "Q3" | "Q4" | "Annual" | "TTM";

export class FinancialPeriod {
  year: number;
  kind: PeriodKind;
  dateStart: string; // YYYY-MM-DD
  dateEnd: string;   // YYYY-MM-DD
  accounts: Record<string, number>;

  constructor(
    year: number,
    kind: PeriodKind,
    dateStart: string,
    dateEnd: string,
    accounts: Record<string, number> = {}
  ) {
    this.year = year;
    this.kind = kind;
    this.dateStart = dateStart;
    this.dateEnd = dateEnd;
    this.accounts = { ...accounts };
  }

  getAccount(code: string): number | undefined {
    return this.accounts[code];
  }

  getAccountOrZero(code: string): number {
    return this.accounts[code] ?? 0.0;
  }

  receitaLiquida(): number {
    return this.getAccountOrZero(cvmCodes.RECEITA_LIQUIDA);
  }

  lucroBruto(): number {
    return this.getAccountOrZero(cvmCodes.LUCRO_BRUTO);
  }

  ebit(): number {
    return this.getAccountOrZero(cvmCodes.EBIT);
  }

  lucroLiquido(): number {
    return (
      this.getAccount(cvmCodes.LUCRO_LIQUIDO_CONSOLIDADO) ??
      this.getAccountOrZero(cvmCodes.LUCRO_LIQUIDO_CONTINUADO)
    );
  }

  patrimonioLiquido(): number {
    return this.getAccountOrZero(cvmCodes.PATRIMONIO_LIQUIDO);
  }

  ativoTotal(): number {
    return this.getAccountOrZero(cvmCodes.ATIVO_TOTAL);
  }

  ativoCirculante(): number {
    return this.getAccountOrZero(cvmCodes.ATIVO_CIRCULANTE);
  }

  passivoCirculante(): number {
    return this.getAccountOrZero(cvmCodes.PASSIVO_CIRCULANTE);
  }

  fluxoCaixaOperacional(): number {
    return this.getAccountOrZero(cvmCodes.FLUXO_CAIXA_OPERACIONAL);
  }

  caixaTotal(): number {
    return (
      this.getAccountOrZero(cvmCodes.CAIXA_EQUIVALENTES) +
      this.getAccountOrZero(cvmCodes.APLICACOES_FINANCEIRAS)
    );
  }

  dividaBruta(): number {
    return (
      this.getAccountOrZero(cvmCodes.EMPRESTIMOS_FINANCIAMENTOS_CP) +
      this.getAccountOrZero(cvmCodes.EMPRESTIMOS_FINANCIAMENTOS_LP)
    );
  }

  dividaLiquida(): number {
    return this.dividaBruta() - this.caixaTotal();
  }
}
