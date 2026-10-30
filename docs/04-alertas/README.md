---
tipo: epica
id: 04-alertas
ordem: 4
titulo: Alertas locais
depende_de:
  - 01-dado-confiavel
---

# Alertas locais

## Objetivo

O usuário define uma regra sobre preço, variação ou notícia da watchlist. O app registra o disparo e mostra o histórico. A notificação do sistema operacional vem por último, depois da lista interna.

## Por que espera o dado

Regra de preço em cima de `simulate_tick` dispararia o dia inteiro. A avaliação usa o close gravado em `quotes` e, para texto, a tabela `news`.

## Definição de pronto

- Dá para criar, desligar e apagar regra por ticker.
- O ciclo que grava cotação também avalia regras de preço e de variação.
- O ciclo que grava notícia avalia regra de texto e de fato relevante.
- O disparo fica em SQLite, uma vez por evento, e aparece numa lista.
- A notificação do sistema operacional é issue separada e não bloqueia o resto.

## Sub-issues

| Ordem | Arquivo | Título |
| --- | --- | --- |
| 1 | [01-modelo-de-regra.md](01-modelo-de-regra.md) | Persistir regras de alerta |
| 2 | [02-avaliar-no-ciclo-de-dados.md](02-avaliar-no-ciclo-de-dados.md) | Avaliar regras no ciclo de cotação e de notícia |
| 3 | [03-historico-e-lista.md](03-historico-e-lista.md) | Mostrar o histórico de disparos |
| 4 | [04-notificacao-do-sistema.md](04-notificacao-do-sistema.md) | Entregar o disparo na notificação do sistema |

## Fora desta épica

Enviar e-mail, Telegram ou webhook. Explicar o disparo com IA. Isso é sub-issue da épica 05.
