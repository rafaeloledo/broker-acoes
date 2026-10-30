---
tipo: sub-issue
epica: 01-dado-confiavel
ordem: 4
titulo: Calcular o primeiro conjunto de indicadores a partir da CVM
depende_de:
  - 01-cotacao-brapi-na-watchlist
  - 03-periodos-cvm-da-watchlist
---

# Calcular o primeiro conjunto de indicadores a partir da CVM

## Contexto

`b3-analytics` já calcula Graham, Bazin e o filtro do screener a partir de `StockIndicators`. A struct tem dezenas de campos. O primeiro corte precisa de um conjunto pequeno, calculado, e do fallback explícito para o resto.

## Escopo

Primeiro conjunto, quando as contas e o preço existirem:

- lucro por ação, valor patrimonial por ação, dividendo por ação
- P/L, P/VP, dividend yield
- Graham e teto Bazin, reusando `b3-analytics`

O preço desses múltiplos é o último close de `quotes`. Onde a conta não existir, o campo permanece vazio na tela, não volta para `default_for`. Campos fora do primeiro conjunto seguem ocultos ou marcados como indisponíveis.

## Fora de escopo

Piotroski completo, Altman, margens, CAGR e o restante de `StockIndicators`. FIIs, até existir fonte equivalente à DFP/ITR.

## Critérios de aceite

- PETR4 com período e cotação gravados mostra P/L, P/VP, DY, Graham e Bazin calculados, não os literais de `default_for`.
- Ticker sem período não exibe esses múltiplos como se fossem de mercado.
- O teste de Graham e Bazin em `b3-analytics` continua passando com indicadores construídos no teste, sem depender do cenário fixo de produção.
