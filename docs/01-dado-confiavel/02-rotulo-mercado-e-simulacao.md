---
tipo: sub-issue
epica: 01-dado-confiavel
ordem: 2
titulo: Rotular dado de mercado e dado simulado
depende_de:
  - 01-cotacao-brapi-na-watchlist
---

# Rotular dado de mercado e dado simulado

## Contexto

O checkbox "Feed Real-Time (Ticks ao Vivo)" chama `LiveFeedClient::simulate_tick`. O heatmap também pulsa em cima desse contador. Isso é útil para o terminal GPU e enganoso numa tela de decisão.

## Escopo

- Preço vindo de `quotes` / Brapi aparece como mercado, com o horário da última atualização.
- Book, tape e pulso do heatmap permanecem no Terminal GPU e no mapa, com o texto de simulação visível.
- O checkbox deixa de se chamar feed ao vivo.
- Indicador que ainda sair de `default_for` aparece como cenário local, até a sub-issue dos indicadores da CVM substituí-lo.

## Fora de escopo

Desligar a simulação. Ela continua disponível para o terminal.

## Critérios de aceite

- Na ficha, o preço de mercado e o horário da coleta são distinguíveis de qualquer número sintético.
- No terminal, o usuário lê que book e tape são simulação antes de interpretar o book.
