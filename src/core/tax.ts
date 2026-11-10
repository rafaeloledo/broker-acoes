export type OperationSide = "Buy" | "Sell";

export type OperationKind = "SwingTrade" | "DayTrade";

export type TaxCategory = "AcoesSwing" | "DayTrade" | "Fii" | "EtfBdr";

export function taxCategoryRate(category: TaxCategory): number {
  switch (category) {
    case "AcoesSwing":
      return 0.15;
    case "DayTrade":
      return 0.20;
    case "Fii":
      return 0.20;
    case "EtfBdr":
      return 0.15;
  }
}

export interface TradeOperation {
  id: string;
  date: string; // YYYY-MM-DD
  symbol: string;
  side: OperationSide;
  kind: OperationKind;
  quantity: number;
  unitPrice: number;
  totalGross: number;
  liquidationFee: number;
  registrationFee: number;
  brokerageFee: number;
  issTax: number;
  irrf: number;
  brokerName: string;
  noteNumber: string;
}

export function tradeTotalCosts(op: TradeOperation): number {
  return op.liquidationFee + op.registrationFee + op.brokerageFee + op.issTax;
}

export function tradeEffectiveTotal(op: TradeOperation): number {
  const costs = tradeTotalCosts(op);
  return op.side === "Buy" ? op.totalGross + costs : op.totalGross - costs;
}

export interface PortfolioPosition {
  symbol: string;
  quantity: number;
  averagePrice: number;
  totalInvested: number;
  currentPrice: number;
  currentValue: number;
  unrealizedPnl: number;
  unrealizedPnlPercent: number;
}

export interface MonthlyTaxReport {
  year: number;
  month: number;
  acoesSwingSales: number;
  isAcoesExempt: boolean;
  netResultAcoesSwing: number;
  netResultDayTrade: number;
  netResultFii: number;
  netResultEtfBdr: number;
  lossOffsetAcoesSwing: number;
  lossOffsetDayTrade: number;
  lossOffsetFii: number;
  lossOffsetEtfBdr: number;
  carryoverLossAcoesSwing: number;
  carryoverLossDayTrade: number;
  carryoverLossFii: number;
  carryoverLossEtfBdr: number;
  taxBaseAcoesSwing: number;
  taxBaseDayTrade: number;
  taxBaseFii: number;
  taxBaseEtfBdr: number;
  taxDueAcoesSwing: number;
  taxDueDayTrade: number;
  taxDueFii: number;
  taxDueEtfBdr: number;
  irrfDeducted: number;
  darfCode: string;
  totalDarfDue: number;
  dueDate: string; // YYYY-MM-DD
}

export function dueDateFor(year: number, month: number): string {
  const nextYear = month === 12 ? year + 1 : year;
  const nextMonth = month === 12 ? 1 : month + 1;

  // Day 0 of (nextMonth + 1) in UTC gives the last calendar day of nextMonth
  const lastDayDate = new Date(Date.UTC(nextYear, nextMonth, 0));
  const dow = lastDayDate.getUTCDay(); // 0 = Sun, 6 = Sat
  if (dow === 6) {
    lastDayDate.setUTCDate(lastDayDate.getUTCDate() - 1);
  } else if (dow === 0) {
    lastDayDate.setUTCDate(lastDayDate.getUTCDate() - 2);
  }
  return lastDayDate.toISOString().slice(0, 10);
}
