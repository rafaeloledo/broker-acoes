import React, { useEffect, useRef, useState } from "react";

export interface IndicatorAnalyticalInfo {
  title: string;
  formula: string;
  summary: string;
  analysis: string;
  benchmark?: string;
}

export const INDICATOR_GUIDE: Record<string, IndicatorAnalyticalInfo> = {
  "Preço Atual": {
    title: "Cotação Atual de Mercado",
    formula: "Último negócio fechado no book da B3",
    summary:
      "Preço pelo qual uma única ação ou cota de FII está sendo negociada no mercado secundário.",
    analysis:
      "Sozinho, o preço nominal não diz se um ativo está caro ou barato (uma ação de R$ 5 pode estar cara e uma de R$ 80 pode estar barata). Deve sempre ser cruzado com o valor intrínseco (VPA, LPA, Graham Justo e Teto Bazin) para avaliar a margem de segurança.",
    benchmark: "Comparar c/ Graham e VPA",
  },
  "Dividend Yield": {
    title: "Dividend Yield Anualizado (DY 12M)",
    formula: "(Proventos pagos nos últimos 12 meses ÷ Preço Atual) × 100",
    summary:
      "Mede o retorno percentual em dinheiro (dividendos, JCP ou rendimentos de FII) recebido pelo investidor em relação ao preço atual da cota/ação.",
    analysis:
      "Acima de 6% a.a. é considerado atrativo pela metodologia de Décio Bazin. Porém, atenção a 'armadilhas de dividendo' (DY artificialmente alto > 18% causado por queda severa na cotação ou venda não recorrente de ativos). Em FIIs de papel, avalie se o DY reflete apenas correção monetária (inflação) ou ganho real sustentável.",
    benchmark: "Referência: > 6% a.a. (Ações) | 8%–13% (FIIs)",
  },
  "DY 12M": {
    title: "Dividend Yield Acumulado em 12 Meses",
    formula: "(Proventos 12M ÷ Cotação Atual) × 100",
    summary:
      "Taxa de retorno em proventos isentos (no caso de FIIs e dividendos) distribuídos nos últimos 12 meses sobre o preço atual.",
    analysis:
      "Em FIIs de tijolo, um DY entre 8% e 11% a.a. costuma indicar distribuição orgânica de aluguéis reais (reajustados por IPCA/IGP-M). Em FIIs de papel (CRI), o DY costuma ser nominalmente maior (11% a 15%) porque embute a amortização da inflação/juros da carteira — parte desse rendimento deve ser reinvestida para preservar o poder de compra.",
    benchmark: "Tijolo: 8%–11% | Papel: 11%–14%",
  },
  "P/L": {
    title: "Preço sobre Lucro (Price to Earnings)",
    formula: "Preço da Ação ÷ Lucro por Ação (LPA 12M)",
    summary:
      "Indica quantos anos seriam necessários para reaver o capital investido apenas com os lucros atuais da empresa (assumindo lucro constante e 100% de repasse).",
    analysis:
      "Quanto menor o P/L, mais barata a empresa parece em relação à sua geração de lucro. P/L entre 5x e 12x sugere desconto ou maturidade; acima de 20x indica que o mercado precifica forte crescimento futuro. Cuidado com P/L distorcido por lucros extraordinários (não recorrentes) ou empresas cíclicas no topo do ciclo de commodities (onde o P/L parece baixo justamente antes da reversão). Não é o múltiplo principal para FIIs (prefira P/VP e DY).",
    benchmark: "Desconto: < 8x | Média B3: 8x–14x",
  },
  "P/VP": {
    title: "Preço sobre Valor Patrimonial (Price to Book)",
    formula: "Preço Atual ÷ Valor Patrimonial por Ação/Cota (VPA)",
    summary:
      "Compara o valor de mercado do ativo com o seu patrimônio líquido contábil (ativos menos dívidas/passivos).",
    analysis:
      "• P/VP < 1,00: O ativo negocia com DESCONTO em relação ao valor contábil de seus bens/caixa.\n• P/VP = 1,00: Negocia exatamente pelo valor patrimonial justo.\n• P/VP > 1,00: Negocia com ÁGIO.\nEm FIIs, é o termômetro central de valuation: comprar bons FIIs abaixo de 0,95x aumenta o yield efetivo e cria ganho de capital potencial. Em Ações, empresas de alto ROE (como WEG ou seguradoras) justificam P/VP elevado porque geram muito retorno com pouco capital físico.",
    benchmark: "Desconto: < 0,95x | Justo FII: ~1,00x",
  },
  "EV/EBITDA": {
    title: "Enterprise Value sobre EBITDA",
    formula: "(Valor de Mercado + Dívida Líquida) ÷ EBITDA 12M",
    summary:
      "Avalia o valor total da firma (incluindo a dívida que um comprador assumiria) frente à sua geração operacional bruta de caixa (EBITDA).",
    analysis:
      "É superior ao P/L para comparar empresas com diferentes níveis de endividamento ou regimes tributários, pois penaliza companhias muito alavancadas (que têm P/L aparentemente baixo, mas EV alto por conta da dívida). Valores abaixo de 6x costumam indicar forte subavaliação operacional; acima de 12x exigem alto crescimento para se justificarem.",
    benchmark: "Atrativo: 4x–7x | Alerta: > 12x",
  },
  "EV/EBIT": {
    title: "Enterprise Value sobre EBIT (Lucro Operacional)",
    formula: "(Valor de Mercado + Dívida Líquida) ÷ EBIT 12M",
    summary:
      "Mede quantos anos de lucro operacional (já descontando depreciação e amortização) são necessários para pagar o valor total da firma (ações + dívida líquida).",
    analysis:
      "É o múltiplo de valuation utilizado por Joel Greenblatt na 'Fórmula Mágica'. Diferente do EV/EBITDA, o EV/EBIT leva em conta o desgaste físico das máquinas e instalações (depreciação/CAPEX de manutenção), sendo muito mais rigoroso para indústrias pesadas.",
    benchmark: "Fórmula Mágica: < 8x é muito atrativo",
  },
  "P/EBITDA": {
    title: "Preço sobre EBITDA",
    formula: "Preço da Ação ÷ EBITDA por Ação",
    summary:
      "Relaciona apenas o valor de mercado das ações (sem somar a dívida líquida) à geração operacional de caixa antes de juros, impostos, depreciação e amortização.",
    analysis:
      "Útil para medir o preço pago pelo fluxo operacional bruto, mas deve ser sempre confrontado com o EV/EBITDA: se o P/EBITDA for muito baixo (ex: 2x) e o EV/EBITDA for alto (ex: 8x), significa que a maior parte do valor gerado pela operação está comprometida com credores (dívida elevada).",
    benchmark: "Saudável: 3x–8x (comparar c/ EV/EBITDA)",
  },
  "P/EBIT": {
    title: "Preço sobre EBIT",
    formula: "Preço da Ação ÷ EBIT por Ação",
    summary:
      "Compara o preço da ação com o lucro operacional genuíno da atividade-fim da empresa, antes do resultado financeiro e impostos.",
    analysis:
      "Permite enxergar a capacidade de geração de lucro do negócio principal isolando distorções financeiras (como variações cambiais ou despesas de juros). Um P/EBIT baixo aliado a baixa alavancagem indica operação barata e eficiente.",
    benchmark: "Atrativo: 4x–9x",
  },
  "P/SR": {
    title: "Price to Sales Ratio (Preço sobre Receita Líquida)",
    formula: "Valor de Mercado ÷ Receita Líquida Anual (12M)",
    summary:
      "Mostra quanto o mercado paga para cada R$ 1,00 de faturamento líquido gerado pela companhia.",
    analysis:
      "Fundamental para avaliar empresas de varejo, tecnologia ou em fase de turnaround (cujo lucro líquido está temporariamente comprimido, mas a receita continua forte). Deve ser analisado junto à Margem Líquida: varejistas com margens de 2% devem ter P/SR baixo (< 0,8x), enquanto empresas de software/concessões com margens de 30% comportam P/SR > 3x.",
    benchmark: "Varejo: < 1,0x | Alta Margem: 1,5x–4,0x",
  },
  "P/SR (Receita)": {
    title: "Price to Sales Ratio (Preço sobre Receita Líquida)",
    formula: "Valor de Mercado ÷ Receita Líquida Anual (12M)",
    summary:
      "Mostra quanto o mercado paga para cada R$ 1,00 de faturamento líquido gerado pela companhia.",
    analysis:
      "Fundamental para avaliar empresas em turnaround ou setores de margens apertadas. Sempre cruze com a Margem Líquida: P/SR baixo só é barganha se a empresa conseguir converter faturamento em caixa real.",
    benchmark: "Varejo: < 1,0x | Alta Margem: 1,5x–4,0x",
  },
  "P/Ativo": {
    title: "Preço sobre Ativos Totais",
    formula: "Valor de Mercado ÷ Ativo Total",
    summary:
      "Compara o valor de mercado da empresa com a soma total de todos os seus bens, direitos, caixa, estoques e imobilizado.",
    analysis:
      "Um P/Ativo abaixo de 0,50x indica que o mercado avalia a empresa por menos da metade de tudo o que ela possui no balanço. É útil para identificar descontos profundos em setores intensivos em ativos (energia, siderurgia, logística), desde que o passivo (dívida) esteja controlado.",
    benchmark: "Desconto patrimonial: < 0,60x",
  },
  "P/Cap. Giro": {
    title: "Preço sobre Capital de Giro Líquido",
    formula: "Preço da Ação ÷ ((Ativo Circulante − Passivo Circulante) por Ação)",
    summary:
      "Mede quantas vezes a cotação atual representa o capital de giro líquido (folga financeira de curto prazo) da companhia.",
    analysis:
      "Inspirado no Deep Value Investing de Benjamin Graham: quanto menor e positivo for esse múltiplo, maior o colchão de liquidez imediata em relação ao preço da ação. Se for negativo, o passivo de curto prazo supera o ativo circulante (exigindo atenção à Liquidez Corrente).",
    benchmark: "Positivo e baixo (< 6x) indica forte caixa",
  },
  "PEG Ratio": {
    title: "Price/Earnings to Growth (P/L ajustado ao Crescimento)",
    formula: "(P/L) ÷ Taxa de Crescimento Anual dos Lucros (CAGR %)",
    summary:
      "Popularizado por Peter Lynch, pondera se o múltiplo P/L pago pela ação é justificado pela velocidade de crescimento dos seus lucros.",
    analysis:
      "• PEG < 1,00: O crescimento dos lucros supera o P/L — forte indício de ação de crescimento descontada (GARP).\n• PEG ≈ 1,00: Preço justo frente ao crescimento.\n• PEG > 2,00: Ação cara em relação ao ritmo de expansão de resultados.",
    benchmark: "Subavaliado: < 1,00 | Justo: ~1,00",
  },
  "LPA": {
    title: "Lucro por Ação (Earnings Per Share — EPS)",
    formula: "Lucro Líquido Acumulado (12M) ÷ Total de Ações",
    summary:
      "Fatia do lucro líquido anual que pertence a cada ação individual da companhia.",
    analysis:
      "É a base para o cálculo do P/L e do Preço Justo de Graham. Um LPA crescente ano após ano e positivo demonstra criação consistente de valor para o acionista e capacidade real de sustentar dividendos crescentes.",
    benchmark: "Buscar LPA positivo e crescente",
  },
  "VPA": {
    title: "Valor Patrimonial por Ação / Cota (Book Value per Share)",
    formula: "Patrimônio Líquido Contábil ÷ Total de Ações ou Cotas",
    summary:
      "Representa o valor contábil líquido de cada ação ou cota caso todos os ativos fossem liquidados e todas as dívidas quitadas pelo valor de balanço.",
    analysis:
      "Em FIIs de tijolo, o VPA reflete o laudo de avaliação patrimonial anual dos imóveis; em FIIs de papel, reflete a marcação a mercado/curva dos CRIs em carteira. Comparar o Preço Atual com o VPA define o desconto ou ágio patrimonial (P/VP).",
    benchmark: "Cotação < VPA implica desconto (P/VP < 1)",
  },
  "VPA (Patrimonial)": {
    title: "Valor Patrimonial por Cota (VPA)",
    formula: "Patrimônio Líquido do Fundo ÷ Nº Total de Cotas Emitidas",
    summary:
      "Valor contábil real dos ativos imobiliários ou títulos de crédito do fundo líquido de obrigações, dividido por cada cota.",
    analysis:
      "Se a cotação de mercado está abaixo do VPA, você está comprando o metro quadrado dos imóveis ou a carteira de CRIs com desconto em relação ao laudo patrimonial. Evite emissões de novas cotas abaixo do VPA, pois diluem o patrimônio do cotista.",
    benchmark: "Base de referência para o P/VP",
  },
  "DPA": {
    title: "Dividendo / Provento por Ação (12M)",
    formula: "Total de Dividendos e JCP pagos em 12M ÷ Total de Ações",
    summary:
      "Valor absoluto em Reais (R$) efetivamente distribuído por cada ação ou cota ao longo dos últimos 12 meses.",
    analysis:
      "É o insumo direto da fórmula de Décio Bazin (Preço Teto = DPA ÷ 0,06). Ao comparar DPA com LPA (Payout = DPA ÷ LPA), verifica-se a sustentabilidade do dividendo: se DPA > LPA em uma ação, a empresa está distribuindo reservas ou se endividando para pagar proventos.",
    benchmark: "Payout saudável em Ações: 30%–80% do LPA",
  },
  "Graham Justo": {
    title: "Preço Intrínseco de Benjamin Graham",
    formula: "√(22,5 × LPA × VPA)",
    summary:
      "Estimativa clássica de valor intrínseco formulada pelo mentor de Warren Buffett, combinando lucratividade (P/L máximo de 15x) e solidez patrimonial (P/VP máximo de 1,5x → 15 × 1,5 = 22,5).",
    analysis:
      "Quando o Preço Atual está significativamente abaixo do Graham Justo, há 'Margem de Segurança' positiva. É ideal para empresas consolidadas, industriais, elétricas e financeiras com lucros recorrentes. Menos aplicável a startups sem lucro ou empresas asset-light com VPA muito reduzido.",
    benchmark: "Margem de Segurança ideal: > 20%",
  },
  "Preço justo de Graham": {
    title: "Preço Intrínseco de Benjamin Graham",
    formula: "√(22,5 × LPA × VPA)",
    summary:
      "Estimativa clássica de valor intrínseco formulada por Benjamin Graham, combinando lucratividade (P/L até 15x) e proteção patrimonial (P/VP até 1,5x).",
    analysis:
      "Indica o preço máximo que um investidor defensivo deveria pagar. Quanto maior o desconto da cotação atual em relação ao Preço Justo de Graham, maior o colchão de proteção contra erros de projeção ou crises de mercado.",
    benchmark: "Margem de Segurança ideal: > 20%",
  },
  "Bazin Teto (6%)": {
    title: "Preço Teto de Décio Bazin (Piso de 6% a.a.)",
    formula: "DPA (Proventos 12M por Ação/Cota) ÷ 0,06",
    summary:
      "Determina o preço máximo que se pode pagar pelo ativo para garantir um Dividend Yield mínimo de 6% ao ano com base nos proventos distribuídos.",
    analysis:
      "Criado pelo jornalista e investidor brasileiro Décio Bazin no livro 'Faça Fortuna com Ações'. Se o Preço Atual for menor que o Teto Bazin, o ativo remunera acima de 6% a.a. em proventos. Bazin também exigia dívida controlada e recorrência histórica de pagamentos.",
    benchmark: "Comprar abaixo do Teto garante DY > 6%",
  },
  "Teto Bazin (6%)": {
    title: "Preço Teto de Décio Bazin (Piso de 6% a.a.)",
    formula: "Proventos Anuais por Cota/Ação (DPA) ÷ 0,06",
    summary:
      "Cotação limite até a qual o ativo entrega pelo menos 6% ao ano de retorno direto em proventos.",
    analysis:
      "Para FIIs e ações previdenciárias, quanto maior a distância positiva entre o Teto Bazin e o preço atual de tela, maior a folga de geração de renda passiva isenta de IR.",
    benchmark: "Cotação abaixo do Teto = Margem Positiva",
  },
  "Preço teto Bazin": {
    title: "Preço Teto de Décio Bazin",
    formula: "DPA ÷ Taxa de Yield Alvo (padrão 6,0% a.a.)",
    summary:
      "Define a cotação máxima para assegurar a rentabilidade mínima desejada em dividendos.",
    analysis:
      "Você pode ajustar a régua de yield alvo conforme o cenário de juros reais (NTN-B): exigir 6% a 8% cria proteção adicional contra oscilações nos proventos futuros.",
    benchmark: "Piso clássico Bazin: 6,0% a.a.",
  },
  "ROE": {
    title: "Return on Equity (Retorno sobre o Patrimônio Líquido)",
    formula: "(Lucro Líquido 12M ÷ Patrimônio Líquido) × 100",
    summary:
      "Mede a rentabilidade que a administração consegue gerar em cima do capital próprio pertencente aos acionistas.",
    analysis:
      "É o principal indicador de qualidade e vantagem competitiva ('moat') de uma ação. Um ROE consistentemente acima de 15% a.a. (superior à taxa Selic/custo de oportunidade) mostra que a empresa multiplica riqueza internamente. Atenção: dívida excessiva reduz o Patrimônio Líquido e pode inflar artificialmente o ROE — sempre confira o ROIC e a Dív. Líq/EBITDA.",
    benchmark: "Excelente: > 15% | Bancos líderes: 16%–22%",
  },
  "ROIC": {
    title: "Return on Invested Capital (Retorno sobre o Capital Investido)",
    formula: "(NOPLAT / EBIT ajustado de impostos ÷ Capital Total Investido) × 100",
    summary:
      "Mede o retorno operacional gerado sobre TODO o capital alocado no negócio (somando capital dos acionistas + dívida financeira líquida).",
    analysis:
      "Mais completo que o ROE para empresas industriais e de infraestrutura porque não é distorcido pela estrutura de alavancagem financeira. Quando o ROIC supera o custo médio ponderado de capital (WACC), a empresa cria valor econômico real a cada real investido em expansão.",
    benchmark: "Forte geração de valor: > 12% a.a.",
  },
  "ROA": {
    title: "Return on Assets (Retorno sobre os Ativos Totais)",
    formula: "(Lucro Líquido 12M ÷ Ativo Total) × 100",
    summary:
      "Avalia a eficiência global da empresa em extrair lucro líquido a partir de toda a sua base de ativos (próprios e financiados por terceiros).",
    analysis:
      "Um ROA acima de 7% a 10% demonstra alta produtividade dos ativos. Empresas muito endividadas costumam ter ROE alto, mas ROA baixo (pois as despesas financeiras corroem o lucro líquido sobre o ativo total).",
    benchmark: "Saudável: > 6% (indústria) | > 1,5% (bancos)",
  },
  "Margem Bruta": {
    title: "Margem Bruta Operacional",
    formula: "(Lucro Bruto ÷ Receita Líquida) × 100",
    summary:
      "Percentual da receita que sobra imediatamente após pagar os custos diretos de produção ou mercadoria vendida (CPV/CMV).",
    analysis:
      "Mede o poder de precificação ('pricing power') e a força da marca frente a fornecedores e inflação de insumos. Margens brutas altas (> 40%) e estáveis indicam barreiras de entrada e capacidade de repassar custos aos clientes.",
    benchmark: "Alto poder de preço: > 35%",
  },
  "Margem EBITDA": {
    title: "Margem EBITDA",
    formula: "(EBITDA ÷ Receita Líquida) × 100",
    summary:
      "Representa a fatia do faturamento convertida em caixa operacional potencial antes de depreciação, juros e impostos.",
    analysis:
      "Excelente termômetro de eficiência operacional pura e diluição de despesas administrativas/comerciais (SG&A). Concessões de transmissão de energia e saneamento costumam ter Margem EBITDA > 50%, enquanto varejo alimentar opera entre 5% e 9%.",
    benchmark: "Boa eficiência: > 20% (varia por setor)",
  },
  "Margem EBIT": {
    title: "Margem EBIT (Margem Operacional)",
    formula: "(EBIT ÷ Receita Líquida) × 100",
    summary:
      "Percentual da receita que resta após todos os custos operacionais, despesas gerais e a depreciação dos ativos.",
    analysis:
      "Reflete a lucratividade real da operação antes do impacto das dívidas e tributos sobre a renda. Se a Margem EBITDA for alta mas a Margem EBIT for muito baixa, o negócio consome grande parte do caixa apenas para repor ativos depreciados.",
    benchmark: "Saudável: > 15%",
  },
  "Margem Líquida": {
    title: "Margem Líquida Final",
    formula: "(Lucro Líquido ÷ Receita Líquida) × 100",
    summary:
      "Mostra quantos centavos de lucro líquido final sobram no bolso do acionista para cada R$ 1,00 vendido, após absolutamente todos os custos, juros e impostos.",
    analysis:
      "Margem Líquida acima de 10% a 15% oferece forte amortecedor contra crises econômicas e alta da taxa Selic. Margens muito comprimidas (< 3%) tornam a empresa vulnerável a qualquer aumento de custo ou despesa financeira.",
    benchmark: "Forte: > 12% | Defensiva: > 20%",
  },
  "Giro de Ativos": {
    title: "Giro do Ativo Total (Asset Turnover)",
    formula: "Receita Líquida Anual ÷ Ativo Total",
    summary:
      "Mede quantas vezes o valor total dos ativos da empresa 'gira' na forma de receita de vendas ao longo de um ano.",
    analysis:
      "Componente essencial da Análise DuPont: empresas de varejo ganham dinheiro no alto Giro de Ativos (> 1,2x) mesmo com margens baixas, enquanto elétricas e shoppings têm giro mais lento (< 0,5x) compensado por margens elevadas.",
    benchmark: "Varejo: > 1,0x | Infraestrutura: 0,3x–0,6x",
  },
  "CAGR Receitas (5a)": {
    title: "CAGR de Receitas (5 Anos)",
    formula: "((Receita Atual ÷ Receita há 5 anos)^(1/5) − 1) × 100",
    summary:
      "Taxa composta de crescimento anual da receita líquida da companhia nos últimos 5 anos.",
    analysis:
      "Demonstra se a empresa está ganhando participação de mercado e expandindo suas vendas acima da inflação (IPCA). Crescimento de lucro sustentável no longo prazo exige crescimento real de receitas.",
    benchmark: "Acima da inflação: > 6% a.a.",
  },
  "CAGR Lucros (5a)": {
    title: "CAGR de Lucros (5 Anos)",
    formula: "((Lucro Atual ÷ Lucro há 5 anos)^(1/5) − 1) × 100",
    summary:
      "Taxa composta de crescimento anual do lucro líquido nos últimos 5 anos.",
    analysis:
      "Quando o CAGR de Lucros supera o CAGR de Receitas, a empresa apresenta ganho de escala operacional (alavancagem operacional positiva), diluindo custos fixos à medida que cresce.",
    benchmark: "Forte expansão: > 10% a.a.",
  },
  "Dív. Líq / PL": {
    title: "Dívida Líquida sobre Patrimônio Líquido",
    formula: "(Dívida Bruta − Caixa e Equivalentes) ÷ Patrimônio Líquido",
    summary:
      "Mede quanto a dívida financeira líquida representa em relação ao capital próprio dos acionistas.",
    analysis:
      "• Negativo (< 0): Empresa possui MAIS CAIXA do que dívida (Caixa Líquido).\n• Entre 0 e 1,0x: Endividamento conservador e saudável.\n• Acima de 1,5x: Alavancagem elevada — parte relevante do negócio é financiada por terceiros, aumentando a sensibilidade à taxa de juros.",
    benchmark: "Conservador: < 0,80x | Alerta: > 1,50x",
  },
  "Dív. Líquida / PL": {
    title: "Dívida Líquida sobre Patrimônio Líquido",
    formula: "(Dívida Bruta − Caixa e Equivalentes) ÷ Patrimônio Líquido",
    summary:
      "Mede quanto a dívida financeira líquida representa em relação ao capital próprio dos acionistas.",
    analysis:
      "Valores negativos indicam posição de Caixa Líquido (mais disponibilidades que dívidas). Acima de 1,0x indica que a dívida líquida supera o patrimônio líquido contábil.",
    benchmark: "Conservador: < 0,80x | Alerta: > 1,50x",
  },
  "Dív. Líq / EBITDA": {
    title: "Dívida Líquida sobre EBITDA (Alavancagem Financeira)",
    formula: "(Dívida Bruta − Caixa) ÷ EBITDA 12M",
    summary:
      "Indica quantos anos de geração operacional de caixa (EBITDA) seriam necessários para quitar 100% da dívida líquida da empresa.",
    analysis:
      "É o indicador de solvência mais acompanhado por bancos e agências de rating:\n• < 0x: Caixa líquido (zero pressão financeira).\n• 0x a 2,0x: Endividamento muito confortável.\n• 2,0x a 3,0x: Alavancagem moderada (comum em elétricas com fluxo previsível).\n• > 3,5x: Zona de risco — juros altos podem consumir todo o lucro líquido.",
    benchmark: "Saudável: < 2,5x | Crítico: > 3,5x",
  },
  "Dív. Líquida / EBITDA": {
    title: "Dívida Líquida sobre EBITDA (Alavancagem Financeira)",
    formula: "(Dívida Bruta − Caixa) ÷ EBITDA 12M",
    summary:
      "Indica quantos anos de geração operacional de caixa (EBITDA) seriam necessários para quitar toda a dívida líquida.",
    analysis:
      "Acima de 3,0x em cenário de Selic elevada é um sinal amarelo importante, pois a despesa financeira tende a comprimir a Margem Líquida e os dividendos.",
    benchmark: "Saudável: < 2,5x | Crítico: > 3,5x",
  },
  "Dív. Líquida / EBIT": {
    title: "Dívida Líquida sobre EBIT",
    formula: "(Dívida Bruta − Caixa) ÷ EBIT 12M",
    summary:
      "Mede quantos anos de lucro operacional (após descontar a depreciação) são necessários para pagar a dívida líquida.",
    analysis:
      "Mais conservador que o Dív. Líq/EBITDA porque já reserva a parcela de caixa necessária para manter o parque industrial/imobilizado funcionando.",
    benchmark: "Confortável: < 3,0x",
  },
  "PL / Ativos": {
    title: "Patrimônio Líquido sobre Ativos Totais",
    formula: "Patrimônio Líquido ÷ Ativo Total",
    summary:
      "Mostra qual fração de todos os ativos da companhia é financiada com recursos próprios dos sócios.",
    analysis:
      "Quanto mais próximo de 0,50 ou superior, maior a independência financeira da empresa frente a bancos e credores.",
    benchmark: "Sólido: > 0,40",
  },
  "Passivos / Ativos": {
    title: "Passivos sobre Ativos Totais",
    formula: "Passivo Total ÷ Ativo Total",
    summary:
      "Indica a proporção dos ativos totais que está comprometida com obrigações (fornecedores, impostos, dívidas e provisões).",
    analysis:
      "Valores acima de 0,75 exigem cautela, pois deixam uma margem patrimonial estreita caso ocorra desvalorização de ativos ou queda de receitas.",
    benchmark: "Confortável: < 0,65",
  },
  "Liquidez Corrente": {
    title: "Índice de Liquidez Corrente (Curto Prazo)",
    formula: "Ativo Circulante ÷ Passivo Circulante",
    summary:
      "Avalia a capacidade da empresa de honrar todas as suas obrigações dos próximos 12 meses usando apenas seu caixa, contas a receber e estoques de curto prazo.",
    analysis:
      "• > 1,50: Folga financeira robusta no curto prazo.\n• 1,00 a 1,50: Equilíbrio operacional adequado.\n• < 1,00: As dívidas que vencem em até 1 ano superam os recursos de curto prazo, exigindo refinanciamento ou forte geração diária de caixa.",
    benchmark: "Saudável: > 1,20",
  },
  "Liquidez Seca": {
    title: "Índice de Liquidez Seca (Acid Test)",
    formula: "(Ativo Circulante − Estoques) ÷ Passivo Circulante",
    summary:
      "Teste rigoroso de solvência de curto prazo que exclui os estoques (pois estoques podem demorar a ser vendidos em uma crise).",
    analysis:
      "Acima de 1,00 significa que a empresa consegue pagar todas as contas de curto prazo usando apenas caixa disponível e recebíveis líquidos, sem precisar vender uma única peça do estoque.",
    benchmark: "Forte solvência: > 1,00",
  },
  "Piotroski F-Score": {
    title: "Piotroski F-Score (Saúde Financeira 0 a 9)",
    formula: "Soma de 9 critérios binários: Lucratividade (4), Alavancagem/Liquidez (3) e Eficiência (2)",
    summary:
      "Criado pelo Prof. Joseph Piotroski (Stanford), pontua de 0 a 9 a evolução da saúde contábil e operacional da empresa.",
    analysis:
      "• 7 a 9 pontos: Empresa financeiramente muito forte, com geração de caixa operacional superior ao lucro contábil, queda de alavancagem e expansão de margens.\n• 4 a 6 pontos: Estabilidade intermediária.\n• 0 a 3 pontos: Deterioração financeira ('value trap') — alerta de diluição acionária, queima de caixa ou aumento de dívida.",
    benchmark: "Forte: 7 a 9 | Neutro: 4 a 6 | Frágil: 0 a 3",
  },
  "Piotroski e Altman Z": {
    title: "Scores de Solidez: Piotroski F-Score & Altman Z-Score",
    formula: "F-Score (0–9 pontos operacionais) + Altman Z (combinação ponderada de 5 índices de balanço)",
    summary:
      "Combina a nota de evolução operacional de Piotroski com o modelo estatístico de risco de insolvência de Edward Altman.",
    analysis:
      "F-Score ≥ 7 confirma tendência positiva nos fundamentos. No Altman Z-Score, valores acima de 2,99 situam a empresa na Zona Segura (baixíssimo risco de crédito), entre 1,81 e 2,99 na Zona de Alerta, e abaixo de 1,81 na Zona de Estresse Financeiro.",
    benchmark: "F-Score ≥ 7 e Altman Z > 2,99",
  },
  "Vacância Física": {
    title: "Taxa de Vacância Física do Portfólio Imobiliário",
    formula: "(Área Bruta Locável Vaga em m² ÷ Área Bruta Locável Total em m²) × 100",
    summary:
      "Percentual do espaço físico total dos imóveis do FII (galpões, lajes corporativas ou lojas de shopping) que está desocupado no momento.",
    analysis:
      "Quanto menor, melhor. Imóveis vagos não apenas deixam de gerar receita de aluguel como geram despesas diretas para o fundo (IPTU, condomínio e manutenção). Em fundos de Papel/CRI e Fiagros, não há imóveis físicos (N/A).",
    benchmark: "Logística/Shoppings: < 5% | Lajes: < 10%",
  },
  "Vacância Financ.": {
    title: "Taxa de Vacância Financeira",
    formula: "(Receita Potencial das Áreas Vagas ÷ Receita Potencial Total do Portfólio) × 100",
    summary:
      "Mede o impacto financeiro da desocupação, ponderando as áreas vagas pelo valor de locação de mercado de cada espaço.",
    analysis:
      "Se a Vacância Financeira for MAIOR que a Vacância Física, significa que justamente os espaços mais nobres e caros do fundo estão vagos. Se for MENOR que a física, as áreas desocupadas são secundárias e afetam pouco a receita global do FII.",
    benchmark: "Ideal: ≤ Vacância Física e < 5%",
  },
  "Vacância Física / Financ.": {
    title: "Vacância Física vs. Vacância Financeira",
    formula: "Física (% da ABL em m² desocupada) vs. Financeira (% da receita potencial não capturada)",
    summary:
      "Confronta o espaço físico vago com o impacto econômico real na receita de locação do Fundo Imobiliário.",
    analysis:
      "Quando a Vacância Financeira supera a Física, os imóveis vagos têm aluguel/m² acima da média do portfólio. Em fundos de Papel/CRI não se aplica (N/A), devendo-se monitorar a inadimplência e razão de garantia dos certificados.",
    benchmark: "Tijolo saudável: < 5%",
  },
  "Cotistas": {
    title: "Número Total de Investidores (Cotistas)",
    formula: "Total de CPFs e CNPJs detentores de cotas na B3",
    summary:
      "Indica a pulverização da base de investidores e a relevância institucional do Fundo Imobiliário no mercado.",
    analysis:
      "FIIs com mais de 100.000 cotistas costumam apresentar excelente liquidez diária no book de ofertas (facilitando a entrada e saída sem distorcer o preço) e menor risco de concentração nas mãos de um único investidor institucional.",
    benchmark: "Alta liquidez/pulverização: > 100 mil cotistas",
  },
  "Taxa Adm.": {
    title: "Taxa de Administração e Gestão Anual",
    formula: "Percentual anual cobrado sobre o Patrimônio Líquido ou Valor de Mercado do FII",
    summary:
      "Custo operacional anual descontado do fundo para remunerar o administrador fiduciário, a gestora ativa e a escrituração.",
    analysis:
      "Impacta diretamente o rendimento líquido que chega ao cotista no longo prazo. Para FIIs de gestão ativa, taxas totais (administração + gestão) entre 0,60% e 1,05% a.a. estão dentro do padrão de mercado; acima de 1,25% a.a. exige retorno diferenciando da gestão.",
    benchmark: "Competitiva: 0,50% a 1,00% a.a.",
  },
  "Último Rendimento": {
    title: "Último Provento Mensal Distribuído",
    formula: "Valor em R$ pago por cada cota na última distribuição mensal",
    summary:
      "Rendimento mensal isento de Imposto de Renda para pessoa física creditado aos detentores do FII na data-com.",
    analysis:
      "Compare o Último Rendimento com a média de 12 meses (DPA ÷ 12) e com o resultado caixa do relatório gerencial para verificar se o patamar atual é recorrente ou fruto de distribuição extraordinária de semestre.",
    benchmark: "Yield mensal saudável: 0,70% a 1,10% a.m.",
  },
  "Segmento": {
    title: "Segmento de Atuação do FII",
    formula: "Classificação ANBIMA / B3 por natureza dos ativos-alvo",
    summary:
      "Define se o fundo investe em imóveis físicos (Tijolo: Logística, Shoppings, Lajes, Renda Urbana), títulos de crédito (Papel/CRI), Agronegócio (Fiagro) ou cotas de outros fundos (FoF).",
    analysis:
      "FIIs de Tijolo protegem o patrimônio contra a inflação no longo prazo via valorização dos imóveis e reajuste dos aluguéis. FIIs de Papel repassam juros e correção monetária no curto prazo com maior rendimento imediato.",
    benchmark: "Diversificar entre Tijolo e Papel",
  },
};

