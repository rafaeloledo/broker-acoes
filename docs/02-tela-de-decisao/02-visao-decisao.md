---
tipo: sub-issue
epica: 02-tela-de-decisao
ordem: 2
titulo: Criar a visão Decisão como entrada do app
depende_de:
  - 01-dado-confiavel/01-cotacao-brapi-na-watchlist
  - 01-dado-confiavel/04-indicadores-a-partir-da-cvm
  - 02-tela-de-decisao/01-modularizar-b3-app
---

# Criar a visão Decisão como entrada do app

## Contexto

Status Invest já mostra a ficha e a comparação, com números demais e origem opaca. A entrada do app precisa de um resumo do ativo focado.

## Escopo

- Nova variante de visão, selecionada ao abrir.
- Bloco do ativo focado: ticker, preço de mercado, horário da última coleta, variação contra o candle anterior se existir.
- Bloco de valuation com o primeiro conjunto da épica 01: P/L, P/VP, DY, Graham, Bazin.
- Campo sem dado fica como indisponível, não como zero e não como `default_for`.
- A troca de ativo da watchlist atualiza essa visão.

## Fora de escopo

Notícias e alertas nessa tela. Eles entram nas épicas 03 e 04, em seções próprias. Redesign do terminal GPU.

## Critérios de aceite

- Abrir o app cai na Decisão, no primeiro ticker da watchlist.
- Os números batem com `quotes` e com os indicadores calculados, não com os literais de `default_for`.
- Ativo sem período mostra preço, se houver, e valuation indisponível.
