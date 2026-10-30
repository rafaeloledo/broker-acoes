---
tipo: sub-issue
epica: 02-tela-de-decisao
ordem: 3
titulo: Mover terminal, mapa, screener e carteira para ferramentas
depende_de:
  - 02-tela-de-decisao/02-visao-decisao
---

# Mover terminal, mapa, screener e carteira para ferramentas

## Contexto

As cinco abas competem com a decisão. Gráfico GPU, heatmap, screener e DARF continuam úteis, e deixam de ser o caminho de quem abriu o app para ver um papel.

## Escopo

- Barra principal: Decisão e o seletor de ativos da watchlist.
- Grupo Ferramentas: Terminal, Mapa, Screener, Carteira.
- O estado de cada ferramenta permanece o que a extração de módulos já preservou.
- Atalho de voltar para a Decisão no ativo que estava focado.

## Fora de escopo

Mudar o cálculo do screener, do imposto ou do renderer. Aumentar o limite de 4 ativos. Isso pode ser uma issue futura se a watchlist passar a ser lista, não grade fixa.

## Critérios de aceite

- Do início ao fim, o usuário chega na Decisão sem passar pelo terminal.
- Cada ferramenta abre e o gráfico, o mapa, o filtro e a carteira seguem operando.
