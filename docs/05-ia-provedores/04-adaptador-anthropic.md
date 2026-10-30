---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 4
titulo: Adaptador Anthropic
depende_de:
  - 05-ia-provedores/01-contrato-llm
  - 05-ia-provedores/02-ajustes-e-chave
---

# Adaptador Anthropic

## Contexto

A Anthropic não usa o corpo de chat completions. O adaptador traduz o mesmo pedido do contrato para o endpoint de messages.

## Escopo

- `x-api-key` e o cabeçalho de versão exigido pela API.
- Instrução do sistema no campo próprio, mensagens do usuário no array de messages.
- Mesma regra de citação do adaptador OpenAI.
- Testes de 200, 401 e corpo sem bloco de texto.

## Fora de escopo

Streaming e prompt caching.

## Critérios de aceite

- O mesmo pedido usado no teste da OpenAI, com JSON de messages, produz o mesmo formato de resposta do contrato.
- A chave não aparece em erro de autenticação.
