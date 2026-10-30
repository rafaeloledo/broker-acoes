---
tipo: sub-issue
epica: 04-alertas
ordem: 3
titulo: Mostrar o histórico de disparos
depende_de:
  - 02-tela-de-decisao/01-modularizar-b3-app
  - 04-alertas/02-avaliar-no-ciclo-de-dados
---

# Mostrar o histórico de disparos

## Contexto

O disparo já fica no banco. Falta criar a regra pela interface e ler o histórico sem abrir o SQLite.

## Escopo

- Visão Alertas: formulário da regra (ticker da watchlist, tipo, parâmetro, ativa) e lista de disparos mais recentes.
- Na Decisão do ativo, mostrar os disparos daquele ticker desde a abertura do dia, se a visão Decisão já existir. Se a épica 02 ainda não tiver a visão, a lista completa basta e o encaixe na Decisão fica anotado no PR.
- Desligar uma regra pela lista de regras.
- Marcar disparo como lido localmente.

## Fora de escopo

Notificação do Windows. Texto gerado por modelo.

## Critérios de aceite

- Criar uma regra de preço pela tela, forçar um candle de teste que cruza o limite e ver o disparo na lista depois do ciclo.
- Desligar a regra impede o próximo cruzamento de aparecer.
