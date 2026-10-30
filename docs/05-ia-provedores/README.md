---
tipo: epica
id: 05-ia-provedores
ordem: 5
titulo: IA em OpenAI, Anthropic e OpenRouter
depende_de:
  - 01-dado-confiavel
---

# IA em OpenAI, Anthropic e OpenRouter

## Objetivo

Um contrato de conclusão de texto, três adaptadores e uma tela de ajustes. O produto usa esse cliente em três leituras: briefing das notícias, memorando do ativo e explicação do alerta. As três leituras citam a fonte que receberam. Nenhuma inventa manchete nem preço.

## Contrato

`b3-ai` expõe um pedido e uma resposta. O pedido leva instrução, trechos já carregados (indicadores, notícias com URL, disparo) e o nome do modelo. A resposta leva o texto e a lista de URLs citadas, restrita às URLs que entraram no pedido.

Os adaptadores:

| Provedor | Formato |
| --- | --- |
| OpenAI | Chat completions, `Authorization: Bearer` |
| Anthropic | Messages, `x-api-key` e versão da API |
| OpenRouter | Chat completions compatível com OpenAI, outro host e outro catálogo de modelos |

A interface escolhe um provedor e um modelo. O restante do app não importa o cliente HTTP do provedor.

## Chave

A chave fica em arquivo de configuração do usuário, fora do repositório e fora de `broker.db`. O log não imprime a chave nem o cabeçalho `Authorization`. O `.gitignore` cobre o arquivo se alguém o criar dentro da árvore do projeto.

## Definição de pronto

- Os três adaptadores passam num teste com resposta HTTP gravada.
- Ajustes salvam provedor e modelo e recusam salvar com chave vazia.
- Briefing, memorando e explicação de alerta chamam o mesmo contrato.
- Se a URL citada não estava no pedido, a interface não a mostra.

## Sub-issues

| Ordem | Arquivo | Título |
| --- | --- | --- |
| 1 | [01-contrato-llm.md](01-contrato-llm.md) | Definir o contrato LlmProvider |
| 2 | [02-ajustes-e-chave.md](02-ajustes-e-chave.md) | Guardar provedor, modelo e chave fora do repositório |
| 3 | [03-adaptador-openai.md](03-adaptador-openai.md) | Adaptador OpenAI |
| 4 | [04-adaptador-anthropic.md](04-adaptador-anthropic.md) | Adaptador Anthropic |
| 5 | [05-adaptador-openrouter.md](05-adaptador-openrouter.md) | Adaptador OpenRouter |
| 6 | [06-briefing-de-noticias.md](06-briefing-de-noticias.md) | Briefing das notícias da watchlist |
| 7 | [07-memorando-do-ativo.md](07-memorando-do-ativo.md) | Memorando do ativo na visão Decisão |
| 8 | [08-explicacao-do-alerta.md](08-explicacao-do-alerta.md) | Explicar um disparo de alerta com as notícias do ticker |

## Fora desta épica

Treino de modelo, embedding, agente com ferramenta livre e envio da base inteira do SQLite no prompt. O pedido só leva o recorte montado pelas issues 6, 7 e 8.
