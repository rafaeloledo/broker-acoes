import React, { useEffect, useRef, useState } from "react";
import { type Candle } from "../../core/quote.ts";
import { type HeatmapNode } from "../../shared/types.ts";

interface CandlestickChartProps {
  symbol: string;
  candles: readonly Candle[];
  candleWidth?: number;
  visibleCount?: number;
  height?: number;
  darkTheme?: boolean;
  onTelemetry?: (fps: number, frameMs: number) => void;
}

export const CandlestickChart: React.FC<CandlestickChartProps> = ({
  symbol,
  candles,
  candleWidth = 9,
  visibleCount = 70,
  height = 420,
  darkTheme = true,
  onTelemetry,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [hoverIdx, setHoverIdx] = useState<number | null>(null);
  const fpsState = useRef({
    lastTime: performance.now(),
    fps: 60.0,
    frameMs: 16.6,
  });

  const slice = candles.slice(-visibleCount);

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const t0 = performance.now();
    const dpr = window.devicePixelRatio || 1;
    const width = Math.max(260, container.clientWidth);
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.scale(dpr, dpr);

    // Background
    ctx.fillStyle = darkTheme ? "#0d1117" : "#ffffff";
    ctx.fillRect(0, 0, width, height);

    if (slice.length === 0) return;

    const padLeft = 10;
    const padRight = 58;
    const padTop = 18;
    const padBottom = 26;
    const plotW = Math.max(50, width - padLeft - padRight);
    const plotH = Math.max(50, height - padTop - padBottom);

    let minP = Number.POSITIVE_INFINITY;
    let maxP = Number.NEGATIVE_INFINITY;
    let maxVol = 1;

    for (const c of slice) {
      if (c.low < minP) minP = c.low;
      if (c.high > maxP) maxP = c.high;
      if (c.volume > maxVol) maxVol = c.volume;
    }

    const priceRange = Math.max(0.05, maxP - minP);
    minP -= priceRange * 0.06;
    maxP += priceRange * 0.06;
    const span = Math.max(0.01, maxP - minP);

    const priceToY = (p: number) =>
      padTop + (1.0 - (p - minP) / span) * plotH;

    // Horizontal grid lines & right price labels
    ctx.strokeStyle = darkTheme
      ? "rgba(48, 54, 61, 0.65)"
      : "rgba(208, 215, 222, 0.8)";
    ctx.lineWidth = 1;
    ctx.fillStyle = darkTheme ? "#8b949e" : "#57606a";
    ctx.font = "11px monospace";
    ctx.textAlign = "left";

    const gridSteps = 5;
    for (let i = 0; i <= gridSteps; i++) {
      const ratio = i / gridSteps;
      const y = padTop + ratio * plotH;
      const pVal = maxP - ratio * span;
      ctx.beginPath();
      ctx.moveTo(padLeft, y);
      ctx.lineTo(padLeft + plotW, y);
      ctx.stroke();
      ctx.fillText(pVal.toFixed(2), padLeft + plotW + 6, y + 4);
    }

    const stepX = plotW / Math.max(slice.length, 1);
    const bodyW = Math.max(2, Math.min(candleWidth, stepX * 0.82));

    // Volume bars at bottom 18% of plot
    const volMaxH = plotH * 0.18;
    for (let i = 0; i < slice.length; i++) {
      const c = slice[i]!;
      const xCenter = padLeft + (i + 0.5) * stepX;
      const isUp = c.close >= c.open;
      const vH = (c.volume / maxVol) * volMaxH;
      ctx.fillStyle = isUp
        ? "rgba(8, 153, 129, 0.22)"
        : "rgba(242, 54, 69, 0.22)";
      ctx.fillRect(
        xCenter - bodyW / 2,
        padTop + plotH - vH,
        bodyW,
        vH
      );
    }

    // SMA 20 line
    ctx.strokeStyle = "rgba(180, 140, 255, 0.75)";
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    let startedSma = false;
    for (let i = 0; i < slice.length; i++) {
      const start = Math.max(0, i - 19);
      const sub = slice.slice(start, i + 1);
      const avg = sub.reduce((a, b) => a + b.close, 0) / sub.length;
      const x = padLeft + (i + 0.5) * stepX;
      const y = priceToY(avg);
      if (!startedSma) {
        ctx.moveTo(x, y);
        startedSma = true;
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.stroke();

    // Candles (matching candlestick.wgsl colors: #089981 bullish, #F23645 bearish)
    for (let i = 0; i < slice.length; i++) {
      const c = slice[i]!;
      const xCenter = padLeft + (i + 0.5) * stepX;
      const isUp = c.close >= c.open;
      const color = isUp ? "#089981" : "#F23645";

      const highY = priceToY(c.high);
      const lowY = priceToY(c.low);
      const openY = priceToY(c.open);
      const closeY = priceToY(c.close);

      // Wick
      ctx.strokeStyle = color;
      ctx.lineWidth = Math.max(1, bodyW * 0.16);
      ctx.beginPath();
      ctx.moveTo(xCenter, highY);
      ctx.lineTo(xCenter, lowY);
      ctx.stroke();

      // Body
      const topY = Math.min(openY, closeY);
      const bodyH = Math.max(1.5, Math.abs(closeY - openY));
      ctx.fillStyle = color;
      ctx.fillRect(xCenter - bodyW / 2, topY, bodyW, bodyH);
    }

    // Current price dashed line & badge
    const lastCandle = slice[slice.length - 1]!;
    const lastY = priceToY(lastCandle.close);
    const isLastUp = lastCandle.close >= lastCandle.open;
    ctx.save();
    ctx.setLineDash([4, 4]);
    ctx.strokeStyle = isLastUp ? "#089981" : "#F23645";
    ctx.beginPath();
    ctx.moveTo(padLeft, lastY);
    ctx.lineTo(padLeft + plotW, lastY);
    ctx.stroke();
    ctx.restore();

    ctx.fillStyle = isLastUp ? "#089981" : "#F23645";
    ctx.fillRect(padLeft + plotW + 2, lastY - 9, padRight - 4, 18);
    ctx.fillStyle = "#ffffff";
    ctx.font = "bold 11px monospace";
    ctx.fillText(lastCandle.close.toFixed(2), padLeft + plotW + 6, lastY + 4);

    // Crosshair on hover/touch
    if (hoverIdx != null && hoverIdx >= 0 && hoverIdx < slice.length) {
      const hc = slice[hoverIdx]!;
      const hx = padLeft + (hoverIdx + 0.5) * stepX;
      const hy = priceToY(hc.close);

      ctx.save();
      ctx.setLineDash([3, 3]);
      ctx.strokeStyle = darkTheme
        ? "rgba(255, 255, 255, 0.45)"
        : "rgba(31, 35, 40, 0.45)";
      ctx.beginPath();
      ctx.moveTo(hx, padTop);
      ctx.lineTo(hx, padTop + plotH);
      ctx.moveTo(padLeft, hy);
      ctx.lineTo(padLeft + plotW, hy);
      ctx.stroke();
      ctx.restore();
    }

    // Telemetry
    const now = performance.now();
    const elapsed = Math.max(1, now - fpsState.current.lastTime);
    const instFps = Math.min(120, 1000 / elapsed);
    fpsState.current.fps = fpsState.current.fps * 0.9 + instFps * 0.1;
    fpsState.current.frameMs = Math.max(0.2, now - t0);
    fpsState.current.lastTime = now;
    onTelemetry?.(fpsState.current.fps, fpsState.current.frameMs);
  }, [slice, candleWidth, height, darkTheme, hoverIdx, symbol]);

  const handlePointer = (clientX: number) => {
    const canvas = canvasRef.current;
    if (!canvas || slice.length === 0) return;
    const rect = canvas.getBoundingClientRect();
    const x = clientX - rect.left - 10;
    const plotW = Math.max(50, rect.width - 68);
    const idx = Math.floor((x / plotW) * slice.length);
    if (idx >= 0 && idx < slice.length) {
      setHoverIdx(idx);
    } else {
      setHoverIdx(null);
    }
  };

  const activeCandle =
    hoverIdx != null && slice[hoverIdx]
      ? slice[hoverIdx]!
      : slice[slice.length - 1];

  return (
    <div
      ref={containerRef}
      style={{ width: "100%", position: "relative", userSelect: "none" }}
    >
      {activeCandle && (
        <div
          style={{
            display: "flex",
            gap: "10px",
            flexWrap: "wrap",
            fontSize: "11px",
            fontFamily: "monospace",
            padding: "4px 8px",
            color: "var(--subtle)",
          }}
        >
          <strong style={{ color: "var(--text)" }}>{symbol}</strong>
          <span>A: R$ {activeCandle.open.toFixed(2)}</span>
          <span>M: R$ {activeCandle.high.toFixed(2)}</span>
          <span>m: R$ {activeCandle.low.toFixed(2)}</span>
          <span
            style={{
              color:
                activeCandle.close >= activeCandle.open
                  ? "var(--positive)"
                  : "var(--negative)",
              fontWeight: 700,
            }}
          >
            F: R$ {activeCandle.close.toFixed(2)}
          </span>
          <span>
            Vol: {(activeCandle.volume / 1_000_000).toFixed(2)}M
          </span>
        </div>
      )}
      <canvas
        ref={canvasRef}
        onMouseMove={(e) => handlePointer(e.clientX)}
        onMouseLeave={() => setHoverIdx(null)}
        onTouchMove={(e) => {
          if (e.touches[0]) handlePointer(e.touches[0].clientX);
        }}
        onTouchEnd={() => setHoverIdx(null)}
        style={{
          display: "block",
          borderRadius: "8px",
          border: "1px solid var(--nav-border)",
          cursor: "crosshair",
        }}
      />
    </div>
  );
};

interface TreemapHeatmapProps {
  nodes: readonly HeatmapNode[];
  onSelectSymbol: (symbol: string) => void;
}

export const TreemapHeatmap: React.FC<TreemapHeatmapProps> = ({
  nodes,
  onSelectSymbol,
}) => {
  const getCellBackground = (changePct: number, flashIntensity: number) => {
    const t = Math.max(0.0, Math.min(1.0, Math.abs(changePct) / 4.0));
    let r = 0.18;
    let g = 0.2;
    let b = 0.24;

    if (changePct > 0.001) {
      r = 0.1 + (0.03 - 0.1) * t;
      g = 0.22 + (0.6 - 0.22) * t;
      b = 0.16 + (0.3 - 0.16) * t;
    } else if (changePct < -0.001) {
      r = 0.28 + (0.75 - 0.28) * t;
      g = 0.12 + (0.15 - 0.12) * t;
      b = 0.14 + (0.2 - 0.14) * t;
    }

    const f = Math.max(0.0, Math.min(1.0, flashIntensity)) * 0.45;
    const rf = Math.round((r + (1.0 - r) * f) * 255);
    const gf = Math.round((g + (1.0 - g) * f) * 255);
    const bf = Math.round((b + (1.0 - b) * f) * 255);
    return `rgb(${rf}, ${gf}, ${bf})`;
  };

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
        gap: "8px",
        width: "100%",
      }}
    >
      {nodes.map((node) => {
        const bg = getCellBackground(node.changePct, node.flashIntensity);
        const sign = node.changePct >= 0 ? "+" : "";
        return (
          <button
            key={node.symbol}
            onClick={() => onSelectSymbol(node.symbol)}
            style={{
              background: bg,
              border:
                node.flashIntensity > 0.3
                  ? "2px solid rgba(255,255,255,0.85)"
                  : "2px solid rgb(31, 33, 41)",
              borderRadius: "10px",
              padding: "18px 14px",
              minHeight: "136px",
              color: "#ffffff",
              display: "flex",
              flexDirection: "column",
              justifyContent: "space-between",
              alignItems: "flex-start",
              cursor: "pointer",
              textAlign: "left",
              transition: "transform 0.12s ease",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.12)",
            }}
          >
            <div>
              <div
                style={{
                  fontSize: "20px",
                  fontWeight: 800,
                  letterSpacing: "-0.4px",
                }}
              >
                {node.symbol}
              </div>
              <div
                style={{
                  fontSize: "11px",
                  opacity: 0.85,
                  marginTop: "2px",
                  maxWidth: "190px",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                }}
              >
                {node.name}
              </div>
            </div>
            <div style={{ width: "100%" }}>
              <div
                style={{
                  fontSize: "18px",
                  fontWeight: 800,
                  fontFamily: "monospace",
                }}
              >
                {sign}
                {node.changePct.toFixed(2)}%
              </div>
              <div
                style={{
                  fontSize: "12px",
                  opacity: 0.9,
                  fontFamily: "monospace",
                }}
              >
                R$ {node.price.toFixed(2)}
              </div>
            </div>
          </button>
        );
      })}
    </div>
  );
};
