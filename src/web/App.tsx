import React, { useCallback, useEffect, useMemo, useState } from "react";
import { type TradeOperation } from "../core/index.ts";
import {
  type ActiveView,
  type InitialStatePayload,
  type WsServerMessage,
} from "../shared/types.ts";
import { SearchModal } from "./components/SearchModal.tsx";
import { DecisionView, NewsView } from "./views/DecisionAndNews.tsx";
import {
  InputAnalysisView,
  parseInputTickers,
} from "./views/InputAnalysisView.tsx";
import { PortfolioView, FichaView } from "./views/PortfolioAndFicha.tsx";
import { TerminalView, HeatmapView } from "./views/TerminalAndHeatmap.tsx";
import { TrendingView, ScreenerView } from "./views/TrendingAndScreener.tsx";

const NAV_ITEMS: ReadonlyArray<{
  view: ActiveView;
  label: string;
  icon: string;
}> = [
  { view: "Decision", label: "Decisão", icon: "🎯" },
  { view: "InputAnalysis", label: "Análise por Input", icon: "📋" },
  { view: "Trending", label: "Trending", icon: "🔥" },
  { view: "News", label: "Notícias", icon: "📰" },
  { view: "GpuTerminal", label: "Terminal", icon: "📈" },
  { view: "MarketHeatmap", label: "Mapa", icon: "🗺️" },
  { view: "Screener", label: "Busca", icon: "🔎" },
  { view: "PortfolioTax", label: "Carteira", icon: "💼" },
  { view: "StatusInvest", label: "Ficha", icon: "📊" },
];

const WIDE_VIEWS = new Set<ActiveView>([
  "GpuTerminal",
  "MarketHeatmap",
  "Trending",
  "InputAnalysis",
]);

