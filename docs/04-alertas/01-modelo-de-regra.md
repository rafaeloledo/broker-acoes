---
tipo: sub-issue
epica: 04-alertas
ordem: 1
titulo: Persistir regras de alerta
depende_de: []
---

# Persistir regras de alerta

## Contexto

Não há tabela de regra. O único agendamento implícito é o `request_repaint` do tick simulado.

## Escopo

Tipos de regra nesta issue:

- preço acima de um valor
- preço abaixo de um valor
- variação do dia acima de um percentual, quando houver close anterior no mesmo dia ou o candle anterior gravado
- título de notícia contém um texto
- fato relevante novo do ticker

Campos: id, ticker, tipo, parâmetro, ativa, criada em. CRUD no SQLite. Teste de round-trip e de regra inativa.

## Fora de escopo

Avaliar a regra, desenhar a lista e notificar o Windows. A tela de criação entra na sub-issue do histórico, junto da lista, para não existir formulário sem lugar onde o disparo aparece. Esta issue é só o modelo e o repositório.

## Critérios de aceite

- Uma regra ativa e uma inativa do mesmo ticker são gravadas e relidas.
- Tipo desconhecido não é inserido.
