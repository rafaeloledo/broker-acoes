---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 3
titulo: Adaptador OpenAI
depende_de:
  - 05-ia-provedores/01-contrato-llm
  - 05-ia-provedores/02-ajustes-e-chave
---

# Adaptador OpenAI

## Contexto

Primeiro HTTP real do contrato. Os outros dois adaptadores seguem o mesmo desenho de teste.

## Escopo

- Implementar `LlmProvider` contra o endpoint de chat completions da OpenAI.
- Autenticação Bearer com a chave dos ajustes.
- Mapear o corpo de sucesso para texto. Se o provedor não devolve citações estruturadas, as citações da resposta ficam vazias e o chamador só exibe URL que ele mesmo inseriu no pedido e que o texto repetiu.
- Teste com servidor HTTP local ou resposta gravada: 200, 401 e corpo incompleto.

## Fora de escopo

Streaming, ferramentas e imagens.

## Critérios de aceite

- Um pedido do fake de produção, apontando para o servidor de teste, preenche `complete` com o texto do JSON.
- 401 vira erro de autenticação, sem a chave na mensagem.
