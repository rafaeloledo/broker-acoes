import {
  type AssetClass,
  type B3Asset,
  type Candle,
  type CompanyProfile,
  type FiiDetails,
  type MonthlyTaxReport,
  type NewsItem,
  type OrderBook,
  type PortfolioPosition,
  type StockIndicators,
  type Tick,
  type TradeOperation,
} from "../core/index.ts";
import {
  type CoreMultiples,
  type StockTrendAnalysis,
} from "../analytics/index.ts";

export type ActiveView =
  | "Decision"
  | "InputAnalysis"
  | "Trending"
  | "News"
  | "GpuTerminal"
  | "MarketHeatmap"
  | "Screener"
  | "PortfolioTax"
  | "StatusInvest";

export interface AssetSnapshot {
  ticker: string;
  assetClass: AssetClass;
  profile: CompanyProfile;
  indicators: StockIndicators;
  fiiDetails: FiiDetails | null;
  candles: Candle[];
  marketPrice: number | null;
  marketPreviousClose: number | null;
  marketFetchedAtMs: number | null;
  cvmYear: number | null;
  coreMultiples: CoreMultiples | null;
  statusInvestAtMs: number | null;
  statusInvestLoading: boolean;
  statusInvestOffline: boolean;
}

export interface HeatmapNode {
  symbol: string;
  name: string;
  sector: string;
  price: number;
  changePct: number;
  flashIntensity: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface PortfolioSnapshot {
  trades: TradeOperation[];
  positions: PortfolioPosition[];
  monthlyReports: MonthlyTaxReport[];
  totalCustodyValue: number;
  totalInvested: number;
  unrealizedPnl: number;
  unrealizedPnlPercent: number;
}

export interface InitialStatePayload {
  visibleTickers: string[];
  selectedTicker: string;
  assets: Record<string, AssetSnapshot>;
  orderBooks: Record<string, OrderBook>;
  recentTicks: Record<string, Tick[]>;
  heatmapNodes: HeatmapNode[];
  portfolio: PortfolioSnapshot;
  recentNews: NewsItem[];
  trending: StockTrendAnalysis[];
  catalog: readonly B3Asset[];
  isLiveSimRunning: boolean;
}

export type WsServerMessage =
  | {
      type: "tick_batch";
      ticks: Record<string, Tick>;
      orderBooks: Record<string, OrderBook>;
      heatmapNodes: HeatmapNode[];
      prices: Record<
        string,
        { price: number; changePct: number; lastCandle: Candle }
      >;
    }
  | {
      type: "asset_updated";
      asset: AssetSnapshot;
    }
  | {
      type: "watchlist_updated";
      visibleTickers: string[];
      assets: Record<string, AssetSnapshot>;
    }
  | {
      type: "portfolio_updated";
      portfolio: PortfolioSnapshot;
    }
  | {
      type: "news_updated";
      recentNews: NewsItem[];
    };
