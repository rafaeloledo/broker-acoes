# Broker-Ações (Web Edition)

A high-performance, self-hosted Brazilian stock market (B3) web terminal and fundamental analysis platform built with **Bun**, **TypeScript**, **React**, and **SQLite (`bun:sqlite`)**, managed with **`mise`**.

Designed for **desktops, tablets, and smartphones**, inheriting all capabilities from the original desktop terminal.

## Features

- **Decision & CVM Fundamentals (`Decisão`)**: Live market quotes (Brapi + Yahoo Finance fallback), BRT market timestamps (`Mercado · HH:MM` vs `Cotação local`), CVM DFP annual statement multiples (`P/L`, `P/VP`, `Dividend Yield`, `Graham`, `Bazin`), and ticker-specific CVM Relevant Facts (`IPE`) + RSS news.
- **Multi-Asset Input Analysis (`Análise por Input`)**: Paste or type any list of B3 tickers (`Ctrl+V` global paste support, 5 strategy presets), real-time live scraping from StatusInvest with SQLite caching, tailored cards for **Ações**, **FIIs**, and **ETFs/BDRs**, plus a 14-column Comparative Table.
- **Quantitative Trending Engine (`Trending`)**: Evaluates all 187 cataloged B3 assets combining technical indicators (`SMA20`, `SMA50`, `RSI-14`, `Bollinger Bands`, `20d Momentum`) and fundamental valuation (`Graham`, `Bazin`, `DY`, `P/L`, `P/VP`, `ROE`, `Margem Líquida`, `Piotroski F-Score`, `Altman Z-Score`).
- **Interactive OHLCV Terminal (`Terminal`)**: High-DPI hardware-accelerated Canvas candlestick charting (`1` focused chart with simulated Level 2 Order Book & 25-row Times & Trades tape, or `2x1` / `2x2` multi-chart grid for up to 4 watchlist assets) streamed over WebSockets.
- **Sector Heatmap (`Mapa`)**: Interactive 12-asset B3 treemap with real-time price variation gradients and live tick flash highlights.
- **Stock Screener (`Busca`)**: Multi-factor filtering with strategy presets (**Joel Greenblatt Magic Formula**, **Décio Bazin Preço Teto**, **Benjamin Graham Deep Value**).
- **Portfolio & Tax (`Carteira & DARF 6015`)**: SINACOR brokerage note parser, manual trade entry, automated Preço Médio (PM) calculation with prorated B3 fees, tax buckets (Swing Trade with R$ 20,000 monthly sales exemption, Day Trade 20%, FIIs 20%, ETFs/BDRs 15%), category-isolated loss carryforward, IRRF deduction, and monthly DARF 6015 reports.
- **Fundamental Sheet & Comparison (`Ficha`)**: Full 30+ indicator sheet, interactive Bazin target yield slider, FII Raio-X with real estate property breakdown, and side-by-side 26-row comparative matrix for 2–4 watchlist tickers.
- **Responsive Design**: Adaptive navigation and layouts for **smartphones** (`< 640px` with bottom navigation bar, touch-scrollable watchlist, and sticky table columns), **tablets** (`640px–1024px`), and **desktops** (`> 1024px`), with **Dark** (`Escuro`) and **Light** (`Claro`) themes.

## Architecture

```text
├── mise.toml            # Toolchain (Bun) and task runner configuration
├── package.json         # Project dependencies & scripts
├── scripts/
│   └── build.ts         # Bun native frontend bundler (dist/)
├── src/
│   ├── core/            # Domain models, 187-asset B3 catalog, quotes, CVM codes, tax & news models
│   ├── analytics/       # Valuation models (Graham, Bazin, Piotroski, Altman Z), TTM, Screener, TrendingEngine
│   ├── portfolio/       # Position tracker (PM) & Brazilian capital gains tax calculator (DARF 6015)
│   ├── storage/         # Embedded SQLite persistence via bun:sqlite (broker.db)
│   ├── ingest/          # Brapi/Yahoo quotes, CVM DFP/IPE, StatusInvest scraper, SINACOR, COTAHIST, RSS
│   ├── shared/          # Shared API & WebSocket types
│   ├── server/          # Bun.serve() HTTP API, WebSocket real-time broadcaster & market feed scheduler
│   └── web/             # Responsive React UI, Dark/Light CSS theme, Canvas Candlestick & Treemap charts
└── tests/
    └── broker.test.ts   # Unit & integration test suite (bun test)
```

## Getting Started with `mise`

### Prerequisites

- [mise](https://mise.jdx.dev/) (`bun` is automatically provisioned via `mise.toml`)

### Tasks

```bash
# Install dependencies
mise run install

# Run in development mode with live reload (http://localhost:3000)
mise run dev

# Build frontend assets and run TypeScript typecheck
mise run build

# Run production server
mise run start

# Run unit & integration test suite
mise run test

# Run TypeScript typecheck
mise run typecheck
```
