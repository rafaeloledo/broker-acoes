---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 5
titulo: Adaptador OpenRouter
depende_de:
  - 05-ia-provedores/03-adaptador-openai
---

# Adaptador OpenRouter

## Contexto

O OpenRouter fala chat completions com host e catálogo de modelos próprios. O adaptador reaproveita o parse do adaptador OpenAI e troca a base da URL e o nome do modelo, que pode ser prefixado (`openai/...`, `anthropic/...`).

## Escopo

- Base `https://openrouter.ai/api/v1` e Bearer com a chave gravada para este provedor.
- Modelo enviado exatamente como o usuário digitou nos ajustes.
- Teste de 200 com o mesmo formato de chat completions e um teste de modelo com prefixo de outro laboratório.

## Fora de escopo

Catálogo automático de modelos e roteamento por preço.

## Critérios de aceite

- Um modelo `anthropic/claude-...` no campo de ajustes sai no JSON no mesmo texto, sem o adaptador reescrever o id.
- Falha de autenticação segue o erro do contrato, sem a chave na mensagem.
