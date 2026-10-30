# Documentação de etapas

Este diretório é a fila de trabalho do Broker-Ações até ele funcionar como apoio à decisão: preço e fundamento confiáveis, notícias do ativo, alertas e leitura por IA em OpenAI, Anthropic ou OpenRouter.

Ainda não há issues no GitHub. Cada pasta é uma épica. Cada arquivo numerado dentro dela é uma sub-issue. O `README.md` da pasta é o corpo da épica.

## Como virar issue

1. Abrir uma issue épica com o `titulo` do `README.md` da pasta.
2. Abrir uma sub-issue para cada arquivo `NN-*.md`, na ordem do campo `ordem`.
3. Usar o título do front matter. O corpo da issue é o restante do arquivo.
4. Ligar a sub-issue à épica. Copiar `depende_de` como relação de bloqueio.
5. Não juntar duas sub-issues no mesmo PR, a menos que o arquivo diga o contrário.

Sugestão de labels, quando o repositório passar a usá-las:

| Label | Uso |
| --- | --- |
| `epic` | Issue pai da pasta |
| `sub-issue` | Arquivo numerado |
| `area:dados` | Épica 01 |
| `area:interface` | Épica 02 |
| `area:noticias` | Épica 03 |
| `area:alertas` | Épica 04 |
| `area:ia` | Épica 05 |

## Ordem

A épica 01 destrava as outras. Sem cotação e fundamento de mercado, notícia, alerta e parecer descrevem o cenário gravado em `StockIndicators::default_for`.

```text
01 Dado confiável
   ├─ 02 Tela de decisão
   ├─ 03 Notícias
   ├─ 04 Alertas
   └─ 05 IA
        ├─ briefing        espera 03
        ├─ memorando       espera 01 e 02
        └─ explicação      espera 04
```

Os adaptadores de IA (contrato, chave, OpenAI, Anthropic, OpenRouter) podem ser escritos em paralelo à coleta de notícias. Os três usos do modelo esperam o dado que vão ler.

## Épicas

| Ordem | Pasta | Issue épica |
| --- | --- | --- |
| 1 | [01-dado-confiavel](01-dado-confiavel/README.md) | Dado confiável para a decisão |
| 2 | [02-tela-de-decisao](02-tela-de-decisao/README.md) | Tela de decisão e organização da interface |
| 3 | [03-noticias](03-noticias/README.md) | Busca de notícias por ativo |
| 4 | [04-alertas](04-alertas/README.md) | Alertas locais |
| 5 | [05-ia-provedores](05-ia-provedores/README.md) | IA em OpenAI, Anthropic e OpenRouter |

## O que o código já é

Terminal desktop em Rust. Sete crates: `b3-core`, `b3-ingest`, `b3-storage`, `b3-analytics`, `b3-portfolio`, `b3-gpu`, `b3-app`. Cinco telas em `crates/b3-app/src/main.rs`: Status Invest, Terminal GPU, Mapa B3, Screener e Carteira & DARF. Watchlist de até 4 ativos em SQLite (`user_watchlist`).

Gráfico, fórmulas de valuation, parser de nota SINACOR e motor de DARF são lógica real. A ficha fundamentalista usa `default_for`. O feed ao vivo da tela chama `simulate_tick`. `fetch_brapi_quote` existe em `crates/b3-ingest/src/live.rs` e a interface não o chama. `CvmIngestor` só interpreta uma linha CSV. Não há notícia, alerta nem cliente de modelo.
