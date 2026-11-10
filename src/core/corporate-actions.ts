export type CorporateActionType =
  | "Dividendo"
  | "JurosSobreCapitalProprio"
  | "Bonificacao"
  | "Desdobramento"
  | "Grupamento"
  | "Subscricao";

export function corporateActionDisplayName(type: CorporateActionType): string {
  switch (type) {
    case "Dividendo":
      return "Dividendo";
    case "JurosSobreCapitalProprio":
      return "JCP (Juros sobre Capital Próprio)";
    case "Bonificacao":
      return "Bonificação em Ações";
    case "Desdobramento":
      return "Desdobramento (Split)";
    case "Grupamento":
      return "Grupamento (Inplit)";
    case "Subscricao":
      return "Direito de Subscrição";
  }
}

export function isCashDividend(type: CorporateActionType): boolean {
  return type === "Dividendo" || type === "JurosSobreCapitalProprio";
}

export interface CorporateAction {
  symbol: string;
  actionType: CorporateActionType;
  dateCom: string; // YYYY-MM-DD
  dateEx: string;  // YYYY-MM-DD
  datePayment: string | null;
  valuePerShare: number;
  splitRatioFrom: number;
  splitRatioTo: number;
}
