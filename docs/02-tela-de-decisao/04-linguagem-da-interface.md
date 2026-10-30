---
tipo: sub-issue
epica: 02-tela-de-decisao
ordem: 4
titulo: Trocar rótulos internos pela tarefa do usuário
depende_de:
  - 02-tela-de-decisao/03-ferramentas-secundarias
---

# Trocar rótulos internos pela tarefa do usuário

## Contexto

A barra diz "B3 HARDWARE TERMINAL", "Screener SIMD" e mostra "Vulkan RADV" com FPS. Isso descreve a implementação. Quem usa o app para decidir precisa do nome da tarefa e do estado do dado.

## Escopo

- Renomear as entradas visíveis: Decisão, Terminal, Mapa, Busca de ativos, Carteira.
- Tirar FPS e o nome da GPU da barra principal. Se a telemetria ficar, fica dentro do Terminal.
- Onde a épica 01 já distingue mercado e simulação, repetir esse estado em uma linha curta na Decisão.
- Não renomear crates, tipos Rust nem tabelas SQLite.

## Fora de escopo

Tema novo, ícones e revisão de todas as strings de métricas fundamentalistas.

## Critérios de aceite

- A barra principal não menciona SIMD, Vulkan, RADV nem o caminho do arquivo de banco.
- Os nomes das telas cabem na tarefa: decidir, ver o gráfico, ver o mapa, filtrar, ver a carteira.
