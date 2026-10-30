---
tipo: sub-issue
epica: 01-dado-confiavel
ordem: 1
titulo: Ligar a cotação Brapi na watchlist e gravar em quotes
depende_de: []
---

# Ligar a cotação Brapi na watchlist e gravar em quotes

## Contexto

`LiveFeedClient::fetch_brapi_quote` em `crates/b3-ingest/src/live.rs` busca `https://brapi.dev/api/quote/{ticker}` e devolve um `Candle`. Nenhum código de `b3-app` chama esse método. `Database::insert_quotes` e `get_quotes` já existem em `crates/b3-storage/src/lib.rs`.

## Escopo

- Consultar só os tickers de `user_watchlist`.
- Fora do pregão, uma consulta na abertura. No pregão, intervalo de alguns minutos, não a cada frame.
- Gravar em `quotes` uma linha por ticker por dia de pregão. A chave é `(symbol, dia)`. Consulta seguinte no mesmo dia faz `UPDATE` dessa linha.
- Não escrever quando close e volume forem iguais aos já gravados naquele dia.
- Manter no máximo 180 linhas diárias por ticker. A poda roda no máximo uma vez por dia, não a cada consulta.
- Parar de inserir a série sintética de 180 candles que `main.rs` gera quando `get_quotes` volta vazio. Essa série, se ainda servir ao gráfico, fica só em memória.
- Preencher `current_prices` e o preço visível na ficha com o último close gravado.
- Se a chamada falhar, manter o último candle do banco e registrar o erro no log.
- Teste sem rede: duas respostas iguais no mesmo dia geram uma linha; uma resposta com close novo atualiza a mesma linha; o 181º dia remove o mais antigo.

## Fora de escopo

Book, tape, heatmap e o checkbox de feed ao vivo. Nenhum tick simulado entra em `quotes`. Indicadores fundamentalistas. Chave de API da Brapi além de ler uma variável de ambiente se a chamada passar a exigi-la. Candle intradiário.

## Critérios de aceite

- Com rede, PETR4 da watchlist muda de preço quando a Brapi devolve outro valor, e o valor sobrevive ao reabrir o app.
- Várias consultas no mesmo dia, com o mesmo close e volume, deixam uma única linha daquele ticker.
- Sem rede, a tela continua no último close de `quotes`.
- Abrir o app com banco vazio não cria 180 candles sintéticos em `quotes`.
- `fetch_brapi_quote` deixa de ser código morto.
