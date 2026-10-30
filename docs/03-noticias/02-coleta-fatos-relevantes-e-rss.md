---
tipo: sub-issue
epica: 03-noticias
ordem: 2
titulo: Coletar fatos relevantes da CVM e RSS da watchlist
depende_de:
  - 03-noticias/01-modelo-e-tabela-news
---

# Coletar fatos relevantes da CVM e RSS da watchlist

## Contexto

A watchlist já diz quais tickers importam. A carga de notícia usa essa lista, não o universo inteiro da B3.

## Escopo

- Coletor de fatos relevantes da CVM associados às companhias já mapeadas na épica 01.
- Um coletor RSS, com a lista de feeds em configuração local, filtrando item cujo título ou corpo mencione o ticker da watchlist.
- Gravar pelo repositório da sub-issue anterior.
- Agendar a coleta no mesmo processo do app, em intervalo longo, separado do intervalo da cotação.
- Testes com XML e HTML/CSV mínimos, sem rede.

## Fora de escopo

Paywall, login em portal e scraping de página autenticada. Vários veículos além do primeiro RSS. Isso pode ganhar issue própria quando o primeiro feed estiver estável.

## Critérios de aceite

- Uma carga de fixture cria notícias da watchlist e ignora item de ticker de fora dela.
- Rodar de novo não duplica URL.
- Falha de uma fonte não impede a outra.
