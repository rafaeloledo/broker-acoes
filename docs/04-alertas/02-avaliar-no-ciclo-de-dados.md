---
tipo: sub-issue
epica: 04-alertas
ordem: 2
titulo: Avaliar regras no ciclo de cotação e de notícia
depende_de:
  - 01-dado-confiavel/01-cotacao-brapi-na-watchlist
  - 03-noticias/01-modelo-e-tabela-news
  - 04-alertas/01-modelo-de-regra
---

# Avaliar regras no ciclo de cotação e de notícia

## Contexto

A cotação e a notícia passam a ter ciclo próprio nas épicas 01 e 03. O avaliador entra nesses ciclos, não num timer paralelo com outra cópia do preço.

## Escopo

- Depois de gravar quotes, avaliar regras de preço e de variação dos tickers atualizados.
- Depois de gravar news, avaliar regras de texto e de fato relevante.
- Gravar o disparo com regra, ticker, momento e um resumo curto (preço observado ou título da notícia).
- Não disparar de novo o mesmo fato: a mesma regra com o mesmo preço-alvo permanece disparada até o preço voltar para o lado seguro, ou até a notícia ser outra URL.
- Regra inativa não dispara.
- Testes com banco em memória, sem rede.

## Fora de escopo

Widget e notificação do sistema.

## Critérios de aceite

- Preço que cruza o limite gera um disparo. A coleta seguinte, ainda do mesmo lado, não gera outro.
- Notícia cuja URL já disparou aquela regra não dispara de novo.
- Regra inativa com condição verdadeira não gera linha de disparo.
