---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 2
titulo: Guardar provedor, modelo e chave fora do repositório
depende_de:
  - 02-tela-de-decisao/01-modularizar-b3-app
  - 05-ia-provedores/01-contrato-llm
---

# Guardar provedor, modelo e chave fora do repositório

## Contexto

Sem um lugar para a chave, cada adaptador inventaria o próprio arquivo. A visão de ajustes entra no espaço aberto pela modularização de `b3-app`.

## Escopo

- Tela Ajustes: provedor (OpenAI, Anthropic, OpenRouter), modelo em texto livre e campo de chave.
- Gravar em arquivo no diretório de configuração do usuário. Não gravar em `broker.db`.
- Se o arquivo for criado dentro do repo por engano, o `.gitignore` já o ignora.
- A chave não volta para o campo em texto claro depois de salva. A tela mostra só se existe chave.
- Trocar de provedor não reaproveita a chave de outro.
- Log e mensagem de erro não incluem a chave.

## Fora de escopo

Listar modelos pela API de cada provedor. O campo de modelo é texto informado pelo usuário nesta issue.

## Critérios de aceite

- Salvar e reabrir o app mantém provedor e modelo.
- Busca no repositório e no `broker.db` não encontra a chave de teste.
- Salvar com chave vazia na primeira vez deixa o provedor desligado, sem arquivo parcial.
