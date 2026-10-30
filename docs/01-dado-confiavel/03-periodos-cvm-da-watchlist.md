---
tipo: sub-issue
epica: 01-dado-confiavel
ordem: 3
titulo: Persistir períodos CVM dos ativos da watchlist
depende_de: []
---

# Persistir períodos CVM dos ativos da watchlist

## Contexto

`CvmIngestor` em `crates/b3-ingest/src/cvm.rs` interpreta uma linha CSV de demonstração e monta um `FinancialPeriod`. Não há download, não há tabela de períodos e não há vínculo entre ticker e companhia.

## Escopo

- Resolver ticker para a companhia (CNPJ ou código CVM) com uma tabela local versionada no repositório, começando pelos ativos que `default_for` já conhece.
- Ler o conjunto aberto de demonstrações anuais da CVM em fluxo. Descartar na leitura qualquer companhia fora da watchlist e qualquer conta fora da lista fixa do primeiro conjunto (lucro, patrimônio, dividendos e quantidade de ações, ou os códigos equivalentes).
- Persistir só o período anual mais recente de cada ticker da watchlist: ano, tipo, código da conta e valor. Chave única `(ticker, ano, tipo, código)`.
- Não gravar o ZIP, o CSV nem o plano de contas inteiro.
- Pular o download quando esse ano já estiver no banco. Baixar de novo só se a data de referência publicada for mais nova que a gravada.
- Tirar o ticker da watchlist encerra a carga dele. As poucas contas já gravadas ficam, sem crescimento.
- Teste com um CSV mínimo, no formato que `parse_cvm_csv_line` já aceita: de várias companhias e contas, só as da watchlist e da lista fixa permanecem, uma vez.

## Fora de escopo

Calcular P/L, ROE ou Graham. Isso é a sub-issue seguinte. Cobrir o universo inteiro da B3. Guardar ITR trimestral, histórico de vários anos e séries brutas.

## Critérios de aceite

- Para um ticker da watchlist com mapeamento, o banco guarda o ano mais recente e somente as contas da lista fixa.
- Rodar a carga de novo, com a mesma data de referência, não baixa de novo e não duplica conta.
- Uma conta fora da lista fixa, presente no CSV de teste, não aparece no SQLite.
- Ticker sem mapeamento fica sem período e não quebra a abertura do app.