interface IndicatorLabelProps {
  label: string;
  lookupKey?: string;
  style?: React.CSSProperties;
  className?: string;
}

export const IndicatorLabel: React.FC<IndicatorLabelProps> = ({
  label,
  lookupKey,
  style,
  className,
}) => {
  const info = INDICATOR_GUIDE[lookupKey ?? label];
  const [open, setOpen] = useState(false);
  const [coords, setCoords] = useState<{
    top: number;
    left: number;
    placeAbove: boolean;
  }>({ top: 0, left: 0, placeAbove: false });
  const triggerRef = useRef<HTMLSpanElement | null>(null);

  const updatePosition = () => {
    const el = triggerRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const tooltipWidth = Math.min(340, window.innerWidth - 24);
    const spaceBelow = window.innerHeight - rect.bottom;
    const placeAbove = spaceBelow < 240 && rect.top > 240;

    let left = rect.left;
    if (left + tooltipWidth > window.innerWidth - 12) {
      left = Math.max(12, window.innerWidth - tooltipWidth - 12);
    }

    setCoords({
      top: placeAbove ? rect.top - 8 : rect.bottom + 8,
      left,
      placeAbove,
    });
  };

  useEffect(() => {
    if (!open) return;
    const handleScrollOrResize = () => setOpen(false);
    window.addEventListener("scroll", handleScrollOrResize, true);
    window.addEventListener("resize", handleScrollOrResize);
    return () => {
      window.removeEventListener("scroll", handleScrollOrResize, true);
      window.removeEventListener("resize", handleScrollOrResize);
    };
  }, [open]);

  if (!info) {
    return (
      <span className={className} style={style}>
        {label}
      </span>
    );
  }

  return (
    <>
      <span
        ref={triggerRef}
        className={`indicator-tooltip-trigger ${className ?? ""}`}
        style={style}
        onMouseEnter={() => {
          updatePosition();
          setOpen(true);
        }}
        onMouseLeave={() => setOpen(false)}
        onClick={(e) => {
          e.stopPropagation();
          updatePosition();
          setOpen((prev) => !prev);
        }}
      >
        <span>{label}</span>
        <span className="indicator-tooltip-icon" aria-hidden="true">
          ⓘ
        </span>
      </span>

      {open && (
        <div
          className="indicator-tooltip-popover"
          style={{
            top: `${coords.top}px`,
            left: `${coords.left}px`,
            transform: coords.placeAbove ? "translateY(-100%)" : "none",
          }}
          role="tooltip"
        >
          <div className="indicator-tooltip-header">
            <strong className="indicator-tooltip-title">{info.title}</strong>
            {info.benchmark && (
              <span className="indicator-tooltip-badge">{info.benchmark}</span>
            )}
          </div>
          <div className="indicator-tooltip-formula">
            <span>Fórmula:</span> <code>{info.formula}</code>
          </div>
          <div className="indicator-tooltip-summary">{info.summary}</div>
          <div className="indicator-tooltip-analysis">
            <div className="indicator-tooltip-analysis-label">
              Leitura Analítica
            </div>
            <div className="indicator-tooltip-analysis-body">
              {info.analysis}
            </div>
          </div>
        </div>
      )}
    </>
  );
};
