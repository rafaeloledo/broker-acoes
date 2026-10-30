---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 7
titulo: Memorando do ativo na visão Decisão
depende_de:
  - 02-tela-de-decisao/02-visao-decisao
  - 03-noticias/04-contrato-de-leitura
  - 05-ia-provedores/01-contrato-llm
  - 05-ia-provedores/02-ajustes-e-chave
---

# Memorando do ativo na visão Decisão

## Contexto

A visão Decisão já mostra preço e o primeiro conjunto de indicadores. O memorando é um texto curto para decidir se o papel merece leitura, não uma recomendação de compra ou venda.

## Escopo

- Ação na Decisão do ativo focado.
- O pedido leva ticker, preço, horário da coleta, indicadores disponíveis e as notícias recentes daquele ticker pelo contrato de leitura.
- Indicador indisponível entra como ausente, não como zero.
- A instrução manda descrever o que os números e as notícias mostram, sem preço-alvo inventado e sem ordem de operar.
- A tela guarda o último memorando da sessão, com a hora em que foi gerado.

## Fora de escopo

Histórico permanente de memorandos. Pode ser issue futura.

## Critérios de aceite

- O fake recebe os indicadores calculados e não recebe campos de `default_for` que a tela marcou como indisponíveis.
- Sem notícia, o memorando ainda sai, e a tela diz que não havia item na base.
