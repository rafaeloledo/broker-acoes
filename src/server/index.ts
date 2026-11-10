import { readFileSync } from "node:fs";
import { type ServerWebSocket } from "bun";
import { type TradeOperation } from "../core/index.ts";
import { SinacorParser } from "../ingest/index.ts";
import {
  type AssetSnapshot,
  type WsServerMessage,
} from "../shared/types.ts";
import { buildWebAssets } from "../../scripts/build.ts";
import { BrokerServerState } from "./state.ts";

const INDEX_HTML = `<!DOCTYPE html>
<html lang="pt-BR" data-theme="dark">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=5.0, viewport-fit=cover" />
  <meta name="theme-color" content="#0d1117" />
  <title>Broker Ações — Terminal B3 & Análise Fundamentalista</title>
  <link rel="stylesheet" href="/assets/styles.css" />
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/assets/main.js"></script>
</body>
</html>`;

export interface RunningBrokerServer {
  server: ReturnType<typeof Bun.serve>;
  state: BrokerServerState;
  stop: () => void;
}

export async function createBrokerServer(options?: {
  port?: number;
  dbPath?: string;
  enableIntervals?: boolean;
}): Promise<RunningBrokerServer> {
  await buildWebAssets();

  const state = new BrokerServerState(
    options?.dbPath ?? process.env.BROKER_DB_PATH ?? "broker.db"
  );
  const wsClients = new Set<ServerWebSocket<unknown>>();

  state.setBroadcaster((msg: WsServerMessage) => {
    if (wsClients.size === 0) return;
    const payload = JSON.stringify(msg);
    for (const ws of wsClients) {
      try {
        ws.send(payload);
      } catch {
        wsClients.delete(ws);
      }
    }
  });

  const port = options?.port ?? Number(process.env.PORT ?? 3000);
  const enableIntervals = options?.enableIntervals ?? true;

  let simTimer: ReturnType<typeof setInterval> | null = null;
  let feedTimer: ReturnType<typeof setInterval> | null = null;

  if (enableIntervals) {
    simTimer = setInterval(() => {
      state.stepSimulationTick();
    }, 650);

    feedTimer = setInterval(() => {
      void state.syncMarketData(false);
    }, 20_000);

    // Initial non-blocking market feed check
    setTimeout(() => {
      void state.syncMarketData(false);
    }, 500);
  }

  const server = Bun.serve({
    port,
    async fetch(req, srv) {
      const url = new URL(req.url);
      const { pathname } = url;

      if (pathname === "/ws") {
        const upgraded = srv.upgrade(req);
        if (upgraded) return undefined;
        return new Response("WebSocket upgrade failed", { status: 400 });
      }

      if (pathname === "/" || pathname === "/index.html") {
        return new Response(INDEX_HTML, {
          headers: { "Content-Type": "text/html; charset=utf-8" },
        });
      }

      if (pathname === "/assets/styles.css") {
        const css = readFileSync("src/web/styles.css", "utf-8");
        return new Response(css, {
          headers: { "Content-Type": "text/css; charset=utf-8" },
        });
      }

      if (pathname === "/assets/main.js") {
        return new Response(Bun.file("dist/main.js"), {
          headers: { "Content-Type": "application/javascript; charset=utf-8" },
        });
      }

      // REST API Routes
      if (pathname === "/api/state" && req.method === "GET") {
        return Response.json(state.getInitialStatePayload());
      }

      if (pathname === "/api/watchlist" && req.method === "POST") {
        const body = (await req.json()) as { tickers?: string[] };
        const visibleTickers = state.updateWatchlist(body.tickers ?? []);
        const assets: Record<string, AssetSnapshot> = {};
        for (const sym of visibleTickers) {
          assets[sym] = state.ensureAssetLoaded(sym);
        }
        return Response.json({ visibleTickers, assets });
      }

      if (pathname === "/api/assets/batch" && req.method === "POST") {
        const body = (await req.json()) as {
          tickers?: string[];
          forceStatusInvest?: boolean;
        };
        const tickers = body.tickers ?? [];
        const force = Boolean(body.forceStatusInvest);
        const assets: Record<string, AssetSnapshot> = {};

        for (const raw of tickers) {
          const snap = state.ensureAssetLoaded(raw);
          assets[snap.ticker] = snap;
          // Trigger non-blocking StatusInvest query
          void state.queryStatusInvest(snap.ticker, force);
        }
        return Response.json({ assets });
      }

      if (pathname === "/api/simulation" && req.method === "POST") {
        const body = (await req.json()) as { running?: boolean };
        state.isLiveSimRunning = Boolean(body.running);
        return Response.json({ isLiveSimRunning: state.isLiveSimRunning });
      }

      if (pathname === "/api/news" && req.method === "GET") {
        const ticker = url.searchParams.get("ticker");
        const text = url.searchParams.get("text");
        const fromMs = url.searchParams.get("fromMs")
          ? Number(url.searchParams.get("fromMs"))
          : null;
        const items = state.db.searchNews(ticker, text, fromMs, null, 200);
        return Response.json({ items });
      }

      if (pathname === "/api/news/sync" && req.method === "POST") {
        const recentNews = await state.syncNews();
        return Response.json({ recentNews });
      }

      if (pathname === "/api/portfolio/sinacor" && req.method === "POST") {
        const body = (await req.json()) as {
          noteText?: string;
          brokerName?: string;
        };
        const parsed = SinacorParser.parseTradeLines(
          body.noteText ?? "",
          body.brokerName ?? "XP Investimentos"
        );
        const portfolio = state.addTrades(parsed);
        return Response.json({ imported: parsed.length, portfolio });
      }

      if (pathname === "/api/portfolio/trade" && req.method === "POST") {
        const body = (await req.json()) as Partial<TradeOperation>;
        const sym = (body.symbol ?? "PETR4").trim().toUpperCase();
        const side = body.side === "Sell" ? "Sell" : "Buy";
        const kind = body.kind === "DayTrade" ? "DayTrade" : "SwingTrade";
        const quantity = Number(body.quantity) || 100;
        const unitPrice = Number(body.unitPrice) || 10;
        const totalGross = Number(body.totalGross) || quantity * unitPrice;
        const date = body.date || new Date().toISOString().slice(0, 10);

        const trade: TradeOperation = {
          id: body.id || `TRD-${Date.now()}-${Math.floor(Math.random() * 1000)}`,
          date,
          symbol: sym,
          side,
          kind,
          quantity,
          unitPrice,
          totalGross,
          liquidationFee: Math.max(totalGross * 0.00025, 0.01),
          registrationFee: Math.max(totalGross * 0.00005, 0.01),
          brokerageFee: 0.0,
          issTax: 0.0,
          irrf: side === "Sell" ? totalGross * 0.00005 : 0.0,
          brokerName: body.brokerName || "Corretora B3",
          noteNumber: body.noteNumber || "MANUAL",
        };
        const portfolio = state.addTrades([trade]);
        return Response.json({ trade, portfolio });
      }

      if (
        pathname.startsWith("/api/portfolio/trade/") &&
        req.method === "DELETE"
      ) {
        const id = decodeURIComponent(
          pathname.replace("/api/portfolio/trade/", "")
        );
        const portfolio = state.deleteTrade(id);
        return Response.json({ portfolio });
      }

      if (pathname === "/api/quotes/cotahist" && req.method === "POST") {
        const body = (await req.json()) as { content?: string };
        const res = state.importCotahist(body.content ?? "");
        return Response.json(res);
      }

      return new Response("Not Found", { status: 404 });
    },
    websocket: {
      open(ws) {
        wsClients.add(ws);
      },
      message() {
        // Client messages not needed currently
      },
      close(ws) {
        wsClients.delete(ws);
      },
    },
  });

  return {
    server,
    state,
    stop: () => {
      if (simTimer) clearInterval(simTimer);
      if (feedTimer) clearInterval(feedTimer);
      server.stop(true);
      state.db.close();
    },
  };
}

if (import.meta.main) {
  const running = await createBrokerServer();
  console.log(
    `🚀 Broker-Ações Web Server running at http://localhost:${running.server.port}`
  );
}
