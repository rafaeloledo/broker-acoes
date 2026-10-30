---
tipo: epica
id: 02-tela-de-decisao
ordem: 2
titulo: Tela de decisão e organização da interface
depende_de:
  - 01-dado-confiavel
---

# Tela de decisão e organização da interface

## Objetivo

A primeira tela responde o que mudou no ativo escolhido, em linguagem de investidor. Terminal GPU, mapa e DARF continuam no app, como ferramentas. O código da interface deixa de viver num único arquivo.

## Por que vem depois do dado

A visão Decisão lê preço, horário da coleta e o primeiro conjunto de indicadores. Sem a épica 01, a tela nova só reorganiza o cenário fixo.

## Estado atual

`crates/b3-app/src/main.rs` concentra estado, navegação e as cinco renderizações. `MAX_VISIBLE_ASSETS` é 4. A barra mostra FPS Vulkan e o caminho do banco. As abas se chamam Status Invest, Terminal GPU, Mapa B3, Screener SIMD e Carteira & DARF.

## Definição de pronto

- Cada visão mora no próprio módulo. `main.rs` sobe a janela e delega.
- A abertura cai na visão Decisão.
- Decisão mostra, para o ativo focado: preço de mercado, horário, P/L, P/VP, DY, Graham, Bazin e o que ainda está indisponível.
- Terminal, mapa, screener e carteira seguem acessíveis, fora da barra principal.
- Rótulos da barra descrevem a tarefa, não o mecanismo.

## Sub-issues

| Ordem | Arquivo | Título |
| --- | --- | --- |
| 1 | [01-modularizar-b3-app.md](01-modularizar-b3-app.md) | Extrair as visões de main.rs sem mudar comportamento |
| 2 | [02-visao-decisao.md](02-visao-decisao.md) | Criar a visão Decisão como entrada do app |
| 3 | [03-ferramentas-secundarias.md](03-ferramentas-secundarias.md) | Mover terminal, mapa, screener e carteira para ferramentas |
| 4 | [04-linguagem-da-interface.md](04-linguagem-da-interface.md) | Trocar rótulos internos pela tarefa do usuário |

## Fora desta épica

Layout de notícias, lista de alertas e tela de chave de IA. Essas épicas encaixam módulos novos no lugar deixado pela sub-issue 1.
