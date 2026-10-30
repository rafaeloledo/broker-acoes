---
tipo: sub-issue
epica: 03-noticias
ordem: 3
titulo: Buscar notícias na interface
depende_de:
  - 02-tela-de-decisao/01-modularizar-b3-app
  - 03-noticias/02-coleta-fatos-relevantes-e-rss
---

# Buscar notícias na interface

## Contexto

A épica 02 deixa um módulo livre para visões novas. A busca é a primeira tela que consome a tabela `news`.

## Escopo

- Visão Notícias, no grupo de ferramentas ou como seção da Decisão do ativo focado. A seção da Decisão mostra as mais recentes daquele ticker. A visão completa busca.
- Filtros: ticker da watchlist, texto, período.
- Cada linha mostra título, fonte, data e abre a URL no navegador.
- Estado vazio quando a watchlist ainda não tem coleta, com a data da última tentativa.

## Fora de escopo

Resumo de IA e alerta. Não embutir o artigo dentro da janela.

## Critérios de aceite

- Filtrar por PETR4 esconde notícia de outro ticker.
- O clique na linha abre a URL gravada.
- Lista vazia explica que ainda não houve coleta, em vez de parecer erro de renderização.
