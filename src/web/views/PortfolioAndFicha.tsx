import React, { useState } from "react";
import {
  fiiSegmentDisplayName,
  fundamentalsCaption,
  marketSectorDisplayName,
  priceCaption,
  type TradeOperation,
} from "../../core/index.ts";
import { SAMPLE_SINACOR_NOTE } from "../../ingest/sinacor.ts";
import {
  type AssetSnapshot,
  type PortfolioSnapshot,
} from "../../shared/types.ts";
import { IndicatorLabel } from "../components/IndicatorTooltip.tsx";

interface PortfolioViewProps {
  portfolio: PortfolioSnapshot;
  onImportSinacor: (noteText: string, brokerName: string) => Promise<void>;
  onAddManualTrade: (trade: Partial<TradeOperation>) => Promise<void>;
  onDeleteTrade: (id: string) => Promise<void>;
}

export const PortfolioView: React.FC<PortfolioViewProps> = ({
  portfolio,
  onImportSinacor,
  onAddManualTrade,
  onDeleteTrade,
}) => {
  const [showSinacorModal, setShowSinacorModal] = useState(false);
  const [sinacorText, setSinacorText] = useState(SAMPLE_SINACOR_NOTE);
  const [brokerName, setBrokerName] = useState("XP Investimentos");
  const [showTradesTable, setShowTradesTable] = useState(false);

  const [newSymbol, setNewSymbol] = useState("PETR4");
  const [newSide, setNewSide] = useState<"Buy" | "Sell">("Buy");
  const [newKind, setNewKind] = useState<"SwingTrade" | "DayTrade">("SwingTrade");
  const [newDate, setNewDate] = useState(new Date().toISOString().slice(0, 10));
  const [newQty, setNewQty] = useState("100");
  const [newPrice, setNewPrice] = useState("36.50");

  const handleManualSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const qty = Number(newQty) || 0;
    const price = Number(newPrice) || 0;
    if (!newSymbol.trim() || qty <= 0 || price <= 0) return;

    await onAddManualTrade({
      symbol: newSymbol.trim().toUpperCase(),
      side: newSide,
      kind: newKind,
      date: newDate,
      quantity: qty,
      unitPrice: price,
      totalGross: qty * price,
      brokerName: "Corretora B3",
    });
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      {/* Summary Banner */}
      <div className="grid-3">
        <div className="card">
          <div className="kpi-label">Patrimônio em Custódia</div>
          <div className="kpi-value">
            R${" "}
            {portfolio.totalCustodyValue.toLocaleString("pt-BR", {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </div>
          <div className="kpi-sub">
            {portfolio.positions.length} posições em aberto
          </div>
        </div>

        <div className="card">
          <div className="kpi-label">Total Investido (Custo PM)</div>
          <div className="kpi-value">
            R${" "}
            {portfolio.totalInvested.toLocaleString("pt-BR", {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </div>
          <div className="kpi-sub">Inclui taxas operacionais B3</div>
        </div>

        <div className="card">
          <div className="kpi-label">Resultado Não Realizado (Lucro/Prejuízo)</div>
          <div
            className={`kpi-value ${
              portfolio.unrealizedPnl >= 0 ? "text-positive" : "text-negative"
            }`}
          >
            {portfolio.unrealizedPnl >= 0 ? "+" : ""}R${" "}
            {portfolio.unrealizedPnl.toLocaleString("pt-BR", {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}{" "}
            ({portfolio.unrealizedPnlPercent >= 0 ? "+" : ""}
            {portfolio.unrealizedPnlPercent.toFixed(2)}%)
          </div>
          <div className="kpi-sub">Marcação a mercado atual</div>
        </div>
      </div>

      {/* Open Positions Table */}
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "12px" }}>
          <div>
            <strong style={{ fontSize: "16px" }}>
              Posições em Custódia & Preço Médio (PM)
            </strong>
            <div className="text-muted" style={{ fontSize: "12px" }}>
              Cálculo automático de PM com rateio de emolumentos e taxa de liquidação B3
            </div>
          </div>
          <div className="flex-row">
            <button
              className="btn btn-primary"
              onClick={() => setShowSinacorModal(true)}
            >
              📄 Importar Nota SINACOR
            </button>
            <button
              className="btn"
              onClick={() => setShowTradesTable(!showTradesTable)}
            >
              {showTradesTable
                ? "Ocultar Histórico de Ordens"
                : `Ver Ordens (${portfolio.trades.length})`}
            </button>
          </div>
        </div>

        <div className="table-scroll">
          <table className="data-table">
            <thead>
              <tr>
                <th className="sticky-col">Ticker</th>
                <th>Quantidade</th>
                <th>Preço Médio (PM)</th>
                <th>Cotação Atual</th>
                <th>Valor Aplicado</th>
                <th>Saldo Atual</th>
                <th>Retorno (R$)</th>
                <th>Retorno (%)</th>
              </tr>
            </thead>
            <tbody>
              {portfolio.positions.map((pos) => (
                <tr key={pos.symbol}>
                  <td className="sticky-col" style={{ fontWeight: 800 }}>
                    {pos.symbol}
                  </td>
                  <td style={{ fontFamily: "monospace" }}>{pos.quantity}</td>
                  <td style={{ fontFamily: "monospace" }}>
                    R$ {pos.averagePrice.toFixed(2)}
                  </td>
                  <td style={{ fontFamily: "monospace", fontWeight: 700 }}>
                    R$ {pos.currentPrice.toFixed(2)}
                  </td>
                  <td style={{ fontFamily: "monospace" }}>
                    R$ {pos.totalInvested.toFixed(2)}
                  </td>
                  <td style={{ fontFamily: "monospace", fontWeight: 700 }}>
                    R$ {pos.currentValue.toFixed(2)}
                  </td>
                  <td
                    className={
                      pos.unrealizedPnl >= 0
                        ? "text-positive"
                        : "text-negative"
                    }
                    style={{ fontFamily: "monospace", fontWeight: 700 }}
                  >
                    {pos.unrealizedPnl >= 0 ? "+" : ""}R${" "}
                    {pos.unrealizedPnl.toFixed(2)}
                  </td>
                  <td
                    className={
                      pos.unrealizedPnlPercent >= 0
                        ? "text-positive"
                        : "text-negative"
                    }
                    style={{ fontFamily: "monospace", fontWeight: 700 }}
                  >
                    {pos.unrealizedPnlPercent >= 0 ? "+" : ""}
                    {pos.unrealizedPnlPercent.toFixed(2)}%
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* Manual Trade Form */}
        <form
          onSubmit={handleManualSubmit}
          className="card-inset"
          style={{ marginTop: "12px" }}
        >
          <div
            style={{
              fontWeight: 700,
              fontSize: "12px",
              marginBottom: "8px",
              color: "var(--label)",
              textTransform: "uppercase",
            }}
          >
            + Registrar Operação Manualmente
          </div>
          <div className="grid-6" style={{ alignItems: "end" }}>
            <div>
              <div className="kpi-label">Ativo</div>
              <input
                className="input"
                value={newSymbol}
                onChange={(e) => setNewSymbol(e.target.value)}
                placeholder="PETR4"
              />
            </div>
            <div>
              <div className="kpi-label">Lado</div>
              <select
                className="input"
                value={newSide}
                onChange={(e) => setNewSide(e.target.value as "Buy" | "Sell")}
              >
                <option value="Buy">Compra (C)</option>
                <option value="Sell">Venda (V)</option>
              </select>
            </div>
            <div>
              <div className="kpi-label">Modalidade</div>
              <select
                className="input"
                value={newKind}
                onChange={(e) =>
                  setNewKind(e.target.value as "SwingTrade" | "DayTrade")
                }
              >
                <option value="SwingTrade">Swing Trade</option>
                <option value="DayTrade">Day Trade</option>
              </select>
            </div>
            <div>
              <div className="kpi-label">Data</div>
              <input
                type="date"
                className="input"
                value={newDate}
                onChange={(e) => setNewDate(e.target.value)}
              />
            </div>
            <div>
              <div className="kpi-label">Qtd / Preço (R$)</div>
              <div style={{ display: "flex", gap: "4px" }}>
                <input
                  type="number"
                  className="input"
                  value={newQty}
                  onChange={(e) => setNewQty(e.target.value)}
                  placeholder="Qtd"
                />
                <input
                  type="number"
                  step="0.01"
                  className="input"
                  value={newPrice}
                  onChange={(e) => setNewPrice(e.target.value)}
                  placeholder="R$"
                />
              </div>
            </div>
            <button type="submit" className="btn btn-primary">
              Adicionar Ordem
            </button>
          </div>
        </form>

        {/* Optional Trade History Table */}
        {showTradesTable && (
          <div className="table-scroll" style={{ marginTop: "12px" }}>
            <table className="data-table">
              <thead>
                <tr>
                  <th>Data</th>
                  <th>Ativo</th>
                  <th>Lado</th>
                  <th>Tipo</th>
                  <th>Qtd</th>
                  <th>Preço Unit.</th>
                  <th>Total Bruto</th>
                  <th>Taxas B3</th>
                  <th>IRRF</th>
                  <th>Corretora</th>
                  <th>Ação</th>
                </tr>
              </thead>
              <tbody>
                {portfolio.trades.map((t) => (
                  <tr key={t.id}>
                    <td>{t.date}</td>
                    <td style={{ fontWeight: 700 }}>{t.symbol}</td>
                    <td>
                      <span
                        className={`badge ${
                          t.side === "Buy" ? "badge-positive" : "badge-negative"
                        }`}
                      >
                        {t.side === "Buy" ? "COMPRA" : "VENDA"}
                      </span>
                    </td>
                    <td>{t.kind === "DayTrade" ? "Day Trade" : "Swing"}</td>
                    <td>{t.quantity}</td>
                    <td>R$ {t.unitPrice.toFixed(2)}</td>
                    <td>R$ {t.totalGross.toFixed(2)}</td>
                    <td>
                      R$ {(t.liquidationFee + t.registrationFee).toFixed(2)}
                    </td>
                    <td>R$ {t.irrf.toFixed(2)}</td>
                    <td>{t.brokerName}</td>
                    <td>
                      <button
                        className="watchlist-mini-btn danger"
                        onClick={() => onDeleteTrade(t.id)}
                      >
                        Excluir
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Monthly Tax & DARF 6015 Section */}
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "12px" }}>
          <div>
            <strong style={{ fontSize: "16px" }}>
              Apuração Mensal de IR & Relatório DARF 6015
            </strong>
            <div className="text-muted" style={{ fontSize: "12px" }}>
              Regra de isenção de R$ 20.000/mês em vendas de ações Swing Trade, compensação de prejuízos por categoria e dedução de IRRF (dedo-duro)
            </div>
          </div>
        </div>

        <div className="grid-2">
          {portfolio.monthlyReports.map((rep) => {
            const monthStr = String(rep.month).padStart(2, "0");
            return (
              <div key={`${rep.year}-${rep.month}`} className="card-alt">
                <div className="flex-between" style={{ marginBottom: "8px" }}>
                  <strong style={{ fontSize: "16px" }}>
                    Competência {monthStr}/{rep.year}
                  </strong>
                  <span
                    className={`badge ${
                      rep.isAcoesExempt ? "badge-positive" : "badge-warning"
                    }`}
                  >
                    {rep.isAcoesExempt
                      ? "ISENTO (Vendas ≤ R$ 20.000)"
                      : "TRIBUTÁVEL"}
                  </span>
                </div>

                <div className="card-inset" style={{ marginBottom: "8px" }}>
                  <div className="kv-row">
                    <span className="text-muted">Vendas Ações Swing</span>
                    <strong>R$ {rep.acoesSwingSales.toFixed(2)}</strong>
                  </div>
                  <div className="kv-row">
                    <span className="text-muted">Resultado Ações Swing (15%)</span>
                    <strong
                      className={
                        rep.netResultAcoesSwing >= 0
                          ? "text-positive"
                          : "text-negative"
                      }
                    >
                      R$ {rep.netResultAcoesSwing.toFixed(2)}
                    </strong>
                  </div>
                  <div className="kv-row">
                    <span className="text-muted">Resultado Day Trade (20%)</span>
                    <strong>R$ {rep.netResultDayTrade.toFixed(2)}</strong>
                  </div>
                  <div className="kv-row">
                    <span className="text-muted">Resultado FIIs (20%)</span>
                    <strong>R$ {rep.netResultFii.toFixed(2)}</strong>
                  </div>
                  <div className="kv-row">
                    <span className="text-muted">Prejuízo Compensado</span>
                    <strong>
                      R${" "}
                      {(
                        rep.lossOffsetAcoesSwing +
                        rep.lossOffsetDayTrade +
                        rep.lossOffsetFii +
                        rep.lossOffsetEtfBdr
                      ).toFixed(2)}
                    </strong>
                  </div>
                  <div className="kv-row">
                    <span className="text-muted">IRRF Deduzido (Dedo-Duro)</span>
                    <strong>R$ {rep.irrfDeducted.toFixed(2)}</strong>
                  </div>
                  <div className="kv-row">
                    <span className="text-muted">Prejuízo Acumulado a Compensar</span>
                    <strong>
                      R${" "}
                      {(
                        rep.carryoverLossAcoesSwing +
                        rep.carryoverLossDayTrade +
                        rep.carryoverLossFii +
                        rep.carryoverLossEtfBdr
                      ).toFixed(2)}
                    </strong>
                  </div>
                </div>

                <div className="flex-between">
                  <div>
                    <span className="text-muted" style={{ fontSize: "12px" }}>
                      Código Receita: <strong>{rep.darfCode}</strong> · Vencimento:{" "}
                      <strong>{rep.dueDate}</strong>
                    </span>
                  </div>
                  <div style={{ fontSize: "16px", fontWeight: 800 }}>
                    DARF: R$ {rep.totalDarfDue.toFixed(2)}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* SINACOR Import Modal */}
      {showSinacorModal && (
        <div
          className="modal-backdrop"
          onClick={() => setShowSinacorModal(false)}
        >
          <div className="modal-window" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <span>📄 Importar Nota de Corretagem (Padrão SINACOR / B3)</span>
              <button
                className="icon-btn"
                onClick={() => setShowSinacorModal(false)}
              >
                ✕
              </button>
            </div>
            <div className="modal-body">
              <div style={{ marginBottom: "10px" }}>
                <div className="kpi-label">Corretora</div>
                <input
                  className="input"
                  value={brokerName}
                  onChange={(e) => setBrokerName(e.target.value)}
                />
              </div>
              <div style={{ marginBottom: "12px" }}>
                <div className="kpi-label">
                  Conteúdo da Nota SINACOR (linhas 1-BOVESPA C/V VISTA...)
                </div>
                <textarea
                  className="input"
                  rows={8}
                  value={sinacorText}
                  onChange={(e) => setSinacorText(e.target.value)}
                  style={{ fontFamily: "monospace", fontSize: "12px" }}
                />
              </div>
              <div className="flex-between">
                <button
                  className="btn"
                  onClick={() => setSinacorText(SAMPLE_SINACOR_NOTE)}
                >
                  Restaurar Exemplo
                </button>
                <button
                  className="btn btn-primary"
                  onClick={async () => {
                    await onImportSinacor(sinacorText, brokerName);
                    setShowSinacorModal(false);
                  }}
                >
                  Processar e Importar Ordens
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

interface FichaViewProps {
  visibleTickers: readonly string[];
  selectedTicker: string;
  assets: Record<string, AssetSnapshot>;
  onFocusOnlyOne: (ticker: string) => void;
  onOpenSearchModal: (slot: number | null) => void;
  onRemoveFromWatchlist: (ticker: string) => void;
}

export const FichaView: React.FC<FichaViewProps> = ({
  visibleTickers,
  selectedTicker,
  assets,
  onFocusOnlyOne,
  onOpenSearchModal,
  onRemoveFromWatchlist,
}) => {
  const [targetYield, setTargetYield] = useState(6.0);
  const [forceSingle, setForceSingle] = useState(false);

  const isComparison = visibleTickers.length > 1 && !forceSingle;
  const activeSym =
    visibleTickers.includes(selectedTicker)
      ? selectedTicker
      : visibleTickers[0] ?? "PETR4";
  const singleAsset = assets[activeSym];

  if (!isComparison && singleAsset) {
    const ind = singleAsset.indicators;
    const prof = singleAsset.profile;
    const fii = singleAsset.fiiDetails;
    const price = singleAsset.marketPrice ?? ind.price;
    const core = singleAsset.coreMultiples;

    const graham = core?.grahamFairValue ?? ind.grahamFairValue;
    const grahamUpside =
      price > 0 && graham > 0 ? ((graham - price) / price) * 100.0 : 0.0;

    const dpa = core?.dpa ?? ind.dpa;
    const bazinCeiling =
      singleAsset.cvmYear != null && core?.bazinCeilingPrice != null
        ? core.bazinCeilingPrice
        : dpa / (targetYield / 100.0);
    const bazinMargin =
      price > 0 && bazinCeiling > 0
        ? ((bazinCeiling - price) / price) * 100.0
        : 0.0;

    const altmanZone =
      ind.altmanZScore <= 0
        ? "Setor Financeiro / N/A"
        : ind.altmanZScore >= 2.99
        ? "Zona Segura (> 2,99)"
        : ind.altmanZScore >= 1.81
        ? "Zona de Alerta (1,81–2,99)"
        : "Zona de Estresse (< 1,81)";

    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
        {/* Header Card */}
        <div className="card">
          <div className="flex-between">
            <div>
              <div className="flex-row" style={{ gap: "8px", marginBottom: "4px" }}>
                <span style={{ fontSize: "26px", fontWeight: 800 }}>
                  {singleAsset.ticker}
                </span>
                <span className="badge">{prof.segment}</span>
                <span className="badge">
                  {marketSectorDisplayName(prof.sector)}
                </span>
                {visibleTickers.length > 1 && (
                  <button
                    className="chip active"
                    onClick={() => setForceSingle(false)}
                  >
                    Comparar {visibleTickers.length} Ativos Lado a Lado
                  </button>
                )}
              </div>
              <div className="text-subtle" style={{ fontSize: "15px" }}>
                {prof.name} · {prof.subsector}
              </div>
            </div>

            <div style={{ textAlign: "right" }}>
              <div style={{ fontSize: "26px", fontWeight: 800 }}>
                R$ {price.toFixed(2)}
              </div>
              <div className="flex-row" style={{ justifyContent: "flex-end" }}>
                <span className="badge">
                  {priceCaption(singleAsset.marketFetchedAtMs)}
                </span>
                <span className="badge">
                  {fundamentalsCaption(singleAsset.cvmYear)}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* 6 Top KPI Cards */}
        <div className="grid-6">
          <div className="card-alt">
            <div className="kpi-label">
              <IndicatorLabel label="Dividend yield" lookupKey="Dividend Yield" />
            </div>
            <div className="kpi-value text-accent">
              {(core?.dividendYield ?? ind.dividendYield).toFixed(2)}%
            </div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">
              <IndicatorLabel label="P/L" />
            </div>
            <div className="kpi-value">
              {(core?.pl ?? ind.pl).toFixed(2)}
            </div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">
              <IndicatorLabel label="P/VP" />
            </div>
            <div className="kpi-value">
              {(core?.pvp ?? ind.pvp).toFixed(2)}
            </div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">
              <IndicatorLabel label="EV/EBITDA" />
            </div>
            <div className="kpi-value">{ind.evEbitda.toFixed(2)}</div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">
              <IndicatorLabel label="ROE" />
            </div>
            <div className="kpi-value">{ind.roe.toFixed(1)}%</div>
          </div>
          <div className="card-alt">
            <div className="kpi-label">
              <IndicatorLabel label="ROIC" />
            </div>
            <div className="kpi-value">{ind.roic.toFixed(1)}%</div>
          </div>
        </div>

        {/* 3 Valuation & Health Cards */}
        <div className="grid-3">
          <div className="card">
            <div className="kpi-label">
              <IndicatorLabel label="Preço justo de Graham" />
            </div>
            <div className="kpi-value">R$ {graham.toFixed(2)}</div>
            <div
              className={
                grahamUpside >= 0 ? "text-positive" : "text-negative"
              }
              style={{ fontWeight: 700, marginTop: "4px" }}
            >
              {grahamUpside >= 0 ? "+" : ""}
              {grahamUpside.toFixed(1)}% vs cotação atual
            </div>
            <div className="kpi-sub">
              Fórmula: √(22,5 × LPA R$ {(core?.lpa ?? ind.lpa).toFixed(2)} × VPA R${" "}
              {(core?.vpa ?? ind.vpa).toFixed(2)})
            </div>
          </div>

          <div className="card">
            <div className="kpi-label">
              <IndicatorLabel label="Preço teto Bazin" />
            </div>
            <div className="kpi-value">R$ {bazinCeiling.toFixed(2)}</div>
            <div
              className={bazinMargin >= 0 ? "text-positive" : "text-negative"}
              style={{ fontWeight: 700, marginTop: "4px" }}
            >
              Margem: {bazinMargin >= 0 ? "+" : ""}
              {bazinMargin.toFixed(1)}%
            </div>
            {singleAsset.cvmYear == null ? (
              <div style={{ marginTop: "6px" }}>
                <div className="flex-between" style={{ fontSize: "11px" }}>
                  <span className="text-muted">Yield alvo:</span>
                  <strong>{targetYield.toFixed(1)}% a.a.</strong>
                </div>
                <input
                  type="range"
                  min={4.0}
                  max={12.0}
                  step={0.5}
                  value={targetYield}
                  onChange={(e) => setTargetYield(Number(e.target.value))}
                  style={{ width: "100%" }}
                />
              </div>
            ) : (
              <div className="kpi-sub">DPA CVM / 6,0% a.a.</div>
            )}
          </div>

          <div className="card">
            <div className="kpi-label">
              <IndicatorLabel label="Piotroski e Altman Z" />
            </div>
            <div className="kpi-value">F-Score {ind.piotroskiFScore}/9</div>
            <div style={{ fontWeight: 600, marginTop: "4px" }}>
              Altman Z: {ind.altmanZScore.toFixed(2)}
            </div>
            <div className="kpi-sub">{altmanZone}</div>
          </div>
        </div>

        {/* Optional FII Raio-X Card */}
        {fii && (
          <div className="card">
            <strong style={{ fontSize: "15px", display: "block", marginBottom: "10px" }}>
              🏢 Raio-X do Fundo Imobiliário (FII) — {fii.symbol}
            </strong>
            <div className="grid-4" style={{ marginBottom: "12px" }}>
              <div className="card-inset">
                <div className="kpi-label">
                  <IndicatorLabel label="Segmento & Mandato" lookupKey="Segmento" />
                </div>
                <strong>{fiiSegmentDisplayName(fii.segment)}</strong>
                <div className="kpi-sub">{fii.mandate}</div>
              </div>
              <div className="card-inset">
                <div className="kpi-label">
                  <IndicatorLabel label="VP por Cota / Patrimônio" lookupKey="VPA (Patrimonial)" />
                </div>
                <strong>R$ {fii.navPerShare.toFixed(2)}</strong>
                <div className="kpi-sub">
                  PL: R$ {(fii.netAssetValue / 1_000_000).toFixed(0)}M
                </div>
              </div>
              <div className="card-inset">
                <div className="kpi-label">
                  <IndicatorLabel label="Vacância Física / Financ." />
                </div>
                <strong>
                  {fii.physicalVacancyPercent != null
                    ? `${fii.physicalVacancyPercent.toFixed(1)}%`
                    : "N/A"}{" "}
                  /{" "}
                  {fii.financialVacancyPercent != null
                    ? `${fii.financialVacancyPercent.toFixed(1)}%`
                    : "N/A"}
                </strong>
                <div className="kpi-sub">
                  {fii.shareholdersCount != null && fii.shareholdersCount > 0
                    ? `${fii.shareholdersCount.toLocaleString("pt-BR")} cotistas`
                    : "Cotistas: N/D"}
                </div>
              </div>
              <div className="card-inset">
                <div className="kpi-label">
                  <IndicatorLabel label="Último Rendimento" />
                </div>
                <strong className="text-accent">
                  R$ {fii.lastDividend.toFixed(2)} / cota
                </strong>
                <div className="kpi-sub">
                  <IndicatorLabel label="Taxa Adm." />:{" "}
                  {fii.adminFeePercent != null && fii.adminFeePercent > 0
                    ? `${fii.adminFeePercent.toFixed(2)}% a.a.`
                    : "Não informada"}
                </div>
              </div>
            </div>

            {fii.properties.length > 0 && (
              <div className="table-scroll">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Ativo Imobiliário</th>
                      <th>Localização</th>
                      <th>ABL (m²)</th>
                      <th>Unidades</th>
                      <th>Inquilinos</th>
                      <th>Vacância</th>
                    </tr>
                  </thead>
                  <tbody>
                    {fii.properties.map((p, idx) => (
                      <tr key={idx}>
                        <td style={{ fontWeight: 600 }}>{p.name}</td>
                        <td>
                          {p.city}/{p.state} — {p.address}
                        </td>
                        <td>{p.areaM2.toLocaleString("pt-BR")} m²</td>
                        <td>{p.unitsCount}</td>
                        <td>{p.tenantsCount}</td>
                        <td>{p.vacancyPercent.toFixed(1)}%</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {/* 4-Column Fundamental Indicators Grid */}
        <div className="grid-4">
          <div className="card">
            <strong
              style={{
                display: "block",
                marginBottom: "8px",
                color: "var(--accent)",
              }}
            >
              Valuation
            </strong>
            <div className="kv-row">
              <IndicatorLabel label="P/L" className="text-muted" />
              <strong>{ind.pl.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="P/VP" className="text-muted" />
              <strong>{ind.pvp.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="EV/EBITDA" className="text-muted" />
              <strong>{ind.evEbitda.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="EV/EBIT" className="text-muted" />
              <strong>{ind.evEbit.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="P/EBITDA" className="text-muted" />
              <strong>{ind.pEbitda.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="P/EBIT" className="text-muted" />
              <strong>{ind.pEbit.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="P/SR (Receita)" className="text-muted" />
              <strong>{ind.pSr.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="P/Ativo" className="text-muted" />
              <strong>{ind.pAtivo.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="P/Cap. Giro" className="text-muted" />
              <strong>{ind.pCapGiro.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="PEG Ratio" className="text-muted" />
              <strong>{ind.pegRatio.toFixed(2)}</strong>
            </div>
          </div>

          <div className="card">
            <strong
              style={{
                display: "block",
                marginBottom: "8px",
                color: "var(--warning)",
              }}
            >
              Endividamento
            </strong>
            <div className="kv-row">
              <IndicatorLabel label="Dív. Líquida / PL" className="text-muted" />
              <strong>{ind.dividaLiqPl.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Dív. Líquida / EBITDA" className="text-muted" />
              <strong>{ind.dividaLiqEbitda.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Dív. Líquida / EBIT" className="text-muted" />
              <strong>{ind.dividaLiqEbit.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="PL / Ativos" className="text-muted" />
              <strong>{ind.plAtivos.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Passivos / Ativos" className="text-muted" />
              <strong>{ind.passivosAtivos.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Liquidez Corrente" className="text-muted" />
              <strong>{ind.liquidezCorrente.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Liquidez Seca" className="text-muted" />
              <strong>{ind.liquidezSeca.toFixed(2)}</strong>
            </div>
          </div>

          <div className="card">
            <strong
              style={{
                display: "block",
                marginBottom: "8px",
                color: "var(--violet)",
              }}
            >
              Eficiência
            </strong>
            <div className="kv-row">
              <IndicatorLabel label="Margem Bruta" className="text-muted" />
              <strong>{ind.margemBruta.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Margem EBITDA" className="text-muted" />
              <strong>{ind.margemEbitda.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Margem EBIT" className="text-muted" />
              <strong>{ind.margemEbit.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Margem Líquida" className="text-muted" />
              <strong>{ind.margemLiquida.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="Giro de Ativos" className="text-muted" />
              <strong>{ind.giroAtivos.toFixed(2)}</strong>
            </div>
          </div>

          <div className="card">
            <strong
              style={{
                display: "block",
                marginBottom: "8px",
                color: "var(--positive)",
              }}
            >
              Rentabilidade & Por Ação
            </strong>
            <div className="kv-row">
              <IndicatorLabel label="ROE" className="text-muted" />
              <strong>{ind.roe.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="ROIC" className="text-muted" />
              <strong>{ind.roic.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="ROA" className="text-muted" />
              <strong>{ind.roa.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="CAGR Receitas (5a)" className="text-muted" />
              <strong>{ind.cagrReceitas5a.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="CAGR Lucros (5a)" className="text-muted" />
              <strong>{ind.cagrLucros5a.toFixed(1)}%</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="LPA" className="text-muted" />
              <strong>R$ {ind.lpa.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="VPA" className="text-muted" />
              <strong>R$ {ind.vpa.toFixed(2)}</strong>
            </div>
            <div className="kv-row">
              <IndicatorLabel label="DPA" className="text-muted" />
              <strong>R$ {ind.dpa.toFixed(2)}</strong>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Multi-Asset Comparison View (2..4 tickers)
  const comparedAssets = visibleTickers
    .map((sym) => assets[sym])
    .filter((a): a is AssetSnapshot => a != null);

  const matrixRows: Array<{
    label: string;
    render: (a: AssetSnapshot) => string;
  }> = [
    { label: "Preço Atual", render: (a) => `R$ ${(a.marketPrice ?? a.indicators.price).toFixed(2)}` },
    { label: "Dividend Yield", render: (a) => `${a.indicators.dividendYield.toFixed(2)}%` },
    { label: "P/L", render: (a) => a.indicators.pl.toFixed(2) },
    { label: "P/VP", render: (a) => a.indicators.pvp.toFixed(2) },
    { label: "EV/EBITDA", render: (a) => a.indicators.evEbitda.toFixed(2) },
    { label: "EV/EBIT", render: (a) => a.indicators.evEbit.toFixed(2) },
    { label: "P/EBITDA", render: (a) => a.indicators.pEbitda.toFixed(2) },
    { label: "P/EBIT", render: (a) => a.indicators.pEbit.toFixed(2) },
    { label: "P/SR", render: (a) => a.indicators.pSr.toFixed(2) },
    { label: "P/Ativo", render: (a) => a.indicators.pAtivo.toFixed(2) },
    { label: "P/Cap. Giro", render: (a) => a.indicators.pCapGiro.toFixed(2) },
    { label: "PEG Ratio", render: (a) => a.indicators.pegRatio.toFixed(2) },
    { label: "LPA", render: (a) => `R$ ${a.indicators.lpa.toFixed(2)}` },
    { label: "VPA", render: (a) => `R$ ${a.indicators.vpa.toFixed(2)}` },
    { label: "DPA", render: (a) => `R$ ${a.indicators.dpa.toFixed(2)}` },
    { label: "Graham Justo", render: (a) => `R$ ${a.indicators.grahamFairValue.toFixed(2)}` },
    { label: "Bazin Teto (6%)", render: (a) => `R$ ${a.indicators.bazinCeilingPrice.toFixed(2)}` },
    { label: "ROE", render: (a) => `${a.indicators.roe.toFixed(1)}%` },
    { label: "ROIC", render: (a) => `${a.indicators.roic.toFixed(1)}%` },
    { label: "ROA", render: (a) => `${a.indicators.roa.toFixed(1)}%` },
    { label: "Margem Bruta", render: (a) => `${a.indicators.margemBruta.toFixed(1)}%` },
    { label: "Margem EBITDA", render: (a) => `${a.indicators.margemEbitda.toFixed(1)}%` },
    { label: "Margem Líquida", render: (a) => `${a.indicators.margemLiquida.toFixed(1)}%` },
    { label: "Dív. Líq / PL", render: (a) => a.indicators.dividaLiqPl.toFixed(2) },
    { label: "Dív. Líq / EBITDA", render: (a) => a.indicators.dividaLiqEbitda.toFixed(2) },
    { label: "Liquidez Corrente", render: (a) => a.indicators.liquidezCorrente.toFixed(2) },
    { label: "Piotroski F-Score", render: (a) => `${a.indicators.piotroskiFScore}/9` },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      {/* Side-by-side Asset Cards */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: `repeat(auto-fit, minmax(230px, 1fr))`,
          gap: "12px",
        }}
      >
        {comparedAssets.map((asset, idx) => {
          const price = asset.marketPrice ?? asset.indicators.price;
          return (
            <div key={asset.ticker} className="card">
              <div className="flex-between" style={{ marginBottom: "8px" }}>
                <div>
                  <strong style={{ fontSize: "20px" }}>{asset.ticker}</strong>
                  <div className="text-muted" style={{ fontSize: "12px" }}>
                    {asset.profile.name}
                  </div>
                </div>
                <strong style={{ fontSize: "18px" }}>
                  R$ {price.toFixed(2)}
                </strong>
              </div>

              <div className="flex-row" style={{ marginBottom: "10px" }}>
                <button
                  className="btn"
                  onClick={() => {
                    onFocusOnlyOne(asset.ticker);
                    setForceSingle(true);
                  }}
                >
                  Focar
                </button>
                <button
                  className="btn"
                  onClick={() => onOpenSearchModal(idx)}
                >
                  Trocar
                </button>
                {comparedAssets.length > 1 && (
                  <button
                    className="btn"
                    onClick={() => onRemoveFromWatchlist(asset.ticker)}
                  >
                    Remover
                  </button>
                )}
              </div>

              <div className="card-inset">
                <div className="kv-row">
                  <IndicatorLabel label="Graham Justo" className="text-muted" />
                  <strong>
                    R$ {asset.indicators.grahamFairValue.toFixed(2)}
                  </strong>
                </div>
                <div className="kv-row">
                  <IndicatorLabel
                    label="Bazin Teto"
                    lookupKey="Bazin Teto (6%)"
                    className="text-muted"
                  />
                  <strong>
                    R$ {asset.indicators.bazinCeilingPrice.toFixed(2)}
                  </strong>
                </div>
                <div className="kv-row">
                  <span className="text-muted">DY / P/L / P/VP</span>
                  <strong>
                    {asset.indicators.dividendYield.toFixed(1)}% ·{" "}
                    {asset.indicators.pl.toFixed(1)} ·{" "}
                    {asset.indicators.pvp.toFixed(2)}
                  </strong>
                </div>
                <div className="kv-row">
                  <IndicatorLabel
                    label="F-Score / ROE"
                    lookupKey="Piotroski F-Score"
                    className="text-muted"
                  />
                  <strong>
                    {asset.indicators.piotroskiFScore}/9 ·{" "}
                    {asset.indicators.roe.toFixed(1)}%
                  </strong>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* 26-row Comparative Matrix */}
      <div className="card">
        <div className="flex-between" style={{ marginBottom: "10px" }}>
          <strong style={{ fontSize: "15px" }}>
            Matriz Comparativa de Indicadores Fundamentalistas
          </strong>
          <span className="text-muted" style={{ fontSize: "12px" }}>
            Passe o mouse (ou toque) sobre qualquer indicador ⓘ para ver a leitura analítica
          </span>
        </div>
        <div className="table-scroll">
          <table className="data-table">
            <thead>
              <tr>
                <th className="sticky-col">Indicador</th>
                {comparedAssets.map((a) => (
                  <th key={a.ticker}>{a.ticker}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {matrixRows.map((row) => (
                <tr key={row.label}>
                  <td className="sticky-col" style={{ fontWeight: 600 }}>
                    <IndicatorLabel label={row.label} />
                  </td>
                  {comparedAssets.map((a) => (
                    <td key={a.ticker} style={{ fontFamily: "monospace" }}>
                      {row.render(a)}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
