---
tipo: epica
id: 01-dado-confiavel
ordem: 1
titulo: Dado confiável para a decisão
depende_de: []
---

# Dado confiável para a decisão

## Objetivo

A ficha e a watchlist passam a mostrar preço de mercado e, em seguida, fundamento calculado a partir da CVM. O que continuar sintético fica rotulado como simulação.

## Mudança visual

As cinco abas, a barra, as cores e o limite de quatro ativos permanecem. A visão Decisão, o reagrupamento das ferramentas e a troca de nomes como "Screener SIMD" ficam na épica 02.

Nesta etapa, o que muda na tela:

- Em Status Invest, o preço passa a ser o da Brapi, com o horário da última coleta e o rótulo de mercado.
- P/L, P/VP, dividend yield, Graham e Bazin continuam no mesmo card. Enquanto a CVM não entra, levam o rótulo de cenário local. Quando o período existe, mostram o valor calculado. Conta ausente aparece como indisponível.
- No Terminal GPU, o checkbox deixa de se chamar "Feed Real-Time (Ticks ao Vivo)". Book, tape e o pulso do mapa ganham o texto de simulação. O gráfico em si continua no lugar.

## Por que é a primeira

`StockIndicators::default_for` em `crates/b3-core/src/indicators.rs` fixa preço, P/L, DY e os demais campos de um punhado de tickers. `simulate_tick` anima o terminal. Um parecer, um alerta de preço ou um screener em cima disso descreve o código, não o pregão.

## O que entra no SQLite

A fase 1 grava o mínimo que a ficha precisa reler depois de fechar o app. O resto fica em memória e some com a janela.

| Dado | Gravação |
| --- | --- |
| Último preço da watchlist | Uma linha por ticker por dia de pregão. Atualização no lugar se o dia já existe. Sem escrita se close e volume não mudaram. |
| Histórico diário | Teto de 180 pregões por ticker, que é o que `get_quotes` já pede. Acima disso, a linha mais antiga sai. |
| Contas da CVM | Só os códigos usados no primeiro conjunto de indicadores, do período anual mais recente da watchlist. |
| Book, tape, pulso do mapa, série senoidal | Não gravam. A abertura atual, em `main.rs`, já insere 180 candles gerados. Essa inserção para. |

Fora do pregão o app busca a cotação uma vez na abertura. Durante o pregão, o intervalo é de minutos, nos tickers da watchlist (hoje, no máximo 4). Tirar um ticker da watchlist encerra a coleta dele. As linhas já gravadas permanecem até o teto de 180 dias.

## Definição de pronto

- Os ativos da watchlist exibem o último preço obtido por `fetch_brapi_quote`.
- Esse preço fica em `quotes`, uma linha por ticker por dia, sem linha nova a cada consulta.
- A série sintética de 180 candles deixa de ser inserida no banco.
- Book, tape e variação sintética do heatmap só aparecem com o rótulo de simulação e não passam pelo SQLite.
- A watchlist tem o período anual mais recente da CVM, restrito às contas do primeiro conjunto de indicadores.
- Um primeiro conjunto de indicadores deixa de usar `default_for` quando o período existe.
- Falha de rede não apaga o último valor gravado.

## Sub-issues

| Ordem | Arquivo | Título |
| --- | --- | --- |
| 1 | [01-cotacao-brapi-na-watchlist.md](01-cotacao-brapi-na-watchlist.md) | Ligar a cotação Brapi na watchlist e gravar em quotes |
| 2 | [02-rotulo-mercado-e-simulacao.md](02-rotulo-mercado-e-simulacao.md) | Rotular dado de mercado e dado simulado |
| 3 | [03-periodos-cvm-da-watchlist.md](03-periodos-cvm-da-watchlist.md) | Persistir períodos CVM dos ativos da watchlist |
| 4 | [04-indicadores-a-partir-da-cvm.md](04-indicadores-a-partir-da-cvm.md) | Calcular o primeiro conjunto de indicadores a partir da CVM |

## Fora desta épica

Notícias, alertas, tela nova de decisão e qualquer chamada de modelo.
