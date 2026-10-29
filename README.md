# ⚡ Broker-Ações | Self-Hosted Brazilian Stocks GPU Terminal & Platform

A high-performance, self-hosted Brazilian stock market (`B3`) desktop terminal and fundamental analysis platform built in Rust. It combines the deep fundamental analysis of **Status Invest** with a **hardware-accelerated GPU graphics pipeline (`wgpu` + Vulkan)** and an **in-memory data processing engine**.

---

## 🏛️ Architecture Overview

The system is organized as a modular Cargo workspace:

```
broker-acoes/
├── Cargo.toml
├── crates/
│   ├── b3-core/         # Domain types (Tickers, Candles, Accounting codes, Tax rules)
│   ├── b3-ingest/       # B3 COTAHIST (245-byte parser), CVM Open Data (DFP/ITR), SINACOR note parser, Live streamer
│   ├── b3-storage/      # Embedded database (SQLite/DuckDB), quotes & trade persistence
│   ├── b3-analytics/    # 30+ Status Invest indicators, TTM aggregator, Graham/Bazin/DCF models, SIMD screener
│   ├── b3-portfolio/    # Preço Médio (PM), monthly tax calculations, DARF (6015), IRPF reports
│   ├── b3-gpu/          # wgpu render pipelines, WGSL shaders, egui_wgpu paint callbacks, telemetry
│   └── b3-app/          # Native desktop application binary
└── shaders/
    ├── candlestick.wgsl # Hardware-accelerated instanced OHLCV candlestick + wick rendering
    ├── treemap.wgsl     # GPU sector heatmap with real-time tick pulse flashes
    └── sparkline.wgsl   # Batched table sparklines in single draw call
```

---

## 🚀 Key Capabilities

### 1. 📊 Status Invest Fundamental Dashboard
* **30+ Indicators**:
  * **Valuation**: `P/L`, `P/VP`, `EV/EBITDA`, `EV/EBIT`, `P/EBITDA`, `P/SR`, `P/Ativo`, `P/Cap.Giro`, `Dividend Yield (DY)`, `PEG Ratio`.
  * **Endividamento**: `Dívida Líquida / PL`, `Dívida Líquida / EBITDA`, `PL / Ativos`, `Passivos / Ativos`, `Liquidez Corrente`, `Liquidez Seca`.
  * **Eficiência**: `Margem Bruta`, `Margem EBITDA`, `Margem EBIT`, `Margem Líquida`, `Giro dos Ativos`.
  * **Rentabilidade & Crescimento**: `ROE`, `ROIC`, `ROA`, `CAGR Receitas 5A`, `CAGR Lucros 5A`, `LPA`, `VPA`, `DPA`.
* **Valuation & Quality Models**:
  * **Benjamin Graham Fair Value**: $\sqrt{22.5 \times \text{LPA} \times \text{VPA}}$ with live upside/downside %.
  * **Décio Bazin Ceiling Price**: $\text{DPA} / \text{Target Yield}$ with interactive slider (default 6%) and margin of safety.
  * **Piotroski F-Score (0–9)**: Evaluation of profitability, leverage, and operating efficiency.
  * **Altman Z-Score**: Solvency and distress safety zones.
* **Specialized FII View**:
  * Real Estate Funds (`Tijolo`, `Papel`, `Shoppings`, `Agro`) tracking physical/financial vacancy, monthly dividend yield, net asset value (VPA), and multi-property breakdown.

### 2. 📈 Hardware-Accelerated GPU Terminal
* **`wgpu` Direct Vulkan Pipeline**: Instanced OHLCV candlestick chart capable of rendering 100,000+ candles with sub-millisecond frame times.
* **Level 2 Order Book (Livro de Ofertas)**: Real-time bid/ask depth visualization.
* **Times & Trades (Fita de Negócios)**: Live tick stream with buyer/seller aggressor color-coding and broker IDs.
* **Telemetry Overlay**: Real-time FPS (120Hz/240Hz), frame time in milliseconds, and GPU draw call counters.

### 3. 🗺️ GPU Sector Heatmap (Treemap)
* Interactive B3 sector map (Financeiro, Petróleo & Gás, Mineração, Utilidades, Bens Industriais, FIIs).
* Smooth color gradient representing gain/loss with real-time white/cyan flash pulses on incoming ticks.
* One-click navigation to jump directly to any asset's fundamental sheet.

### 4. 🔍 Multi-Factor SIMD Screener
* Dynamic filters for Min DY, Max P/L, Max P/VP, Min ROE, Min Margem Líquida.
* **Preset Strategies**:
  * 🚀 *Fórmula Mágica de Greenblatt* (Rank EY + Rank ROIC)
  * 💰 *Método Décio Bazin* (DY $\ge$ 6%, healthy payout & net debt)
  * 🛡️ *Benjamin Graham Deep Value* (P/L $\le$ 15, P/VP $\le$ 1.5, Liquidez $\ge$ 1.5)
  * 📈 *Status Invest Top Dividendos*

### 5. 💼 Portfolio & Brazilian Tax Engine (DARF 6015 / IRPF)
* **SINACOR Brokerage Note Importer**: Automatic parsing of Brazilian broker notes (XP, Clear, BTG, Inter, Rico).
* **Exact Preço Médio (PM)**: Proration of B3 fees (*Taxa de Liquidação*, *Emolumentos*, *Corretagem*, *ISS*).
* **Receita Federal Compliance**:
  * **R$ 20.000,00 Monthly Sales Exemption**: Automatically marks Ações swing trade profits as exempt when monthly gross sales $\le$ R$ 20.000,00.
  * **Tax Buckets**: Ações Swing (15%), Day Trade (20%), FIIs/Fiagro (20%), ETFs/BDRs (15%).
  * **Loss Carry-Forward**: Automated offset of accumulated historical losses per bucket.
  * **DARF Code 6015**: Exact monthly tax due with due date calculation (last business day of following month).

---

## 🛠️ Build & Run

### Prerequisites
* Rust 1.80+ (tested on Arch Linux Rust 1.98.1)
* Vulkan drivers (e.g., `vulkan-radeon` / `mesa` / `vulkan-intel` / `nvidia-utils`)

### Run the Application
```bash
# Run the desktop terminal in release mode
cargo run --release -p b3-app
```

### Run Tests
```bash
# Execute unit & integration test suite across all workspace crates
cargo test --workspace
```
