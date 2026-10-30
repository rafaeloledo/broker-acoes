---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 8
titulo: Explicar um disparo de alerta com as notícias do ticker
depende_de:
  - 04-alertas/03-historico-e-lista
  - 03-noticias/04-contrato-de-leitura
  - 05-ia-provedores/01-contrato-llm
  - 05-ia-provedores/02-ajustes-e-chave
---

# Explicar um disparo de alerta com as notícias do ticker

## Contexto

O histórico de alertas mostra o que cruzou a regra. Esta issue acrescenta, em cada disparo, uma leitura das notícias daquele ticker no mesmo período.

## Escopo

- Ação em cada linha de disparo.
- O pedido leva a regra, o resumo do disparo (preço ou título) e as notícias do ticker desde um intervalo curto antes do disparo, pelo contrato de leitura.
- A instrução relaciona o disparo aos itens recebidos e diz quando nenhum item explica o movimento.
- A resposta fica colada ao disparo na sessão. Não reescreve o registro do disparo.

## Fora de escopo

Disparar a explicação sozinha a cada alerta. O usuário pede.

## Critérios de aceite

- Um disparo de preço com duas notícias do ticker mostra as duas URLs se o texto as mencionar.
- Um disparo sem notícia no intervalo produz a explicação de ausência, sem chamada inventando manchete. O teste observa o pedido: a lista de notícias vai vazia.
