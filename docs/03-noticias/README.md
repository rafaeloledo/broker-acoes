---
tipo: epica
id: 03-noticias
ordem: 3
titulo: Busca de notícias por ativo
depende_de:
  - 01-dado-confiavel
---

# Busca de notícias por ativo

## Objetivo

O usuário busca o que foi publicado sobre os tickers da watchlist. Cada item tem fonte, data e URL. A IA, na épica 05, só comenta notícia que estiver nesta base.

## Por que é uma épica separada

Não existe entidade, tabela nem tela de notícia. O ingest cobre COTAHIST, linha CVM, SINACOR e cotação. Misturar a coleta com o cliente de modelo acoplaria a fonte ao provedor.

## Definição de pronto

- SQLite guarda notícia com ticker, título, URL, fonte, data de publicação e resumo quando a fonte trouxer.
- A carga traz fatos relevantes da CVM e pelo menos um RSS, filtrados pela watchlist.
- A visão Notícias busca por ticker, texto e período.
- Item sem URL não entra.
- A mesma URL não duplica.

## Sub-issues

| Ordem | Arquivo | Título |
| --- | --- | --- |
| 1 | [01-modelo-e-tabela-news.md](01-modelo-e-tabela-news.md) | Criar o modelo e a tabela de notícias |
| 2 | [02-coleta-fatos-relevantes-e-rss.md](02-coleta-fatos-relevantes-e-rss.md) | Coletar fatos relevantes da CVM e RSS da watchlist |
| 3 | [03-busca-na-interface.md](03-busca-na-interface.md) | Buscar notícias na interface |
| 4 | [04-contrato-de-leitura.md](04-contrato-de-leitura.md) | Fixar o contrato de leitura das notícias para a IA |

## Fora desta épica

Resumo gerado por modelo, alerta de palavra-chave e navegador interno. O clique abre a URL no navegador do sistema.