export const App: React.FC = () => {
  const [state, setState] = useState<InitialStatePayload | null>(null);
  const [activeView, setActiveView] = useState<ActiveView>("Decision");
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [darkTheme, setDarkTheme] = useState<boolean>(() => {
    const saved = localStorage.getItem("dark_theme");
    return saved == null ? true : saved === "true";
  });
  const [inputText, setInputText] = useState<string>(() => {
    return (
      localStorage.getItem("input_analysis_text") ||
      "PETR4, VALE3, WEGE3, ITUB4, BBAS3, HGLG11, KNCR11, XPML11"
    );
  });
  const [searchModalOpen, setSearchModalOpen] = useState(false);
  const [searchTargetSlot, setSearchTargetSlot] = useState<number | null>(null);

  useEffect(() => {
    document.documentElement.setAttribute(
      "data-theme",
      darkTheme ? "dark" : "light"
    );
    localStorage.setItem("dark_theme", String(darkTheme));
  }, [darkTheme]);

  const handleInputTextChange = useCallback((txt: string) => {
    setInputText(txt);
    localStorage.setItem("input_analysis_text", txt);
  }, []);

  // Load initial state & connect WebSocket
  useEffect(() => {
    let ws: WebSocket | null = null;
    let alive = true;

    fetch("/api/state")
      .then((r) => r.json())
      .then((data: InitialStatePayload) => {
        if (alive) setState(data);
      })
      .catch(console.error);

    const connectWs = () => {
      if (!alive) return;
      const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
      ws = new WebSocket(`${proto}//${window.location.host}/ws`);
      ws.onmessage = (ev) => {
        try {
          const msg = JSON.parse(ev.data) as WsServerMessage;
          setState((prev) => {
            if (!prev) return prev;
            if (msg.type === "tick_batch") {
              const nextAssets = { ...prev.assets };
              for (const [sym, pInfo] of Object.entries(msg.prices)) {
                const existing = nextAssets[sym];
                if (existing) {
                  const candles = existing.candles.slice();
                  if (candles.length > 0) {
                    candles[candles.length - 1] = pInfo.lastCandle;
                  }
                  nextAssets[sym] = {
                    ...existing,
                    indicators: {
                      ...existing.indicators,
                      price: pInfo.price,
                    },
                    candles,
                  };
                }
              }

              const nextTicks = { ...prev.recentTicks };
              for (const [sym, tick] of Object.entries(msg.ticks)) {
                const list = [tick, ...(nextTicks[sym] ?? [])].slice(0, 25);
                nextTicks[sym] = list;
              }

              return {
                ...prev,
                assets: nextAssets,
                orderBooks: { ...prev.orderBooks, ...msg.orderBooks },
                recentTicks: nextTicks,
                heatmapNodes: msg.heatmapNodes,
              };
            }
            if (msg.type === "asset_updated") {
              return {
                ...prev,
                assets: {
                  ...prev.assets,
                  [msg.asset.ticker]: msg.asset,
                },
              };
            }
            if (msg.type === "watchlist_updated") {
              return {
                ...prev,
                visibleTickers: msg.visibleTickers,
                selectedTicker: msg.visibleTickers.includes(prev.selectedTicker)
                  ? prev.selectedTicker
                  : msg.visibleTickers[0] ?? prev.selectedTicker,
                assets: { ...prev.assets, ...msg.assets },
              };
            }
            if (msg.type === "portfolio_updated") {
              return {
                ...prev,
                portfolio: msg.portfolio,
              };
            }
            if (msg.type === "news_updated") {
              return {
                ...prev,
                recentNews: msg.recentNews,
              };
            }
            return prev;
          });
        } catch {
          // Ignore invalid WS messages
        }
      };
      ws.onclose = () => {
        if (alive) setTimeout(connectWs, 2000);
      };
    };

    connectWs();
    return () => {
      alive = false;
      ws?.close();
    };
  }, []);

  const updateWatchlistServer = async (tickers: string[]) => {
    const res = await fetch("/api/watchlist", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tickers }),
    });
    if (res.ok) {
      const data = await res.json();
      setState((prev) =>
        prev
          ? {
              ...prev,
              visibleTickers: data.visibleTickers,
              assets: { ...prev.assets, ...data.assets },
            }
          : prev
      );
    }
  };

  const handleSelectTickerFromModal = async (
    ticker: string,
    slot: number | null
  ) => {
    if (!state) return;
    const current = state.visibleTickers.slice();
    if (slot != null && slot >= 0 && slot < current.length) {
      current[slot] = ticker;
    } else if (!current.includes(ticker)) {
      if (current.length < 4) current.push(ticker);
      else current[current.length - 1] = ticker;
    }
    setState((prev) => (prev ? { ...prev, selectedTicker: ticker } : prev));
    await updateWatchlistServer(current);
  };

  const handleRemoveWatchlistTicker = async (ticker: string) => {
    if (!state || state.visibleTickers.length <= 1) return;
    const next = state.visibleTickers.filter((t) => t !== ticker);
    await updateWatchlistServer(next);
  };

  const handleFocusOnlyOne = async (ticker: string) => {
    setState((prev) => (prev ? { ...prev, selectedTicker: ticker } : prev));
    await updateWatchlistServer([ticker]);
  };

  const handleSelectAssetAndNavigate = async (
    ticker: string,
    view: ActiveView
  ) => {
    if (!state) return;
    if (!state.visibleTickers.includes(ticker)) {
      const next =
        state.visibleTickers.length < 4
          ? [...state.visibleTickers, ticker]
          : [ticker, ...state.visibleTickers.slice(0, 3)];
      await updateWatchlistServer(next);
    }
    setState((prev) => (prev ? { ...prev, selectedTicker: ticker } : prev));
    setActiveView(view);
  };

  const handleEnsureTickers = useCallback(
    async (tickers: string[], forceStatusInvest: boolean) => {
      const res = await fetch("/api/assets/batch", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tickers, forceStatusInvest }),
      });
      if (res.ok) {
        const data = await res.json();
        setState((prev) =>
          prev
            ? {
                ...prev,
                assets: { ...prev.assets, ...data.assets },
              }
            : prev
        );
      }
    },
    []
  );

  const handleToggleLiveSim = async (running: boolean) => {
    setState((prev) => (prev ? { ...prev, isLiveSimRunning: running } : prev));
    await fetch("/api/simulation", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ running }),
    });
  };

  const handleSyncNews = async () => {
    const res = await fetch("/api/news/sync", { method: "POST" });
    if (res.ok) {
      const data = await res.json();
      setState((prev) =>
        prev ? { ...prev, recentNews: data.recentNews } : prev
      );
    }
  };

  const handleImportSinacor = async (noteText: string, brokerName: string) => {
    const res = await fetch("/api/portfolio/sinacor", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ noteText, brokerName }),
    });
    if (res.ok) {
      const data = await res.json();
      setState((prev) =>
        prev ? { ...prev, portfolio: data.portfolio } : prev
      );
    }
  };

  const handleAddManualTrade = async (trade: Partial<TradeOperation>) => {
    const res = await fetch("/api/portfolio/trade", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(trade),
    });
    if (res.ok) {
      const data = await res.json();
      setState((prev) =>
        prev ? { ...prev, portfolio: data.portfolio } : prev
      );
    }
  };

  const handleDeleteTrade = async (id: string) => {
    const res = await fetch(`/api/portfolio/trade/${encodeURIComponent(id)}`, {
      method: "DELETE",
    });
    if (res.ok) {
      const data = await res.json();
      setState((prev) =>
        prev ? { ...prev, portfolio: data.portfolio } : prev
      );
    }
  };

  const navCounts = useMemo(() => {
    if (!state) return {} as Partial<Record<ActiveView, number>>;
    const inputCount = parseInputTickers(inputText).length;
    const trendingBuys = state.trending.filter(
      (t) => t.signal === "StrongBuy" || t.signal === "Buy"
    ).length;
    return {
      InputAnalysis: inputCount,
      Trending: trendingBuys,
      News: state.recentNews.length,
      PortfolioTax: state.portfolio.trades.length,
    };
  }, [state, inputText]);

  if (!state) {
    return (
      <div
        style={{
          minHeight: "100dvh",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: "10px",
          fontWeight: 600,
        }}
      >
        <span className="brand-avatar">BA</span>
        <span>Carregando Broker Ações...</span>
      </div>
    );
  }

  const focusedTicker = state.visibleTickers.includes(state.selectedTicker)
    ? state.selectedTicker
    : state.visibleTickers[0] ?? "PETR4";
  const focusedAsset =
    state.assets[focusedTicker] ?? Object.values(state.assets)[0]!;

  return (
    <div className="app-shell">
      {/* Top GitHub-inspired Navigation Bar */}
      <header className="top-nav">
        <div className="nav-brand">
          <button
            className="icon-btn"
            onClick={() => setDrawerOpen(!drawerOpen)}
            title="Menu de Navegação"
          >
            ☰
          </button>
          <span className="brand-avatar">BA</span>
          <span className="brand-title">Broker Ações</span>
        </div>

        <nav className="nav-tabs-scroll">
          {NAV_ITEMS.map(({ view, label }) => {
            const count = navCounts[view];
            return (
              <button
                key={view}
                className={`nav-tab ${activeView === view ? "active" : ""}`}
                onClick={() => setActiveView(view)}
              >
                <span>{label}</span>
                {count != null && count > 0 && (
                  <span className="nav-badge">{count}</span>
                )}
              </button>
            );
          })}
        </nav>

        <div className="nav-actions">
          <button
            className={`theme-chip ${!darkTheme ? "active" : ""}`}
            onClick={() => setDarkTheme(false)}
          >
            Claro
          </button>
          <button
            className={`theme-chip ${darkTheme ? "active" : ""}`}
            onClick={() => setDarkTheme(true)}
          >
            Escuro
          </button>
        </div>
      </header>

      {/* Watchlist Bar (up to 4 slots) */}
      <div className="watchlist-bar">
        <span className="watchlist-label">Watchlist:</span>
        {state.visibleTickers.map((sym, idx) => {
          const snap = state.assets[sym];
          const price = snap?.marketPrice ?? snap?.indicators.price ?? 0;
          const isFocused = sym === focusedTicker;
          return (
            <div
              key={sym}
              className={`watchlist-slot ${isFocused ? "focused" : ""}`}
            >
              <button
                className="watchlist-ticker-btn"
                onClick={() =>
                  setState((prev) =>
                    prev ? { ...prev, selectedTicker: sym } : prev
                  )
                }
              >
                <span>{sym}</span>
                {price > 0 && (
                  <span
                    className="text-muted"
                    style={{ fontFamily: "monospace", fontSize: "12px" }}
                  >
                    R$ {price.toFixed(2)}
                  </span>
                )}
              </button>
              <button
                className="watchlist-mini-btn"
                onClick={() => {
                  setSearchTargetSlot(idx);
                  setSearchModalOpen(true);
                }}
              >
                trocar
              </button>
              {state.visibleTickers.length > 1 && (
                <button
                  className="watchlist-mini-btn danger"
                  onClick={() => handleRemoveWatchlistTicker(sym)}
                >
                  ×
                </button>
              )}
            </div>
          );
        })}

        {state.visibleTickers.length < 4 && (
          <button
            className="watchlist-add-btn"
            onClick={() => {
              setSearchTargetSlot(null);
              setSearchModalOpen(true);
            }}
          >
            + Adicionar
          </button>
        )}
      </div>

      {/* Main Content Area + Collapsible Left Drawer */}
      <div className="app-body">
        {drawerOpen && (
          <>
            <div
              className="drawer-backdrop"
              onClick={() => setDrawerOpen(false)}
            />
            <aside className="side-drawer">
              {NAV_ITEMS.map(({ view, label, icon }) => {
                const count = navCounts[view];
                return (
                  <button
                    key={view}
                    className={`drawer-item ${
                      activeView === view ? "active" : ""
                    }`}
                    onClick={() => {
                      setActiveView(view);
                      if (window.innerWidth <= 900) setDrawerOpen(false);
                    }}
                  >
                    <span>
                      {icon} {label}
                    </span>
                    {count != null && count > 0 && (
                      <span className="nav-badge">{count}</span>
                    )}
                  </button>
                );
              })}
            </aside>
          </>
        )}

        <main
          className={`page-container ${
            WIDE_VIEWS.has(activeView) ? "wide" : ""
          }`}
        >
          {activeView === "Decision" && focusedAsset && (
            <DecisionView
              asset={focusedAsset}
              news={state.recentNews}
              onNavigate={setActiveView}
              onOpenFicha={(sym) =>
                handleSelectAssetAndNavigate(sym, "StatusInvest")
              }
              onOpenTerminal={(sym) =>
                handleSelectAssetAndNavigate(sym, "GpuTerminal")
              }
            />
          )}

          {activeView === "InputAnalysis" && (
            <InputAnalysisView
              inputText={inputText}
              onChangeInputText={handleInputTextChange}
              assets={state.assets}
              onEnsureTickers={handleEnsureTickers}
              onSelectAssetAndNavigate={handleSelectAssetAndNavigate}
              onAddToWatchlist={(sym) => handleSelectTickerFromModal(sym, null)}
            />
          )}

          {activeView === "Trending" && (
            <TrendingView
              trending={state.trending}
              catalog={state.catalog}
              onSelectAssetAndNavigate={handleSelectAssetAndNavigate}
              onAddToWatchlist={(sym) => handleSelectTickerFromModal(sym, null)}
            />
          )}

          {activeView === "News" && (
            <NewsView
              visibleTickers={state.visibleTickers}
              selectedTicker={focusedTicker}
              news={state.recentNews}
              onSyncNews={handleSyncNews}
            />
          )}

          {activeView === "GpuTerminal" && (
            <TerminalView
              visibleTickers={state.visibleTickers}
              selectedTicker={focusedTicker}
              assets={state.assets}
              orderBooks={state.orderBooks}
              recentTicks={state.recentTicks}
              isLiveSimRunning={state.isLiveSimRunning}
              darkTheme={darkTheme}
              onToggleLiveSim={handleToggleLiveSim}
              onOpenSearchModal={(slot) => {
                setSearchTargetSlot(slot);
                setSearchModalOpen(true);
              }}
              onFocusTicker={(sym) =>
                setState((prev) =>
                  prev ? { ...prev, selectedTicker: sym } : prev
                )
              }
              onFocusOnlyOne={handleFocusOnlyOne}
            />
          )}

          {activeView === "MarketHeatmap" && (
            <HeatmapView
              nodes={state.heatmapNodes}
              onSelectSymbol={(sym) =>
                handleSelectAssetAndNavigate(sym, "Decision")
              }
            />
          )}

          {activeView === "Screener" && (
            <ScreenerView
              assets={state.assets}
              catalog={state.catalog}
              onSelectAssetAndNavigate={handleSelectAssetAndNavigate}
            />
          )}

          {activeView === "PortfolioTax" && (
            <PortfolioView
              portfolio={state.portfolio}
              onImportSinacor={handleImportSinacor}
              onAddManualTrade={handleAddManualTrade}
              onDeleteTrade={handleDeleteTrade}
            />
          )}

          {activeView === "StatusInvest" && (
            <FichaView
              visibleTickers={state.visibleTickers}
              selectedTicker={focusedTicker}
              assets={state.assets}
              onFocusOnlyOne={handleFocusOnlyOne}
              onOpenSearchModal={(slot) => {
                setSearchTargetSlot(slot);
                setSearchModalOpen(true);
              }}
              onRemoveFromWatchlist={handleRemoveWatchlistTicker}
            />
          )}
        </main>
      </div>

      {/* Smartphone & Tablet Bottom Navigation Bar */}
      <nav className="mobile-bottom-nav">
        <button
          className={`mobile-nav-btn ${
            activeView === "Decision" ? "active" : ""
          }`}
          onClick={() => setActiveView("Decision")}
        >
          <span className="mobile-nav-icon">🎯</span>
          <span>Decisão</span>
        </button>
        <button
          className={`mobile-nav-btn ${
            activeView === "InputAnalysis" ? "active" : ""
          }`}
          onClick={() => setActiveView("InputAnalysis")}
        >
          <span className="mobile-nav-icon">📋</span>
          <span>Input</span>
        </button>
        <button
          className={`mobile-nav-btn ${
            activeView === "Trending" ? "active" : ""
          }`}
          onClick={() => setActiveView("Trending")}
        >
          <span className="mobile-nav-icon">🔥</span>
          <span>Trending</span>
        </button>
        <button
          className={`mobile-nav-btn ${
            activeView === "GpuTerminal" ? "active" : ""
          }`}
          onClick={() => setActiveView("GpuTerminal")}
        >
          <span className="mobile-nav-icon">📈</span>
          <span>Terminal</span>
        </button>
        <button
          className={`mobile-nav-btn ${
            activeView === "PortfolioTax" ? "active" : ""
          }`}
          onClick={() => setActiveView("PortfolioTax")}
        >
          <span className="mobile-nav-icon">💼</span>
          <span>Carteira</span>
        </button>
        <button
          className={`mobile-nav-btn ${drawerOpen ? "active" : ""}`}
          onClick={() => setDrawerOpen(!drawerOpen)}
        >
          <span className="mobile-nav-icon">☰</span>
          <span>Mais</span>
        </button>
      </nav>

      {/* B3 Search Modal */}
      <SearchModal
        isOpen={searchModalOpen}
        targetSlot={searchTargetSlot}
        visibleTickers={state.visibleTickers}
        onClose={() => setSearchModalOpen(false)}
        onSelectTicker={handleSelectTickerFromModal}
      />
    </div>
  );
};
