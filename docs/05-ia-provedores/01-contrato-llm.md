---
tipo: sub-issue
epica: 05-ia-provedores
ordem: 1
titulo: Definir o contrato LlmProvider
depende_de: []
---

# Definir o contrato LlmProvider

## Contexto

As três frentes precisam ser intercambiáveis. O app pede uma conclusão. O adaptador fala o HTTP do provedor.

## Escopo

- Crate `b3-ai`.
- Pedido: instrução do sistema, mensagens do usuário, modelo, limite de tokens de saída.
- Resposta: texto e vetor de citações. Cada citação é uma URL que o chamador afirmou ter enviado.
- Trait assíncrona `complete`.
- Erros distinguíveis: rede, autenticação, limite, resposta malformada.
- Nenhum adaptador concreto nesta issue. Um fake em memória para os testes das issues de briefing, memorando e alerta.

## Fora de escopo

Tela, arquivo de chave e os três HTTP.

## Critérios de aceite

- `b3-app` pode depender de `b3-ai` sem puxar o formato de nenhum provedor.
- O fake devolve texto e citações fixas num teste.
