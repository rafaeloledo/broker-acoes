import {
  type PortfolioPosition,
  type TradeOperation,
  tradeEffectiveTotal,
} from "../core/tax.ts";

export class PortfolioTracker {
  // symbol -> { quantity, averagePrice }
  private positions = new Map<string, { quantity: number; averagePrice: number }>();

  processTrade(trade: TradeOperation): number {
    const sym = trade.symbol.trim().toUpperCase();
    const current = this.positions.get(sym) ?? { quantity: 0.0, averagePrice: 0.0 };

    if (trade.side === "Buy") {
      const cost = tradeEffectiveTotal(trade);
      const newQty = current.quantity + trade.quantity;
      const newPm =
        newQty > 0.0 ? (current.quantity * current.averagePrice + cost) / newQty : 0.0;
      this.positions.set(sym, { quantity: newQty, averagePrice: newPm });
      return 0.0;
    } else {
      const sellReceived = tradeEffectiveTotal(trade);
      const costBasis = trade.quantity * current.averagePrice;
      const realizedPnl = sellReceived - costBasis;
      const newQty = Math.max(current.quantity - trade.quantity, 0.0);
      const newPm = newQty > 0.0 ? current.averagePrice : 0.0;
      this.positions.set(sym, { quantity: newQty, averagePrice: newPm });
      return realizedPnl;
    }
  }

  getPositions(currentPrices: ReadonlyMap<string, number> | Record<string, number>): PortfolioPosition[] {
    const out: PortfolioPosition[] = [];
    const lookupPrice = (sym: string): number | undefined => {
      if (currentPrices instanceof Map) {
        return (currentPrices as ReadonlyMap<string, number>).get(sym);
      }
      return (currentPrices as Record<string, number>)[sym];
    };

    for (const [symbol, { quantity, averagePrice }] of this.positions.entries()) {
      if (quantity <= 0.0001) continue;
      const currentPrice = lookupPrice(symbol) ?? averagePrice;
      const totalInvested = quantity * averagePrice;
      const currentValue = quantity * currentPrice;
      const unrealizedPnl = currentValue - totalInvested;
      const unrealizedPnlPercent =
        totalInvested > 0.0 ? (unrealizedPnl / totalInvested) * 100.0 : 0.0;

      out.push({
        symbol,
        quantity,
        averagePrice,
        totalInvested,
        currentPrice,
        currentValue,
        unrealizedPnl,
        unrealizedPnlPercent,
      });
    }

    out.sort((a, b) => b.currentValue - a.currentValue);
    return out;
  }
}
