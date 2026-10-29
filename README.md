# Broker-Ações

A high-performance, self-hosted Brazilian stock market (B3) desktop terminal and fundamental analysis platform built in Rust, powered by `wgpu` (Vulkan).

## Features

- **Fundamental Analysis**: Complete financial metrics (valuation, debt, efficiency, growth), valuation models (Graham, Bazin, Piotroski F-Score), and specialized FII tracking.
- **GPU Terminal**: Hardware-accelerated OHLCV candlestick charting capable of 100k+ candles at high refresh rates, real-time Level 2 order book, and Times & Trades tape.
- **Sector Heatmap**: Interactive GPU treemap with real-time price change gradients and tick pulse highlights.
- **Stock Screener**: Multi-factor filtering with strategy presets (Greenblatt Magic Formula, Décio Bazin, Graham Deep Value).
- **Portfolio & Tax**: SINACOR brokerage note parser, automated Preço Médio (PM) calculation with B3 fees, tax buckets (swing trade, day trade, FIIs), and DARF reports.

## Architecture

```
crates/
├── b3-core/         # Domain models, indicators, tax rules
├── b3-ingest/       # COTAHIST, CVM open data, SINACOR parser, live feed
├── b3-storage/      # Embedded database & persistence
├── b3-analytics/    # Valuation models, metrics & screener
├── b3-portfolio/    # Position tracker & Brazilian tax engine
├── b3-gpu/          # wgpu render pipelines & WGSL shaders
└── b3-app/          # Desktop application binary
```

## Getting Started

### Prerequisites

- Rust 1.80+
- Vulkan-compatible GPU and drivers

### Run

```bash
cargo run --release -p b3-app
```

### Test

```bash
cargo test --workspace
```
