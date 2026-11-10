import React, { useEffect, useMemo, useState } from "react";
import {
  fiiSegmentDisplayName,
  findB3Asset,
  normalize,
} from "../../core/index.ts";
import {
  TrendingEngine,
  trendSignalLabel,
} from "../../analytics/index.ts";
import { type ActiveView, type AssetSnapshot } from "../../shared/types.ts";
import { IndicatorLabel } from "../components/IndicatorTooltip.tsx";

export function parseInputTickers(raw: string): string[] {
  const tokens = raw
    .toUpperCase()
    .replace(/[\[\]"'()]/g, " ")
    .split(/[,;\n\r\t\s]+/)
    .map((t) => t.trim())
    .filter((t) => /^[A-Z]{4}\d{1,2}$/.test(t));

  const out: string[] = [];
  const seen = new Set<string>();
  for (const tok of tokens) {
    const found = findB3Asset(tok);
    const canonical = found ? found.ticker : tok;
    if (!seen.has(canonical)) {
      seen.add(canonical);
      out.push(canonical);
    }
  }
  return out;
}

const PRESETS: Record<string, string> = {
  "Ações & FIIs": "PETR4, VALE3, WEGE3, ITUB4, BBAS3, HGLG11, KNCR11, XPML11",
  "Top Dividendos": "PETR4, BBAS3, TAEE11, EGIE3, BBSE3, ITSA4, CMIG4, CPLE6",
  "Fundos Imobiliários":
    "HGLG11, KNCR11, MXRF11, XPML11, BTLG11, VISC11, HGRU11, BRCO11",
  "Tech, ETFs & BDRs":
    "TOTS3, WEGE3, INTB3, BOVA11, IVVB11, NVDC34, AAPL34, MSFT34",
  "Small Caps & Energia":
    "PRIO3, AZZA3, CURY3, DIRR3, EQTL3, ENGI11, SAPR11, ALUP11",
};

type ClassFilter = "ALL" | "ACAO" | "FII" | "ETF_BDR";
type ViewMode = "Cards" | "Table";
type SortMode =
  | "InputOrder"
  | "TickerAsc"
  | "PriceDesc"
  | "DyDesc"
  | "ChangeDesc"
  | "GrahamUpsideDesc";

interface InputAnalysisViewProps {
  inputText: string;
  onChangeInputText: (text: string) => void;
  assets: Record<string, AssetSnapshot>;
  onEnsureTickers: (tickers: string[], forceStatusInvest: boolean) => void;
  onSelectAssetAndNavigate: (ticker: string, view: ActiveView) => void;
  onAddToWatchlist: (ticker: string) => void;
}

export const InputAnalysisView: React.FC<InputAnalysisViewProps> = ({
  inputText,
  onChangeInputText,
  assets,
  onEnsureTickers,
  onSelectAssetAndNavigate,
  onAddToWatchlist,
}) => {
  const [classFilter, setClassFilter] = useState<ClassFilter>("ALL");
  const [searchQuery, setSearchQuery] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("Cards");
  const [sortMode, setSortMode] = useState<SortMode>("InputOrder");

  const parsedTickers = useMemo(
    () => parseInputTickers(inputText),
    [inputText]
  );

  useEffect(() => {
    if (parsedTickers.length > 0) {
      onEnsureTickers(parsedTickers, false);
    }
  }, [parsedTickers.join(",")]);

  // Global Ctrl+V / Cmd+V paste listener when not focused inside an input
  useEffect(() => {
    const onPaste = (e: ClipboardEvent) => {
      const active = document.activeElement;
      if (
        active &&
        (active.tagName === "INPUT" || active.tagName === "TEXTAREA")
      ) {
        return;
      }
      const pasted = e.clipboardData?.getData("text");
      if (pasted && pasted.trim()) {
        const cleaned = pasted.replace(/[\r\n;]+/g, ", ").trim();
        onChangeInputText(
          inputText.trim() ? `${inputText.trim()}, ${cleaned}` : cleaned
        );
      }
    };
    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, [inputText, onChangeInputText]);

  const handlePasteAppend = async () => {
    try {
      const clip = await navigator.clipboard.readText();
      if (clip.trim()) {
        const cleaned = clip.replace(/[\r\n;]+/g, ", ").trim();
        onChangeInputText(
          inputText.trim() ? `${inputText.trim()}, ${cleaned}` : cleaned
        );
      }
    } catch {
      // Clipboard API requires user gesture or permission
    }
  };

  const handlePasteReplace = async () => {
    try {
      const clip = await navigator.clipboard.readText();
      if (clip.trim()) {
        onChangeInputText(clip.replace(/[\r\n;]+/g, ", ").trim());
      }
    } catch {
      // Ignore
    }
  };

  const enrichedItems = useMemo(() => {
    return parsedTickers
      .map((sym, idx) => {
        const snap = assets[sym];
        if (!snap) return null;
        const price = snap.marketPrice ?? snap.indicators.price;
        const trend = TrendingEngine.analyzeStock(
          snap.ticker,
          snap.profile.name,
          snap.profile.subsector,
          price,
          snap.candles,
          snap.indicators,
          snap.coreMultiples
        );
        return { idx, snap, price, trend };
      })
      .filter((x): x is NonNullable<typeof x> => x !== null);
  }, [parsedTickers, assets]);

  const loadingCount = enrichedItems.filter(
    (i) => i.snap.statusInvestLoading
  ).length;

  const filteredAndSorted = useMemo(() => {
    const q = normalize(searchQuery.trim());
    const filtered = enrichedItems.filter(({ snap }) => {
      if (classFilter === "ACAO" && snap.assetClass !== "Acao") return false;
      if (classFilter === "FII" && snap.assetClass !== "Fii") return false;
      if (
        classFilter === "ETF_BDR" &&
        snap.assetClass !== "Etf" &&
        snap.assetClass !== "Bdr"
      ) {
        return false;
      }
      if (q) {
        const match =
          normalize(snap.ticker).includes(q) ||
          normalize(snap.profile.name).includes(q) ||
          normalize(snap.profile.subsector).includes(q);
        if (!match) return false;
      }
      return true;
    });

    return filtered.slice().sort((a, b) => {
      switch (sortMode) {
        case "InputOrder":
          return a.idx - b.idx;
        case "TickerAsc":
          return a.snap.ticker.localeCompare(b.snap.ticker);
        case "PriceDesc":
          return b.price - a.price;
        case "DyDesc":
          return b.trend.dividendYield - a.trend.dividendYield;
        case "ChangeDesc":
          return b.trend.change1dPct - a.trend.change1dPct;
        case "GrahamUpsideDesc":
          return b.trend.grahamUpsidePct - a.trend.grahamUpsidePct;
      }
    });
  }, [enrichedItems, classFilter, searchQuery, sortMode]);

  const renderStatusBadge = (snap: AssetSnapshot) => {
    if (snap.statusInvestLoading) {
      return <span className="badge badge-warning">🔄 Consultando...</span>;
    }
    if (snap.statusInvestAtMs != null) {
      const mins = Math.max(
        0,
        Math.round((Date.now() - snap.statusInvestAtMs) / 60000)
      );
      return (
        <span className="badge badge-positive">
          ✓ StatusInvest Real (há {mins} min)
        </span>
      );
    }
    if (snap.statusInvestOffline) {
      return <span className="badge">⚠ StatusInvest Offline</span>;
    }
    return <span className="badge">Cenário Local / CVM</span>;
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
      {/* Input Card */}
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "8px" }}>
          <div>
            <strong style={{ fontSize: "16px" }}>
              Análise Multiativo por Input ({parsedTickers.length} ativos)
            </strong>
            <div className="text-muted" style={{ fontSize: "12px" }}>
              Digite ou cole códigos B3 separados por vírgula, espaço ou quebra de linha (suporta Ctrl+V direto)
            </div>
          </div>
          <div className="flex-row">
            <button className="btn" onClick={handlePasteAppend}>
              📋 Colar
            </button>
            <button className="btn" onClick={handlePasteReplace}>
              📋 Substituir por Clipboard
            </button>
            <button className="btn" onClick={() => onChangeInputText("")}>
              Limpar
            </button>
          </div>
        </div>

        <textarea
          className="input"
          rows={2}
          value={inputText}
          onChange={(e) => onChangeInputText(e.target.value)}
          placeholder="Ex: PETR4, VALE3, WEGE3, ITUB4, HGLG11, XPML11, BOVA11..."
          style={{ resize: "vertical", fontFamily: "monospace" }}
        />

        {/* Presets */}
        <div className="flex-row" style={{ marginTop: "10px" }}>
          <span className="text-muted" style={{ fontSize: "12px" }}>
            Presets rápidos:
          </span>
          {Object.entries(PRESETS).map(([label, tickersStr]) => (
            <button
              key={label}
              className="chip"
              onClick={() => onChangeInputText(tickersStr)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      {/* Filter & Sort Bar */}
      <div className="card">
        <div className="flex-between" style={{ gap: "10px" }}>
          <div className="flex-row">
            {(
              [
                ["ALL", "Todos"],
                ["ACAO", "Ações"],
                ["FII", "FIIs"],
                ["ETF_BDR", "ETFs/BDRs"],
              ] as const
            ).map(([id, label]) => (
              <button
                key={id}
                className={`chip ${classFilter === id ? "active" : ""}`}
                onClick={() => setClassFilter(id)}
              >
                {label}
              </button>
            ))}
          </div>

          <div className="flex-row">
            <button
              className="btn"
              onClick={() => onEnsureTickers(parsedTickers, true)}
            >
              🔄 Atualizar StatusInvest
              {loadingCount > 0 ? ` (${loadingCount})` : ""}
            </button>
            <button
              className={`chip ${viewMode === "Cards" ? "active" : ""}`}
              onClick={() => setViewMode("Cards")}
            >
              Cartões Detalhados
            </button>
            <button
              className={`chip ${viewMode === "Table" ? "active" : ""}`}
              onClick={() => setViewMode("Table")}
            >
              Tabela Comparativa
            </button>
          </div>
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr auto",
            gap: "10px",
            marginTop: "10px",
          }}
        >
          <input
            type="text"
            className="input"
            placeholder="Filtrar lista carregada por ticker, nome ou setor..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          <select
            className="input"
            style={{ width: "auto" }}
            value={sortMode}
            onChange={(e) => setSortMode(e.target.value as SortMode)}
          >
            <option value="InputOrder">Ordem do Input</option>
            <option value="TickerAsc">Ticker (A-Z)</option>
            <option value="PriceDesc">Maior Preço</option>
            <option value="DyDesc">Maior Dividend Yield</option>
            <option value="ChangeDesc">Maior Variação 1d</option>
            <option value="GrahamUpsideDesc">Maior Desconto Graham</option>
          </select>
        </div>
      </div>

      {/* Results: Cards Mode vs Comparative Table Mode */}
      {viewMode === "Table" ? (
        <div className="table-scroll">
          <table className="data-table">
            <thead>
              <tr>
                <th className="sticky-col">Ticker</th>
                <th>Tipo</th>
                <th>Nome</th>
                <th>Preço</th>
                <th>Var 1d</th>
                <th>P/VP</th>
                <th>P/L</th>
                <th>DY</th>
                <th>Graham / VPA</th>
                <th>Bazin Teto</th>
                <th>ROE / Rend.</th>
                <th>Sinal</th>
                <th>Status</th>
                <th>Ação</th>
              </tr>
            </thead>
            <tbody>
              {filteredAndSorted.map(({ snap, price, trend }) => {
                const isFii = snap.assetClass === "Fii";
                return (
                  <tr key={snap.ticker}>
                    <td className="sticky-col" style={{ fontWeight: 800 }}>
                      {snap.ticker}
                    </td>
                    <td>
                      <span className="badge">{snap.assetClass}</span>
                    </td>
                    <td>{snap.profile.name}</td>
                    <td style={{ fontFamily: "monospace", fontWeight: 700 }}>
                      R$ {price.toFixed(2)}
                    </td>
                    <td
                      className={
                        trend.change1dPct >= 0
                          ? "text-positive"
                          : "text-negative"
                      }
                      style={{ fontFamily: "monospace", fontWeight: 600 }}
                    >
                      {trend.change1dPct >= 0 ? "+" : ""}
                      {trend.change1dPct.toFixed(2)}%
                    </td>
                    <td>{trend.pvp.toFixed(2)}</td>
                    <td>{isFii ? "—" : trend.pl.toFixed(2)}</td>
                    <td className="text-accent" style={{ fontWeight: 700 }}>
                      {trend.dividendYield.toFixed(2)}%
                    </td>
                    <td>
                      R${" "}
                      {(isFii && snap.fiiDetails
                        ? snap.fiiDetails.navPerShare
                        : trend.grahamFairValue
                      ).toFixed(2)}
                    </td>
                    <td>R$ {trend.bazinCeilingPrice.toFixed(2)}</td>
                    <td>
                      {isFii && snap.fiiDetails
                        ? `R$ ${snap.fiiDetails.lastDividend.toFixed(2)}/mês`
                        : `${trend.roe.toFixed(1)}%`}
                    </td>
                    <td>
                      <span
                        className={`badge ${
                          trend.overallScore >= 57
                            ? "badge-positive"
                            : trend.overallScore < 43
                            ? "badge-negative"
                            : ""
                        }`}
                      >
                        {trendSignalLabel(trend.signal)} (
                        {trend.overallScore.toFixed(0)})
                      </span>
                    </td>
                    <td>{renderStatusBadge(snap)}</td>
                    <td>
                      <div className="flex-row" style={{ flexWrap: "nowrap" }}>
                        <button
                          className="btn"
                          onClick={() =>
                            onSelectAssetAndNavigate(snap.ticker, "Decision")
                          }
                        >
                          Decisão
                        </button>
                        <button
                          className="btn"
                          onClick={() =>
                            onSelectAssetAndNavigate(
                              snap.ticker,
                              "StatusInvest"
                            )
                          }
                        >
                          Ficha
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="grid-2">
          {filteredAndSorted.map(({ snap, price, trend }) => {
            const ind = snap.indicators;
            const fii = snap.fiiDetails;
            return (
              <div key={snap.ticker} className="card">
                {/* Card Header */}
                <div className="flex-between" style={{ marginBottom: "10px" }}>
                  <div>
                    <div className="flex-row" style={{ gap: "6px" }}>
                      <strong style={{ fontSize: "20px" }}>{snap.ticker}</strong>
                      <span className="badge">{snap.assetClass}</span>
                      <span className="badge">{snap.profile.subsector}</span>
                    </div>
                    <div className="text-muted" style={{ fontSize: "12px" }}>
                      {snap.profile.name}
                    </div>
                  </div>
                  <div style={{ textAlign: "right" }}>
                    <div style={{ fontSize: "20px", fontWeight: 800 }}>
                      R$ {price.toFixed(2)}
                    </div>
                    <span
                      className={
                        trend.change1dPct >= 0
                          ? "text-positive"
                          : "text-negative"
                      }
                      style={{ fontSize: "12px", fontWeight: 700 }}
                    >
                      {trend.change1dPct >= 0 ? "+" : ""}
                      {trend.change1dPct.toFixed(2)}% (1d)
                    </span>
                  </div>
                </div>

                {/* Asset Class Specific Body */}
                {snap.assetClass === "Fii" && fii ? (
                  <div className="grid-2" style={{ marginBottom: "10px" }}>
                    <div className="card-inset">
                      <div className="kv-row">
                        <IndicatorLabel label="P/VP" className="text-muted" />
                        <strong>
                          {ind.pvp.toFixed(2)} (
                          {ind.pvp < 1
                            ? `Desconto ${((1 - ind.pvp) * 100).toFixed(1)}%`
                            : `Ágio ${((ind.pvp - 1) * 100).toFixed(1)}%`}
                          )
                        </strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel
                          label="VPA (Patrimonial)"
                          className="text-muted"
                        />
                        <strong>R$ {fii.navPerShare.toFixed(2)}</strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel
                          label="Último Rendimento"
                          className="text-muted"
                        />
                        <strong className="text-accent">
                          R$ {fii.lastDividend.toFixed(2)}
                        </strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel label="DY 12M" className="text-muted" />
                        <strong className="text-accent">
                          {ind.dividendYield.toFixed(2)}%
                        </strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel
                          label="Teto Bazin (6%)"
                          className="text-muted"
                        />
                        <strong>R$ {ind.bazinCeilingPrice.toFixed(2)}</strong>
                      </div>
                    </div>

                    <div className="card-inset">
                      <div className="kv-row">
                        <IndicatorLabel
                          label="Segmento"
                          className="text-muted"
                        />
                        <strong>{fiiSegmentDisplayName(fii.segment)}</strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel
                          label="Vacância Física"
                          className="text-muted"
                        />
                        <strong>
                          {fii.physicalVacancyPercent != null
                            ? `${fii.physicalVacancyPercent.toFixed(1)}%`
                            : fii.segment === "PapeisCRIs" ||
                              fii.segment === "Agro" ||
                              fii.segment === "FoF"
                            ? "N/A (Papel)"
                            : "Não informada"}
                        </strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel
                          label="Vacância Financ."
                          className="text-muted"
                        />
                        <strong>
                          {fii.financialVacancyPercent != null
                            ? `${fii.financialVacancyPercent.toFixed(1)}%`
                            : fii.segment === "PapeisCRIs" ||
                              fii.segment === "Agro" ||
                              fii.segment === "FoF"
                            ? "N/A (Papel)"
                            : "Não informada"}
                        </strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel
                          label="Cotistas"
                          className="text-muted"
                        />
                        <strong>
                          {fii.shareholdersCount != null &&
                          fii.shareholdersCount > 0
                            ? fii.shareholdersCount.toLocaleString("pt-BR")
                            : "Não informado"}
                        </strong>
                      </div>
                      <div className="kv-row">
                        <IndicatorLabel
                          label="Taxa Adm."
                          className="text-muted"
                        />
                        <strong>
                          {fii.adminFeePercent != null &&
                          fii.adminFeePercent > 0
                            ? `${fii.adminFeePercent.toFixed(2)}% a.a.`
                            : "Não informada"}
                        </strong>
                      </div>
                    </div>
                  </div>
                ) : snap.assetClass === "Acao" ? (
                  <>
                    <div className="grid-4" style={{ marginBottom: "10px" }}>
                      <div className="card-alt">
                        <div className="kpi-label">
                          <IndicatorLabel label="P/L" />
                        </div>
                        <div style={{ fontWeight: 700, fontSize: "16px" }}>
                          {trend.pl.toFixed(2)}
                        </div>
                      </div>
                      <div className="card-alt">
                        <div className="kpi-label">
                          <IndicatorLabel label="P/VP" />
                        </div>
                        <div style={{ fontWeight: 700, fontSize: "16px" }}>
                          {trend.pvp.toFixed(2)}
                        </div>
                      </div>
                      <div className="card-alt">
                        <div className="kpi-label">
                          <IndicatorLabel label="Dividend Yield" />
                        </div>
                        <div
                          className="text-accent"
                          style={{ fontWeight: 700, fontSize: "16px" }}
                        >
                          {trend.dividendYield.toFixed(2)}%
                        </div>
                      </div>
                      <div className="card-alt">
                        <div className="kpi-label">
                          <IndicatorLabel label="EV/EBITDA" />
                        </div>
                        <div style={{ fontWeight: 700, fontSize: "16px" }}>
                          {ind.evEbitda.toFixed(2)}
                        </div>
                      </div>
                    </div>

                    <div className="grid-2" style={{ marginBottom: "10px" }}>
                      <div className="card-inset">
                        <div className="kv-row">
                          <IndicatorLabel
                            label="ROE / ROIC"
                            lookupKey="ROE"
                            className="text-muted"
                          />
                          <strong>
                            {ind.roe.toFixed(1)}% / {ind.roic.toFixed(1)}%
                          </strong>
                        </div>
                        <div className="kv-row">
                          <IndicatorLabel
                            label="Margem Líq. / Bruta"
                            lookupKey="Margem Líquida"
                            className="text-muted"
                          />
                          <strong>
                            {ind.margemLiquida.toFixed(1)}% /{" "}
                            {ind.margemBruta.toFixed(1)}%
                          </strong>
                        </div>
                        <div className="kv-row">
                          <IndicatorLabel
                            label="Dív. Líq/EBITDA"
                            lookupKey="Dív. Líq / EBITDA"
                            className="text-muted"
                          />
                          <strong>{ind.dividaLiqEbitda.toFixed(2)}x</strong>
                        </div>
                        <div className="kv-row">
                          <IndicatorLabel
                            label="Liquidez Corrente"
                            className="text-muted"
                          />
                          <strong>{ind.liquidezCorrente.toFixed(2)}</strong>
                        </div>
                      </div>

                      <div className="card-inset">
                        <div className="kv-row">
                          <IndicatorLabel
                            label="Graham Justo"
                            className="text-muted"
                          />
                          <strong
                            className={
                              trend.grahamUpsidePct >= 0
                                ? "text-positive"
                                : "text-negative"
                            }
                          >
                            R$ {trend.grahamFairValue.toFixed(2)} (
                            {trend.grahamUpsidePct >= 0 ? "+" : ""}
                            {trend.grahamUpsidePct.toFixed(0)}%)
                          </strong>
                        </div>
                        <div className="kv-row">
                          <IndicatorLabel
                            label="Bazin Teto (6%)"
                            className="text-muted"
                          />
                          <strong
                            className={
                              trend.bazinUpsidePct >= 0
                                ? "text-positive"
                                : "text-negative"
                            }
                          >
                            R$ {trend.bazinCeilingPrice.toFixed(2)} (
                            {trend.bazinUpsidePct >= 0 ? "+" : ""}
                            {trend.bazinUpsidePct.toFixed(0)}%)
                          </strong>
                        </div>
                        <div className="kv-row">
                          <IndicatorLabel
                            label="LPA / VPA / DPA"
                            lookupKey="LPA"
                            className="text-muted"
                          />
                          <strong>
                            {ind.lpa.toFixed(2)} / {ind.vpa.toFixed(2)} /{" "}
                            {ind.dpa.toFixed(2)}
                          </strong>
                        </div>
                        <div className="kv-row">
                          <IndicatorLabel
                            label="F-Score / IFR 14"
                            lookupKey="Piotroski F-Score"
                            className="text-muted"
                          />
                          <strong>
                            {ind.piotroskiFScore}/9 · IFR {trend.rsi14.toFixed(0)}
                          </strong>
                        </div>
                      </div>
                    </div>
                  </>
                ) : (
                  <div className="card-inset" style={{ marginBottom: "10px" }}>
                    <div className="kv-row">
                      <span className="text-muted">Classe / Setor</span>
                      <strong>
                        {snap.assetClass} · {snap.profile.subsector}
                      </strong>
                    </div>
                    <div className="kv-row">
                      <span className="text-muted">Dividend Yield / P/VP</span>
                      <strong>
                        {ind.dividendYield.toFixed(2)}% · P/VP{" "}
                        {ind.pvp.toFixed(2)}
                      </strong>
                    </div>
                    <div
                      className="text-muted"
                      style={{ fontSize: "12px", marginTop: "6px" }}
                    >
                      {snap.profile.description}
                    </div>
                  </div>
                )}

                {/* Card Footer */}
                <div
                  className="flex-between"
                  style={{
                    paddingTop: "10px",
                    borderTop: "1px solid var(--nav-border)",
                  }}
                >
                  <div className="flex-row" style={{ gap: "6px" }}>
                    <span
                      className={`badge ${
                        trend.overallScore >= 57
                          ? "badge-positive"
                          : trend.overallScore < 43
                          ? "badge-negative"
                          : ""
                      }`}
                    >
                      {trendSignalLabel(trend.signal)} · Score{" "}
                      {trend.overallScore.toFixed(0)}/100
                    </span>
                    {renderStatusBadge(snap)}
                  </div>

                  <div className="flex-row" style={{ gap: "6px" }}>
                    <button
                      className="btn"
                      onClick={() =>
                        onSelectAssetAndNavigate(snap.ticker, "Decision")
                      }
                    >
                      Ver Decisão
                    </button>
                    <button
                      className="btn"
                      onClick={() =>
                        onSelectAssetAndNavigate(snap.ticker, "StatusInvest")
                      }
                    >
                      Ficha Completa
                    </button>
                    <button
                      className="btn"
                      onClick={() => onAddToWatchlist(snap.ticker)}
                    >
                      + Watchlist
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
