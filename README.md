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

## Persistence & Custom Watchlist

Broker-Ações stores user watchlists, quotes, and portfolio trades in an embedded SQLite database (`broker.db` by default, or via `BROKER_DB_PATH`).

You can customize the active stocks directly in the interface or via SQLite:

```sql
-- View current user watchlist:
SELECT * FROM user_watchlist ORDER BY slot_index ASC;

-- Define your own custom stocks:
DELETE FROM user_watchlist;
INSERT INTO user_watchlist (slot_index, ticker) VALUES
  (0, 'BBAS3'),
  (1, 'TAEE11'),
  (2, 'XPML11'),
  (3, 'B3SA3');
```
