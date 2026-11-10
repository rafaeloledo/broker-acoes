import React, { useMemo, useRef, useState } from "react";
import {
  assetClassFromSymbol,
  defaultStockIndicators,
  normalize,
  type B3Asset,
  type StockIndicators,
} from "../../core/index.ts";
import {
  ScreenerEngine,
  trendSignalLabel,
  trendSignalRgb,
  type StockTrendAnalysis,
  type TrendSignal,
} from "../../analytics/index.ts";
import { type ActiveView, type AssetSnapshot } from "../../shared/types.ts";

interface TrendingViewProps {
  trending: readonly StockTrendAnalysis[];
  catalog: readonly B3Asset[];
  onSelectAssetAndNavigate: (ticker: string, view: ActiveView) => void;
  onAddToWatchlist: (ticker: string) => void;
}

type TrendSort =
  | "ScoreDesc"
  | "GrahamUpsideDesc"
  | "BazinUpsideDesc"
  | "DyDesc"
  | "RsiAsc"
  | "TickerAsc";

export const TrendingView: React.FC<TrendingViewProps> = ({
  trending,
  catalog,
  onSelectAssetAndNavigate,
  onAddToWatchlist,
}) => {
  const [searchQuery, setSearchQuery] = useState("");
  const [signalFilter, setSignalFilter] = useState<TrendSignal | "ALL">("ALL");
  const [onlyStocks, setOnlyStocks] = useState(true);
  const [sortBy, setSortBy] = useState<TrendSort>("ScoreDesc");
  const [selectedTicker, setSelectedTicker] = useState("PETR4");
  const detailRef = useRef<HTMLDivElement | null>(null);

  const catalogClassMap = useMemo(() => {
    const m = new Map<string, string>();
    for (const c of catalog) m.set(c.ticker, c.assetClass);
    return m;
  }, [catalog]);

  const baseList = useMemo(() => {
    return trending.filter((item) => {
      if (onlyStocks) {
        const cls =
          catalogClassMap.get(item.ticker) ?? assetClassFromSymbol(item.ticker);
        if (cls !== "Acao") return false;
      }
      return true;
    });
  }, [trending, onlyStocks, catalogClassMap]);

  const counts = useMemo(() => {
    let strongBuy = 0;
    let buy = 0;
    let neutral = 0;
    let sellCaution = 0;
    let totalScore = 0;
    for (const item of baseList) {
      totalScore += item.overallScore;
      if (item.signal === "StrongBuy") strongBuy++;
      else if (item.signal === "Buy") buy++;
      else if (item.signal === "Neutral") neutral++;
      else sellCaution++;
    }
    const avgScore = baseList.length > 0 ? totalScore / baseList.length : 0;
    return { strongBuy, buy, neutral, sellCaution, avgScore };
  }, [baseList]);

  const filteredList = useMemo(() => {
    const q = normalize(searchQuery.trim());
    const filtered = baseList.filter((item) => {
      if (signalFilter !== "ALL" && item.signal !== signalFilter) return false;
      if (q) {
        const matches =
          normalize(item.ticker).includes(q) ||
          normalize(item.name).includes(q) ||
          normalize(item.sector).includes(q);
        if (!matches) return false;
      }
      return true;
    });

    return filtered.slice().sort((a, b) => {
      switch (sortBy) {
        case "ScoreDesc":
          return b.overallScore - a.overallScore;
        case "GrahamUpsideDesc":
          return b.grahamUpsidePct - a.grahamUpsidePct;
        case "BazinUpsideDesc":
          return b.bazinUpsidePct - a.bazinUpsidePct;
        case "DyDesc":
          return b.dividendYield - a.dividendYield;
        case "RsiAsc":
          return a.rsi14 - b.rsi14;
        case "TickerAsc":
          return a.ticker.localeCompare(b.ticker);
      }
    });
  }, [baseList, signalFilter, searchQuery, sortBy]);

  const activeDetail =
    trending.find((t) => t.ticker === selectedTicker) ??
    filteredList[0] ??
    trending[0];

  const handleAnalyzeClick = (ticker: string) => {
    setSelectedTicker(ticker);
    if (window.innerWidth <= 960) {
      setTimeout(() => {
        detailRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
      }, 50);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
      {/* Top 5 Summary Metric Cards */}
      <div className="grid-5">
        <div className="card-alt">
          <div className="kpi-label">Forte Compra</div>
          <div className="kpi-value text-positive">{counts.strongBuy}</div>
          <div className="kpi-sub">Score ≥ 72</div>
        </div>
        <div className="card-alt">
          <div className="kpi-label">Compra</div>
          <div className="kpi-value text-positive">{counts.buy}</div>
          <div className="kpi-sub">Score 57–71</div>
        </div>
        <div className="card-alt">
          <div className="kpi-label">Neutro</div>
          <div className="kpi-value">{counts.neutral}</div>
          <div className="kpi-sub">Score 43–56</div>
        </div>
        <div className="card-alt">
          <div className="kpi-label">Venda / Cautela</div>
          <div className="kpi-value text-negative">{counts.sellCaution}</div>
          <div className="kpi-sub">Score &lt; 43</div>
        </div>
        <div className="card-alt">
          <div className="kpi-label">Score Médio B3</div>
          <div className="kpi-value text-accent">
            {counts.avgScore.toFixed(1)}
          </div>
          <div className="kpi-sub">{baseList.length} ativos avaliados</div>
        </div>
      </div>

      {/* Filter & Sort Toolbar */}
      <div className="card">
        <div className="flex-between" style={{ gap: "10px", marginBottom: "10px" }}>
          <div className="flex-row">
            {(
              [
                ["ALL", "Todos"],
                ["StrongBuy", "Forte Compra"],
                ["Buy", "Compra"],
                ["Neutral", "Neutro"],
                ["Sell", "Venda"],
                ["StrongSell", "Forte Venda"],
              ] as const
            ).map(([sig, label]) => (
              <button
                key={sig}
                className={`chip ${signalFilter === sig ? "active" : ""}`}
                onClick={() => setSignalFilter(sig)}
              >
                {label}
              </button>
            ))}
          </div>

          <label
            className="flex-row"
            style={{ cursor: "pointer", fontSize: "13px", fontWeight: 600 }}
          >
            <input
              type="checkbox"
              checked={onlyStocks}
              onChange={(e) => setOnlyStocks(e.target.checked)}
            />
            Somente Ações
          </label>
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr auto",
            gap: "10px",
          }}
        >
          <input
            type="text"
            className="input"
            placeholder="Buscar ativo por código, nome ou setor..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          <select
            className="input"
            style={{ width: "auto" }}
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as TrendSort)}
          >
            <option value="ScoreDesc">Maior Score</option>
            <option value="GrahamUpsideDesc">Potencial Graham</option>
            <option value="BazinUpsideDesc">Preço Teto Bazin</option>
            <option value="DyDesc">Maior DY</option>
            <option value="RsiAsc">IFR (Sobrevenda)</option>
            <option value="TickerAsc">Ticker A-Z</option>
          </select>
        </div>
      </div>

      {/* Split View: Left Master Table + Right Quantitative Breakdown */}
      <div className="split-layout">
        <div className="table-scroll" style={{ maxHeight: "680px" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th className="sticky-col">Ticker</th>
                <th>Preço</th>
                <th>Var 1d</th>
                <th>Sinal</th>
                <th>Score</th>
                <th>IFR</th>
                <th>Graham</th>
                <th>DY</th>
                <th>Ação</th>
              </tr>
            </thead>
            <tbody>
              {filteredList.map((item) => {
                const [r, g, b] = trendSignalRgb(item.signal);
                const sigColor = `rgb(${r}, ${g}, ${b})`;
                const isSelected = activeDetail?.ticker === item.ticker;
                return (
                  <tr
                    key={item.ticker}
                    onClick={() => handleAnalyzeClick(item.ticker)}
                    style={{
                      cursor: "pointer",
                      background: isSelected ? "var(--accent-alpha)" : undefined,
                    }}
                  >
                    <td className="sticky-col">
                      <div style={{ fontWeight: 800 }}>{item.ticker}</div>
                      <div
                        className="text-muted"
                        style={{
                          fontSize: "11px",
                          maxWidth: "120px",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        {item.name}
                      </div>
                    </td>
                    <td style={{ fontFamily: "monospace", fontWeight: 700 }}>
                      R$ {item.price.toFixed(2)}
                    </td>
                    <td
                      className={
                        item.change1dPct >= 0
                          ? "text-positive"
                          : "text-negative"
                      }
                      style={{ fontFamily: "monospace", fontWeight: 600 }}
                    >
                      {item.change1dPct >= 0 ? "+" : ""}
                      {item.change1dPct.toFixed(2)}%
                    </td>
                    <td>
                      <span
                        className="badge"
                        style={{
                          background: `rgba(${r}, ${g}, ${b}, 0.16)`,
                          color: sigColor,
                        }}
                      >
                        {trendSignalLabel(item.signal)}
                      </span>
                    </td>
                    <td style={{ minWidth: "90px" }}>
                      <div
                        style={{
                          fontSize: "12px",
                          fontWeight: 700,
                          marginBottom: "3px",
                        }}
                      >
                        {item.overallScore.toFixed(0)}/100
                      </div>
                      <div className="progress-track">
                        <div
                          className="progress-fill"
                          style={{
                            width: `${item.overallScore}%`,
                            background: sigColor,
                          }}
                        />
                      </div>
                    </td>
                    <td style={{ fontFamily: "monospace" }}>
                      {item.rsi14.toFixed(1)}
                    </td>
                    <td style={{ fontFamily: "monospace" }}>
                      R$ {item.grahamFairValue.toFixed(2)}
                    </td>
                    <td className="text-accent" style={{ fontWeight: 700 }}>
                      {item.dividendYield.toFixed(1)}%
                    </td>
                    <td>
                      <button
                        className="btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleAnalyzeClick(item.ticker);
                        }}
                      >
                        Analisar
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {/* Right Detail Panel */}
        {activeDetail && (
          <div
            ref={detailRef}
            className="card"
            style={{ display: "flex", flexDirection: "column", gap: "12px" }}
          >
            <div className="flex-between">
              <div>
                <div className="flex-row" style={{ gap: "8px" }}>
                  <strong style={{ fontSize: "22px" }}>
                    {activeDetail.ticker}
                  </strong>
                  <span className="badge">{activeDetail.sector}</span>
                </div>
                <div className="text-muted" style={{ fontSize: "13px" }}>
                  {activeDetail.name}
                </div>
              </div>
              <div style={{ textAlign: "right" }}>
                <div style={{ fontSize: "20px", fontWeight: 800 }}>
                  R$ {activeDetail.price.toFixed(2)}
                </div>
                <div
                  className={
                    activeDetail.change1dPct >= 0
                      ? "text-positive"
                      : "text-negative"
                  }
                  style={{ fontSize: "12px", fontWeight: 700 }}
                >
                  {activeDetail.change1dPct >= 0 ? "+" : ""}
                  {activeDetail.change1dPct.toFixed(2)}%
                </div>
              </div>
            </div>

            {/* Composite Score Gauge */}
            <div className="card-alt">
              <div className="flex-between" style={{ marginBottom: "6px" }}>
                <strong>
                  Score Composto: {activeDetail.overallScore.toFixed(0)}/100 (
                  {trendSignalLabel(activeDetail.signal)})
                </strong>
                <span className="text-muted" style={{ fontSize: "12px" }}>
                  Técnico {activeDetail.technicalScore.toFixed(0)} · Fundamento{" "}
                  {activeDetail.fundamentalScore.toFixed(0)}
                </span>
              </div>
              <div className="progress-track" style={{ height: "10px" }}>
                <div
                  className="progress-fill"
                  style={{
                    width: `${activeDetail.overallScore}%`,
                    background: `rgb(${trendSignalRgb(
                      activeDetail.signal
                    ).join(",")})`,
                  }}
                />
              </div>
              <div
                className="text-subtle"
                style={{ fontSize: "12px", marginTop: "8px" }}
              >
                {activeDetail.summaryRationale}
              </div>
            </div>

            {/* Card 1: Technical Breakdown */}
            <div className="card-inset">
              <strong
                style={{
                  display: "block",
                  marginBottom: "6px",
                  fontSize: "12px",
                  textTransform: "uppercase",
                  color: "var(--label)",
                }}
              >
                1. Indicadores Técnicos
              </strong>
              <div className="kv-row">
                <span className="text-muted">IFR (RSI 14)</span>
                <strong>
                  {activeDetail.rsi14.toFixed(1)} (
                  {activeDetail.rsi14 < 30
                    ? "Sobrevendido"
                    : activeDetail.rsi14 > 70
                    ? "Sobrecomprado"
                    : "Neutro"}
                  )
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">SMA 20</span>
                <strong>
                  R$ {activeDetail.sma20.toFixed(2)} (
                  {activeDetail.priceVsSma20Pct >= 0 ? "+" : ""}
                  {activeDetail.priceVsSma20Pct.toFixed(1)}%)
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">SMA 50</span>
                <strong>
                  R$ {activeDetail.sma50.toFixed(2)} (
                  {activeDetail.priceVsSma50Pct >= 0 ? "+" : ""}
                  {activeDetail.priceVsSma50Pct.toFixed(1)}%)
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">Tendência de Médias</span>
                <strong
                  className={
                    activeDetail.sma20 >= activeDetail.sma50
                      ? "text-positive"
                      : "text-negative"
                  }
                >
                  {activeDetail.sma20 >= activeDetail.sma50
                    ? "Alta (SMA20 ≥ SMA50)"
                    : "Baixa (SMA20 < SMA50)"}
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">Bandas de Bollinger (20, 2σ)</span>
                <strong>
                  Inf {activeDetail.bollingerLower.toFixed(2)} · Sup{" "}
                  {activeDetail.bollingerUpper.toFixed(2)} (%B{" "}
                  {(activeDetail.bollingerPctB * 100).toFixed(0)}%)
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">Momentum (20 pregões)</span>
                <strong
                  className={
                    activeDetail.momentum20dPct >= 0
                      ? "text-positive"
                      : "text-negative"
                  }
                >
                  {activeDetail.momentum20dPct >= 0 ? "+" : ""}
                  {activeDetail.momentum20dPct.toFixed(2)}%
                </strong>
              </div>
            </div>

            {/* Card 2: Fundamental & Valuation */}
            <div className="card-inset">
              <strong
                style={{
                  display: "block",
                  marginBottom: "6px",
                  fontSize: "12px",
                  textTransform: "uppercase",
                  color: "var(--label)",
                }}
              >
                2. Fundamentos & Valuation
              </strong>
              <div className="kv-row">
                <span className="text-muted">Preço Justo Graham</span>
                <strong
                  className={
                    activeDetail.grahamUpsidePct >= 0
                      ? "text-positive"
                      : "text-negative"
                  }
                >
                  R$ {activeDetail.grahamFairValue.toFixed(2)} (
                  {activeDetail.grahamUpsidePct >= 0 ? "+" : ""}
                  {activeDetail.grahamUpsidePct.toFixed(1)}%)
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">Preço Teto Bazin (6%)</span>
                <strong
                  className={
                    activeDetail.bazinUpsidePct >= 0
                      ? "text-positive"
                      : "text-negative"
                  }
                >
                  R$ {activeDetail.bazinCeilingPrice.toFixed(2)} (
                  {activeDetail.bazinUpsidePct >= 0 ? "+" : ""}
                  {activeDetail.bazinUpsidePct.toFixed(1)}%)
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">P/L & P/VP</span>
                <strong>
                  {activeDetail.pl.toFixed(2)}x · {activeDetail.pvp.toFixed(2)}x
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">Dividend Yield</span>
                <strong className="text-accent">
                  {activeDetail.dividendYield.toFixed(2)}%
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">ROE & Margem Líquida</span>
                <strong>
                  {activeDetail.roe.toFixed(1)}% ·{" "}
                  {activeDetail.margemLiquida.toFixed(1)}%
                </strong>
              </div>
              <div className="kv-row">
                <span className="text-muted">Piotroski F-Score & Altman Z</span>
                <strong>
                  {activeDetail.piotroskiFScore}/9 · Z{" "}
                  {activeDetail.altmanZScore.toFixed(2)}
                </strong>
              </div>
            </div>

            {/* Card 3: Catalysts & Risk Factors */}
            <div className="card-inset">
              <strong
                style={{
                  display: "block",
                  marginBottom: "6px",
                  fontSize: "12px",
                  textTransform: "uppercase",
                  color: "var(--label)",
                }}
              >
                3. Catalisadores & Fatores de Risco
              </strong>
              {activeDetail.buyCatalysts.map((c, idx) => (
                <div
                  key={idx}
                  style={{
                    fontSize: "12px",
                    color: "var(--positive)",
                    marginBottom: "4px",
                  }}
                >
                  ✔ {c}
                </div>
              ))}
              {activeDetail.riskFactors.map((r, idx) => (
                <div
                  key={idx}
                  style={{
                    fontSize: "12px",
                    color: "var(--warning)",
                    marginBottom: "4px",
                  }}
                >
                  ⚠ {r}
                </div>
              ))}
            </div>

            {/* Action Buttons */}
            <div className="flex-row">
              <button
                className="btn btn-primary"
                onClick={() =>
                  onSelectAssetAndNavigate(activeDetail.ticker, "Decision")
                }
              >
                Ver Decisão
              </button>
              <button
                className="btn"
                onClick={() =>
                  onSelectAssetAndNavigate(activeDetail.ticker, "GpuTerminal")
                }
              >
                Abrir no Terminal
              </button>
              <button
                className="btn"
                onClick={() =>
                  onSelectAssetAndNavigate(activeDetail.ticker, "StatusInvest")
                }
              >
                Ver na Ficha
              </button>
              <button
                className="btn"
                onClick={() => onAddToWatchlist(activeDetail.ticker)}
              >
                + Adicionar à Tela
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

interface ScreenerViewProps {
  assets: Record<string, AssetSnapshot>;
  catalog: readonly B3Asset[];
  onSelectAssetAndNavigate: (ticker: string, view: ActiveView) => void;
}

export const ScreenerView: React.FC<ScreenerViewProps> = ({
  assets,
  catalog,
  onSelectAssetAndNavigate,
}) => {
  const [mode, setMode] = useState<"custom" | "magic">("custom");
  const [minDy, setMinDy] = useState(0);
  const [maxPl, setMaxPl] = useState(50);
  const [maxPvp, setMaxPvp] = useState(10);
  const [minRoe, setMinRoe] = useState(0);

  const allIndicators = useMemo(() => {
    const list: StockIndicators[] = [];
    for (const item of catalog) {
      if (item.assetClass !== "Acao" && item.assetClass !== "Fii") continue;
      const snap = assets[item.ticker];
      list.push(snap?.indicators ?? defaultStockIndicators(item.ticker));
    }
    return list;
  }, [assets, catalog]);

  const applyBazin = () => {
    setMode("custom");
    const p = ScreenerEngine.bazinPreset();
    setMinDy(p.minDy ?? 6);
    setMaxPl(p.maxPl ?? 15);
    setMaxPvp(10);
    setMinRoe(p.minRoe ?? 10);
  };

  const applyGraham = () => {
    setMode("custom");
    const p = ScreenerEngine.grahamPreset();
    setMinDy(0);
    setMaxPl(p.maxPl ?? 15);
    setMaxPvp(p.maxPvp ?? 1.5);
    setMinRoe(p.minRoe ?? 8);
  };

  const clearFilters = () => {
    setMode("custom");
    setMinDy(0);
    setMaxPl(50);
    setMaxPvp(10);
    setMinRoe(0);
  };

  const results = useMemo(() => {
    if (mode === "magic") {
      return ScreenerEngine.magicFormula(allIndicators);
    }
    return ScreenerEngine.filter(allIndicators, {
      minDy: minDy > 0 ? minDy : null,
      maxPl: maxPl < 50 ? maxPl : null,
      maxPvp: maxPvp < 10 ? maxPvp : null,
      minRoe: minRoe > 0 ? minRoe : null,
    });
  }, [allIndicators, mode, minDy, maxPl, maxPvp, minRoe]);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "12px" }}>
          <div>
            <strong style={{ fontSize: "16px" }}>
              Screener Fundamentalista B3
            </strong>
            <div className="text-muted" style={{ fontSize: "12px" }}>
              Filtros multi-fator e estratégias clássicas (Greenblatt, Décio Bazin, Benjamin Graham)
            </div>
          </div>
          <div className="flex-row">
            <button
              className={`chip ${mode === "magic" ? "active" : ""}`}
              onClick={() => setMode("magic")}
            >
              ✨ Fórmula Mágica (Greenblatt)
            </button>
            <button className="chip" onClick={applyBazin}>
              💰 Método Bazin (Preço Teto)
            </button>
            <button className="chip" onClick={applyGraham}>
              📐 Graham Deep Value
            </button>
            <button className="btn" onClick={clearFilters}>
              Limpar Filtros
            </button>
          </div>
        </div>

        <div className="grid-4">
          <div className="card-inset">
            <div className="flex-between">
              <span className="kpi-label">Min Dividend Yield</span>
              <strong>{minDy.toFixed(1)}%</strong>
            </div>
            <input
              type="range"
              min={0}
              max={20}
              step={0.5}
              value={minDy}
              onChange={(e) => {
                setMode("custom");
                setMinDy(Number(e.target.value));
              }}
              style={{ width: "100%" }}
            />
          </div>

          <div className="card-inset">
            <div className="flex-between">
              <span className="kpi-label">Max P/L</span>
              <strong>{maxPl >= 50 ? "Sem limite" : `${maxPl.toFixed(1)}x`}</strong>
            </div>
            <input
              type="range"
              min={2}
              max={50}
              step={1}
              value={maxPl}
              onChange={(e) => {
                setMode("custom");
                setMaxPl(Number(e.target.value));
              }}
              style={{ width: "100%" }}
            />
          </div>

          <div className="card-inset">
            <div className="flex-between">
              <span className="kpi-label">Max P/VP</span>
              <strong>
                {maxPvp >= 10 ? "Sem limite" : `${maxPvp.toFixed(1)}x`}
              </strong>
            </div>
            <input
              type="range"
              min={0.5}
              max={10}
              step={0.25}
              value={maxPvp}
              onChange={(e) => {
                setMode("custom");
                setMaxPvp(Number(e.target.value));
              }}
              style={{ width: "100%" }}
            />
          </div>

          <div className="card-inset">
            <div className="flex-between">
              <span className="kpi-label">Min ROE</span>
              <strong>{minRoe.toFixed(1)}%</strong>
            </div>
            <input
              type="range"
              min={0}
              max={40}
              step={1}
              value={minRoe}
              onChange={(e) => {
                setMode("custom");
                setMinRoe(Number(e.target.value));
              }}
              style={{ width: "100%" }}
            />
          </div>
        </div>
      </div>

      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr>
              <th className="sticky-col">Ativo</th>
              <th>Preço</th>
              <th>DY (%)</th>
              <th>P/L</th>
              <th>P/VP</th>
              <th>EV/EBIT</th>
              <th>ROE (%)</th>
              <th>ROIC (%)</th>
              <th>Graham Justo</th>
              <th>Bazin Teto</th>
              <th>F-Score</th>
              <th>Ação</th>
            </tr>
          </thead>
          <tbody>
            {results.map((item) => (
              <tr key={item.symbol}>
                <td className="sticky-col" style={{ fontWeight: 800 }}>
                  {item.symbol}
                </td>
                <td style={{ fontFamily: "monospace", fontWeight: 700 }}>
                  R$ {item.price.toFixed(2)}
                </td>
                <td className="text-accent" style={{ fontWeight: 700 }}>
                  {item.dividendYield.toFixed(2)}%
                </td>
                <td>{item.pl.toFixed(2)}</td>
                <td>{item.pvp.toFixed(2)}</td>
                <td>{item.evEbit.toFixed(2)}</td>
                <td>{item.roe.toFixed(1)}%</td>
                <td>{item.roic.toFixed(1)}%</td>
                <td>R$ {item.grahamFairValue.toFixed(2)}</td>
                <td>R$ {item.bazinCeilingPrice.toFixed(2)}</td>
                <td>{item.piotroskiFScore}/9</td>
                <td>
                  <div className="flex-row" style={{ flexWrap: "nowrap" }}>
                    <button
                      className="btn btn-primary"
                      onClick={() =>
                        onSelectAssetAndNavigate(item.symbol, "Decision")
                      }
                    >
                      Ver decisão
                    </button>
                    <button
                      className="btn"
                      onClick={() =>
                        onSelectAssetAndNavigate(item.symbol, "StatusInvest")
                      }
                    >
                      Ficha
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
