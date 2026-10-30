---
tipo: sub-issue
epica: 03-noticias
ordem: 1
titulo: Criar o modelo e a tabela de notícias
depende_de: []
---

# Criar o modelo e a tabela de notícias

## Contexto

`b3-storage` cria `quotes`, `trades` e `user_watchlist`. Não há onde pendurar uma manchete nem um fato relevante.

## Escopo

- Tipo de domínio em `b3-core`: ticker, título, url, fonte, `publicado_em`, resumo opcional, identificador externo da fonte.
- Tabela `news` com unicidade por URL.
- Inserção em lote e consulta por ticker, intervalo de datas e texto no título.
- Teste de inserção duplicada e de consulta por ticker.

## Fora de escopo

HTTP, parser de RSS e qualquer widget.

## Critérios de aceite

- Duas inserções da mesma URL resultam em uma linha.
- A consulta por ticker devolve só as notícias daquele papel, da mais nova para a mais antiga.
