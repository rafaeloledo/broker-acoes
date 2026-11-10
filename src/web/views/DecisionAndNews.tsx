import React, { useMemo, useState } from "react";
import {
  changePercent,
  formatMetric,
  fundamentalsCaption,
  priceCaption,
  type NewsItem,
} from "../../core/index.ts";
import { type ActiveView, type AssetSnapshot } from "../../shared/types.ts";

interface DecisionViewProps {
  asset: AssetSnapshot;
  news: readonly NewsItem[];
  onNavigate: (view: ActiveView) => void;
  onOpenFicha: (ticker: string) => void;
  onOpenTerminal: (ticker: string) => void;
}

export const DecisionView: React.FC<DecisionViewProps> = ({
  asset,
  news,
  onNavigate,
  onOpenFicha,
  onOpenTerminal,
}) => {
  const hasMarketPrice = asset.marketPrice != null && asset.marketPrice > 0;
  const displayPrice = asset.marketPrice ?? asset.indicators.price;
  const chgPct =
    asset.marketPrice != null && asset.marketPreviousClose != null
      ? changePercent(asset.marketPrice, asset.marketPreviousClose)
      : asset.candles.length >= 2
      ? changePercent(
          displayPrice,
          asset.candles[asset.candles.length - 1]?.open ||
            asset.candles[asset.candles.length - 2]!.close
        )
      : null;

  const tickerNews = useMemo(
    () =>
      news
        .filter((n) => n.ticker.toUpperCase() === asset.ticker.toUpperCase())
        .slice(0, 5),
    [news, asset.ticker]
  );

  const core = asset.coreMultiples;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      {/* Focused Asset Header Card */}
      <div className="card">
        <div className="flex-between">
          <div>
            <div className="flex-row" style={{ gap: "8px", marginBottom: "4px" }}>
              <span style={{ fontSize: "26px", fontWeight: 800 }}>
                {asset.ticker}
              </span>
              <span className="badge">{asset.profile.segment}</span>
              <span className="badge">{asset.profile.subsector}</span>
            </div>
            <div className="text-subtle" style={{ fontSize: "15px", fontWeight: 500 }}>
              {asset.profile.name}
            </div>
          </div>

          <div style={{ textAlign: "right" }}>
            <div style={{ fontSize: "26px", fontWeight: 800 }}>
              {displayPrice > 0 ? `R$ ${displayPrice.toFixed(2)}` : "indisponível"}
            </div>
            <div
              className="flex-row"
              style={{ justifyContent: "flex-end", gap: "8px", marginTop: "4px" }}
            >
              {chgPct != null ? (
                <span
                  className={
                    chgPct >= 0 ? "badge badge-positive" : "badge badge-negative"
                  }
                >
                  {chgPct >= 0 ? "+" : ""}
                  {chgPct.toFixed(2)}% vs dia anterior
                </span>
              ) : (
                <span className="text-muted" style={{ fontSize: "12px" }}>
                  variação indisponível
                </span>
              )}
              <span className="badge">
                {priceCaption(
                  hasMarketPrice ? asset.marketFetchedAtMs : null
                )}
              </span>
              <span className="badge">{fundamentalsCaption(asset.cvmYear)}</span>
            </div>
          </div>
        </div>

        <div
          className="flex-between"
          style={{
            marginTop: "14px",
            paddingTop: "12px",
            borderTop: "1px solid var(--nav-border)",
          }}
        >
          <span className="text-muted" style={{ fontSize: "12px" }}>
            Preço de mercado quando a coleta existe. Livro, tape e o pulso do mapa são simulação.
          </span>
          <div className="flex-row">
            <button
              className="btn"
              onClick={() => onOpenTerminal(asset.ticker)}
            >
              📈 Abrir no Terminal
            </button>
            <button className="btn" onClick={() => onOpenFicha(asset.ticker)}>
              📊 Ficha Completa
            </button>
          </div>
        </div>
      </div>

      {/* 5 CVM Valuation Cards */}
      <div>
        <div
          className="flex-between"
          style={{ marginBottom: "8px" }}
        >
          <strong style={{ fontSize: "14px" }}>
            Indicadores de Valuation ({fundamentalsCaption(asset.cvmYear)})
          </strong>
          <span className="text-muted" style={{ fontSize: "12px" }}>
            {core
              ? "Calculado a partir das demonstrações CVM (DFP) e cotação atual"
              : "Sem período CVM carregado — exibindo indisponível para múltiplos CVM estritos"}
          </span>
        </div>
        <div className="grid-5">
          <div className="card-alt">
            <div className="kpi-label">P/L</div>
            <div className="kpi-value">{formatMetric(core, "Pl")}</div>
            <div className="kpi-sub">
              LPA: {formatMetric(core, "Lpa")}
            </div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">P/VP</div>
            <div className="kpi-value">{formatMetric(core, "Pvp")}</div>
            <div className="kpi-sub">
              VPA: {formatMetric(core, "Vpa")}
            </div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">Dividend yield</div>
            <div className="kpi-value text-accent">
              {formatMetric(core, "DividendYield")}
            </div>
            <div className="kpi-sub">
              DPA: {formatMetric(core, "Dpa")}
            </div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">Graham</div>
            <div className="kpi-value">{formatMetric(core, "Graham")}</div>
            <div className="kpi-sub">√(22,5 × LPA × VPA)</div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">Bazin</div>
            <div className="kpi-value">{formatMetric(core, "Bazin")}</div>
            <div className="kpi-sub">DPA / 6% a.a.</div>
          </div>
        </div>
      </div>

      {/* Recent News for Focused Asset */}
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "12px" }}>
          <div>
            <strong style={{ fontSize: "15px" }}>
              Notícias e Fatos Relevantes — {asset.ticker}
            </strong>
            <div className="text-muted" style={{ fontSize: "12px" }}>
              Últimos comunicados CVM (IPE) e manchetes de mercado (RSS)
            </div>
          </div>
          <button className="btn" onClick={() => onNavigate("News")}>
            Ver todas →
          </button>
        </div>

        {tickerNews.length === 0 ? (
          <div
            className="card-inset text-muted"
            style={{ textAlign: "center", padding: "20px" }}
          >
            Nenhuma notícia ou fato relevante registrado para {asset.ticker}.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {tickerNews.map((item) => {
              const dateStr = new Date(item.publishedAtMs).toLocaleDateString(
                "pt-BR",
                {
                  day: "2-digit",
                  month: "2-digit",
                  year: "numeric",
                }
              );
              return (
                <a
                  key={item.url}
                  href={item.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="card-inset"
                  style={{
                    textDecoration: "none",
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    gap: "12px",
                  }}
                >
                  <div>
                    <div
                      style={{
                        fontWeight: 600,
                        color: "var(--link)",
                        marginBottom: "2px",
                      }}
                    >
                      {item.title}
                    </div>
                    {item.summary && (
                      <div
                        className="text-muted"
                        style={{
                          fontSize: "12px",
                          display: "-webkit-box",
                          WebkitLineClamp: 2,
                          WebkitBoxOrient: "vertical",
                          overflow: "hidden",
                        }}
                      >
                        {item.summary}
                      </div>
                    )}
                  </div>
                  <div
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      alignItems: "flex-end",
                      gap: "4px",
                      flexShrink: 0,
                    }}
                  >
                    <span className="badge">{item.source}</span>
                    <span className="text-muted" style={{ fontSize: "11px" }}>
                      {dateStr}
                    </span>
                  </div>
                </a>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

interface NewsViewProps {
  visibleTickers: readonly string[];
  selectedTicker: string;
  news: readonly NewsItem[];
  onSyncNews: () => Promise<void>;
}

type PeriodFilter = "7d" | "30d" | "12m" | "all";

export const NewsView: React.FC<NewsViewProps> = ({
  visibleTickers,
  selectedTicker,
  news,
  onSyncNews,
}) => {
  const [tickerFilter, setTickerFilter] = useState<string>(
    selectedTicker || "TODOS"
  );
  const [textFilter, setTextFilter] = useState("");
  const [period, setPeriod] = useState<PeriodFilter>("all");
  const [syncing, setSyncing] = useState(false);

  const allTickers = useMemo(() => {
    const s = new Set<string>(["TODOS", ...visibleTickers]);
    for (const item of news) {
      s.add(item.ticker.toUpperCase());
    }
    return Array.from(s);
  }, [visibleTickers, news]);

  const filteredNews = useMemo(() => {
    const now = Date.now();
    let minMs: number | null = null;
    if (period === "7d") minMs = now - 7 * 86_400_000;
    else if (period === "30d") minMs = now - 30 * 86_400_000;
    else if (period === "12m") minMs = now - 365 * 86_400_000;

    const q = textFilter.trim().toLowerCase();

    return news.filter((item) => {
      if (
        tickerFilter !== "TODOS" &&
        item.ticker.toUpperCase() !== tickerFilter.toUpperCase()
      ) {
        return false;
      }
      if (minMs != null && item.publishedAtMs < minMs) {
        return false;
      }
      if (q && !item.title.toLowerCase().includes(q)) {
        return false;
      }
      return true;
    });
  }, [news, tickerFilter, textFilter, period]);

  const handleSync = async () => {
    setSyncing(true);
    try {
      await onSyncNews();
    } finally {
      setSyncing(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "12px" }}>
          <div>
            <strong style={{ fontSize: "16px" }}>
              Central de Notícias & Fatos Relevantes CVM
            </strong>
            <div className="text-muted" style={{ fontSize: "12px" }}>
              Busca por ativo, palavras-chave no título e janela de tempo
            </div>
          </div>
          <button className="btn" onClick={handleSync} disabled={syncing}>
            {syncing ? "🔄 Sincronizando..." : "🔄 Atualizar Feeds (CVM + RSS)"}
          </button>
        </div>

        {/* Ticker chips */}
        <div className="flex-row" style={{ marginBottom: "10px" }}>
          {allTickers.map((t) => (
            <button
              key={t}
              className={`chip ${tickerFilter === t ? "active" : ""}`}
              onClick={() => setTickerFilter(t)}
            >
              {t === "TODOS" ? "Todos os Ativos" : t}
            </button>
          ))}
        </div>

        {/* Search & Period row */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr auto",
            gap: "10px",
            alignItems: "center",
          }}
        >
          <input
            type="text"
            className="input"
            placeholder="Filtrar por palavras no título (ex: dividendo, lucro, aquisição)..."
            value={textFilter}
            onChange={(e) => setTextFilter(e.target.value)}
          />
          <div className="flex-row">
            {(
              [
                ["7d", "7 dias"],
                ["30d", "30 dias"],
                ["12m", "12 meses"],
                ["all", "Tudo"],
              ] as const
            ).map(([id, label]) => (
              <button
                key={id}
                className={`chip ${period === id ? "active" : ""}`}
                onClick={() => setPeriod(id)}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="card">
        <div className="flex-between" style={{ marginBottom: "10px" }}>
          <strong>Resultados ({filteredNews.length})</strong>
        </div>
        {filteredNews.length === 0 ? (
          <div
            className="card-inset text-muted"
            style={{ textAlign: "center", padding: "28px" }}
          >
            Nenhuma notícia encontrada para os filtros selecionados.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {filteredNews.map((item) => {
              const dt = new Date(item.publishedAtMs).toLocaleDateString(
                "pt-BR",
                {
                  day: "2-digit",
                  month: "2-digit",
                  year: "numeric",
                }
              );
              return (
                <a
                  key={item.url}
                  href={item.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="card-inset"
                  style={{
                    textDecoration: "none",
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "flex-start",
                    gap: "12px",
                  }}
                >
                  <div>
                    <div
                      className="flex-row"
                      style={{ gap: "6px", marginBottom: "4px" }}
                    >
                      <span className="badge" style={{ color: "var(--accent)" }}>
                        {item.ticker}
                      </span>
                      <span className="badge">{item.source}</span>
                      <span className="text-muted" style={{ fontSize: "11px" }}>
                        {dt}
                      </span>
                    </div>
                    <div
                      style={{
                        fontWeight: 600,
                        color: "var(--link)",
                        fontSize: "14px",
                      }}
                    >
                      {item.title}
                    </div>
                    {item.summary && (
                      <div
                        className="text-muted"
                        style={{ fontSize: "12px", marginTop: "4px" }}
                      >
                        {item.summary}
                      </div>
                    )}
                  </div>
                  <span
                    className="text-muted"
                    style={{ fontSize: "12px", flexShrink: 0 }}
                  >
                    Abrir ↗
                  </span>
                </a>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};
