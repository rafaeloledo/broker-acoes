---
tipo: sub-issue
epica: 04-alertas
ordem: 4
titulo: Entregar o disparo na notificação do sistema
depende_de:
  - 04-alertas/03-historico-e-lista
---

# Entregar o disparo na notificação do sistema

## Contexto

A lista interna já é o registro. Esta issue só avisa o Windows quando um disparo novo entra, para o app poder ficar em segundo plano.

## Escopo

- Ao inserir um disparo ainda não lido, emitir uma notificação do sistema com ticker e resumo.
- Clicar na notificação foca a janela na visão Alertas, naquele ticker.
- Preferência em ajustes para ligar ou desligar o aviso do sistema, ligada por padrão depois que a lista já funciona.
- Se a API de notificação falhar, o disparo continua na lista.

## Fora de escopo

Som, e-mail e aviso com a janela em foco repetindo o que a lista já mostra em tempo real. Com a janela em foco, a lista basta.

## Critérios de aceite

- Com a janela fora de foco, um disparo de teste gera uma notificação.
- Com a preferência desligada, o mesmo disparo só aparece na lista.
