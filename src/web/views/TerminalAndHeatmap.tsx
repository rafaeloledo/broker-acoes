import React, { useState } from "react";
import { type OrderBook, type Tick } from "../../core/index.ts";
import { type AssetSnapshot, type HeatmapNode } from "../../shared/types.ts";
import { CandlestickChart, TreemapHeatmap } from "../components/Charts.tsx";

interface TerminalViewProps {
  visibleTickers: readonly string[];
  selectedTicker: string;
  assets: Record<string, AssetSnapshot>;
  orderBooks: Record<string, OrderBook>;
  recentTicks: Record<string, Tick[]>;
  isLiveSimRunning: boolean;
  darkTheme: boolean;
  onToggleLiveSim: (running: boolean) => void;
  onOpenSearchModal: (slot: number | null) => void;
  onFocusTicker: (ticker: string) => void;
  onFocusOnlyOne: (ticker: string) => void;
}

export const TerminalView: React.FC<TerminalViewProps> = ({
  visibleTickers,
  selectedTicker,
  assets,
  orderBooks,
  recentTicks,
  isLiveSimRunning,
  darkTheme,
  onToggleLiveSim,
  onOpenSearchModal,
  onFocusTicker,
  onFocusOnlyOne,
}) => {
  const [candleWidth, setCandleWidth] = useState(9);
  const [telemetry, setTelemetry] = useState({ fps: 60.0, frameMs: 1.4 });
  const [forceSingleMode, setForceSingleMode] = useState(false);

  const isSingleMode = visibleTickers.length === 1 || forceSingleMode;
  const focusedSym =
    visibleTickers.includes(selectedTicker)
      ? selectedTicker
      : visibleTickers[0] ?? "PETR4";
  const focusedAsset = assets[focusedSym];
  const book = orderBooks[focusedSym];
  const ticks = recentTicks[focusedSym] ?? [];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
      {/* Top Toolbar */}
      <div className="card">
        <div className="flex-between" style={{ gap: "12px" }}>
          <div className="flex-row" style={{ gap: "10px" }}>
            <strong style={{ fontSize: "15px" }}>
              Terminal OHLCV ({visibleTickers.length}{" "}
              {visibleTickers.length === 1 ? "gráfico" : "gráficos"})
            </strong>
            <span
              className="badge badge-positive"
              style={{ fontFamily: "monospace" }}
            >
              ⚡ {telemetry.fps.toFixed(0)} FPS ({telemetry.frameMs.toFixed(1)}{" "}
              ms)
            </span>
            {visibleTickers.length > 1 && (
              <button
                className="chip active"
                onClick={() => setForceSingleMode(!forceSingleMode)}
              >
                {forceSingleMode
                  ? `Ver Grade (${visibleTickers.length} gráficos)`
                  : `Focar ${focusedSym} + Book & Tape`}
              </button>
            )}
          </div>

          <div className="flex-row" style={{ gap: "14px" }}>
            <label
              className="flex-row"
              style={{ fontSize: "12px", color: "var(--subtle)" }}
            >
              <span>Largura do Candle: {candleWidth}px</span>
              <input
                type="range"
                min={3}
                max={20}
                step={1}
                value={candleWidth}
                onChange={(e) => setCandleWidth(Number(e.target.value))}
                style={{ width: "90px" }}
              />
            </label>

            <label
              className="flex-row"
              style={{
                cursor: "pointer",
                fontSize: "12px",
                fontWeight: 600,
              }}
            >
              <input
                type="checkbox"
                checked={isLiveSimRunning}
                onChange={(e) => onToggleLiveSim(e.target.checked)}
              />
              Simulação de ticks (book e tape)
            </label>

            {visibleTickers.length < 4 && (
              <button
                className="btn btn-primary"
                onClick={() => onOpenSearchModal(null)}
              >
                + Adicionar Gráfico
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Single-Asset Mode vs Multi-Chart Grid Mode */}
      {isSingleMode && focusedAsset ? (
        <div className="terminal-split">
          {/* Left 72%: Candlestick Chart */}
          <div className="card">
            <div className="flex-between" style={{ marginBottom: "8px" }}>
              <div className="flex-row">
                <strong style={{ fontSize: "18px" }}>
                  {focusedAsset.ticker}
                </strong>
                <span className="text-muted" style={{ fontSize: "13px" }}>
                  {focusedAsset.profile.name}
                </span>
              </div>
              <div style={{ fontFamily: "monospace", fontWeight: 800, fontSize: "18px" }}>
                R$ {focusedAsset.indicators.price.toFixed(2)}
              </div>
            </div>
            <CandlestickChart
              symbol={focusedAsset.ticker}
              candles={focusedAsset.candles}
              candleWidth={candleWidth}
              visibleCount={70}
              height={440}
              darkTheme={darkTheme}
              onTelemetry={(fps, frameMs) => setTelemetry({ fps, frameMs })}
            />
          </div>

          {/* Right 28%: Level 2 Order Book + Times & Trades */}
          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            <div className="card">
              <div className="flex-between" style={{ marginBottom: "8px" }}>
                <strong style={{ fontSize: "13px" }}>
                  Livro de Ofertas (simulação)
                </strong>
                <span className="badge">{focusedAsset.ticker}</span>
              </div>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "1fr 1fr",
                  gap: "8px",
                  fontFamily: "monospace",
                  fontSize: "11px",
                }}
              >
                <div>
                  <div
                    className="text-positive"
                    style={{ fontWeight: 700, marginBottom: "4px" }}
                  >
                    COMPRA (BID)
                  </div>
                  {(book?.bids ?? []).slice(0, 8).map((lvl, i) => (
                    <div
                      key={i}
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        padding: "2px 4px",
                        background: "var(--positive-bg)",
                        marginBottom: "2px",
                        borderRadius: "3px",
                      }}
                    >
                      <span>{lvl.quantity}</span>
                      <strong className="text-positive">
                        {lvl.price.toFixed(2)}
                      </strong>
                    </div>
                  ))}
                </div>
                <div>
                  <div
                    className="text-negative"
                    style={{ fontWeight: 700, marginBottom: "4px" }}
                  >
                    VENDA (ASK)
                  </div>
                  {(book?.asks ?? []).slice(0, 8).map((lvl, i) => (
                    <div
                      key={i}
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        padding: "2px 4px",
                        background: "var(--negative-bg)",
                        marginBottom: "2px",
                        borderRadius: "3px",
                      }}
                    >
                      <strong className="text-negative">
                        {lvl.price.toFixed(2)}
                      </strong>
                      <span>{lvl.quantity}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            <div className="card" style={{ flex: 1 }}>
              <div className="flex-between" style={{ marginBottom: "8px" }}>
                <strong style={{ fontSize: "13px" }}>
                  Times & Trades (simulação)
                </strong>
                <span className="text-muted" style={{ fontSize: "11px" }}>
                  Últimos {ticks.length} negócios
                </span>
              </div>
              <div
                style={{
                  maxHeight: "220px",
                  overflowY: "auto",
                  fontFamily: "monospace",
                  fontSize: "11px",
                }}
              >
                {ticks.length === 0 ? (
                  <div className="text-muted" style={{ padding: "8px 0" }}>
                    Aguardando negócios simulados...
                  </div>
                ) : (
                  ticks.map((t, idx) => {
                    const timeStr = new Date(t.timestampMs)
                      .toISOString()
                      .slice(11, 19);
                    return (
                      <div
                        key={idx}
                        style={{
                          display: "grid",
                          gridTemplateColumns: "62px 1fr 56px 56px",
                          padding: "3px 0",
                          borderBottom:
                            "1px solid rgba(140, 155, 175, 0.1)",
                        }}
                      >
                        <span className="text-muted">{timeStr}</span>
                        <strong
                          className={
                            t.isBuyerAggressor
                              ? "text-positive"
                              : "text-negative"
                          }
                        >
                          R$ {t.price.toFixed(2)}
                        </strong>
                        <span>{t.quantity}</span>
                        <span className="text-muted">
                          #{t.buyerBroker}×#{t.sellerBroker}
                        </span>
                      </div>
                    );
                  })
                )}
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="multi-chart-grid-2">
          {visibleTickers.map((sym, idx) => {
            const snap = assets[sym];
            if (!snap) return null;
            return (
              <div key={sym} className="card">
                <div className="flex-between" style={{ marginBottom: "6px" }}>
                  <div className="flex-row" style={{ gap: "8px" }}>
                    <strong style={{ fontSize: "16px" }}>
                      {idx + 1}: {sym}
                    </strong>
                    <span
                      style={{
                        fontFamily: "monospace",
                        fontWeight: 700,
                        color: "var(--accent)",
                      }}
                    >
                      R$ {snap.indicators.price.toFixed(2)}
                    </span>
                  </div>
                  <div className="flex-row" style={{ gap: "6px" }}>
                    <button
                      className="btn"
                      onClick={() => onOpenSearchModal(idx)}
                    >
                      Trocar
                    </button>
                    <button
                      className="btn"
                      onClick={() => {
                        onFocusTicker(sym);
                        onFocusOnlyOne(sym);
                      }}
                    >
                      Focar
                    </button>
                  </div>
                </div>
                <CandlestickChart
                  symbol={sym}
                  candles={snap.candles}
                  candleWidth={candleWidth}
                  visibleCount={60}
                  height={280}
                  darkTheme={darkTheme}
                  onTelemetry={
                    idx === 0
                      ? (fps, frameMs) => setTelemetry({ fps, frameMs })
                      : undefined
                  }
                />
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

interface HeatmapViewProps {
  nodes: readonly HeatmapNode[];
  onSelectSymbol: (symbol: string) => void;
}

export const HeatmapView: React.FC<HeatmapViewProps> = ({
  nodes,
  onSelectSymbol,
}) => {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "12px" }}>
          <div>
            <strong style={{ fontSize: "16px" }}>
              Mapa de Calor Setorial B3 (Treemap)
            </strong>
            <div className="text-muted" style={{ fontSize: "12px" }}>
              Gradiente de variação diária e pulso de negociação em tempo real (simulação). Clique em qualquer ativo para abrir a Decisão.
            </div>
          </div>
        </div>
        <TreemapHeatmap nodes={nodes} onSelectSymbol={onSelectSymbol} />
      </div>
    </div>
  );
};
