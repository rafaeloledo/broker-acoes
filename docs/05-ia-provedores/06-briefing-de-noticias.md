---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 6
titulo: Briefing das notícias da watchlist
depende_de:
  - 03-noticias/04-contrato-de-leitura
  - 05-ia-provedores/01-contrato-llm
  - 05-ia-provedores/02-ajustes-e-chave
---

# Briefing das notícias da watchlist

## Contexto

O contrato de leitura da épica 03 já devolve notícias truncadas, com URL. Esta issue monta o pedido e mostra a resposta.

## Escopo

- Ação na visão Notícias: gerar briefing dos tickers filtrados, ou da watchlist inteira se o filtro estiver vazio.
- O pedido inclui só o que a função de leitura devolveu.
- A instrução pede um resumo por ticker e proíbe citar fato que não esteja nos itens.
- A tela mostra o texto e, ao lado, as URLs que estavam no pedido e foram mencionadas.
- URL ausente do pedido não vira link.
- Sem chave configurada, a ação explica que os ajustes estão vazios e não chama a rede.

## Fora de escopo

Memorando e alerta. Agendar briefing sozinho.

## Critérios de aceite

- Com o fake do contrato, três notícias de dois tickers geram um briefing e a tela só linka URLs dessas três.
- Se o texto do fake mencionar uma URL estranha, ela não aparece como link.
