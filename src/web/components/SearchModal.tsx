import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  type AssetClass,
  type B3Asset,
  searchB3Assets,
} from "../../core/index.ts";

interface SearchModalProps {
  isOpen: boolean;
  targetSlot: number | null;
  visibleTickers: readonly string[];
  onClose: () => void;
  onSelectTicker: (ticker: string, slot: number | null) => void;
}

export const SearchModal: React.FC<SearchModalProps> = ({
  isOpen,
  targetSlot,
  visibleTickers,
  onClose,
  onSelectTicker,
}) => {
  const [query, setQuery] = useState("");
  const [filterClass, setFilterClass] = useState<AssetClass | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery("");
      setTimeout(() => inputRef.current?.focus(), 40);
    }
  }, [isOpen]);

  const results = useMemo(
    () => searchB3Assets(query, filterClass).slice(0, 60),
    [query, filterClass]
  );

  if (!isOpen) return null;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-window" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <span>
            🔍 Buscar Ativo B3{" "}
            {targetSlot != null ? `(Trocar Slot ${targetSlot + 1})` : "(Adicionar)"}
          </span>
          <button className="icon-btn" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="modal-body">
          <input
            ref={inputRef}
            type="text"
            className="input"
            placeholder="Digite o código (PETR4, HGLG11, AZZA3), nome ou setor..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            style={{ marginBottom: "10px" }}
          />

          <div className="flex-row" style={{ marginBottom: "12px" }}>
            {(
              [
                [null, "Todos"],
                ["Acao", "Ações"],
                ["Fii", "FIIs"],
                ["Etf", "ETFs"],
                ["Bdr", "BDRs"],
              ] as const
            ).map(([cls, label]) => (
              <button
                key={label}
                className={`chip ${filterClass === cls ? "active" : ""}`}
                onClick={() => setFilterClass(cls)}
              >
                {label}
              </button>
            ))}
          </div>

          <div className="table-scroll" style={{ maxHeight: "360px" }}>
            <table className="data-table">
              <thead>
                <tr>
                  <th className="sticky-col">Código</th>
                  <th>Nome do Ativo</th>
                  <th>Setor</th>
                  <th>Tipo</th>
                  <th>Preço Ref.</th>
                  <th>Ação</th>
                </tr>
              </thead>
              <tbody>
                {results.map((asset: B3Asset) => {
                  const inWatchlist = visibleTickers.includes(asset.ticker);
                  return (
                    <tr key={asset.ticker}>
                      <td className="sticky-col" style={{ fontWeight: 700 }}>
                        {asset.ticker}
                      </td>
                      <td>{asset.name}</td>
                      <td className="text-muted">{asset.sector}</td>
                      <td>
                        <span className="badge">{asset.assetClass}</span>
                      </td>
                      <td style={{ fontFamily: "monospace" }}>
                        R$ {asset.basePrice.toFixed(2)}
                      </td>
                      <td>
                        <button
                          className={`btn ${
                            inWatchlist ? "" : "btn-primary"
                          }`}
                          onClick={() => {
                            onSelectTicker(asset.ticker, targetSlot);
                            onClose();
                          }}
                        >
                          {inWatchlist ? "Na Tela" : "Selecionar"}
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
};
