---
tipo: sub-issue
epica: 02-tela-de-decisao
ordem: 1
titulo: Extrair as visões de main.rs sem mudar comportamento
depende_de: []
---

# Extrair as visões de main.rs sem mudar comportamento

## Contexto

Notícias, alertas e ajustes precisam de módulo próprio. Encaixá-los em `main.rs` aumenta o arquivo que já renderiza as cinco telas, o modal de busca e o ciclo de tick.

## Escopo

- Mover cada `render_*_view` para um módulo em `crates/b3-app/src/`, com o estado que a visão usa.
- Manter `AppState` como dono dos dados até uma épica precisar fatiá-lo.
- Nenhuma mudança de texto, ordem de aba ou regra visual nesta issue.
- O binário continua `cargo run --release -p b3-app`.

## Fora de escopo

Visão Decisão, renomear abas, alterar o limite de 4 ativos.

## Critérios de aceite

- `main.rs` inicia o eframe, monta a barra e despacha a visão ativa.
- As cinco telas atuais abrem com o mesmo conteúdo de antes da extração.
- `cargo test --workspace` segue verde.
