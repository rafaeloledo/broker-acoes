---
tipo: sub-issue
epica: 03-noticias
ordem: 4
titulo: Fixar o contrato de leitura das notícias para a IA
depende_de:
  - 03-noticias/01-modelo-e-tabela-news
---

# Fixar o contrato de leitura das notícias para a IA

## Contexto

A épica 05 vai pedir briefing e explicação de alerta. Os dois precisam receber notícia do mesmo jeito, com URL, para a resposta poder citar a fonte.

## Escopo

- Função de leitura que devolve, para um ou mais tickers, no máximo N notícias recentes, já truncadas a um tamanho máximo de caracteres.
- Cada item entregue ao chamador inclui ticker, título, fonte, data e URL.
- N e o tamanho máximo ficam em constantes nomeadas.
- Teste: a função não devolve notícia de outro ticker e corta texto acima do limite.

## Fora de escopo

Prompt, HTTP de modelo e a frase que o modelo escreve. Isso fica na épica 05.

## Critérios de aceite

- O contrato não importa `reqwest` nem conhece OpenAI, Anthropic ou OpenRouter.
- Um teste com três notícias e limite de uma devolve a mais recente do ticker pedido.
