export type AssetClass =
  | "Acao"
  | "Fii"
  | "Bdr"
  | "Etf"
  | "Opcao"
  | "Tesouro"
  | "Indice";

export function assetClassFromSymbol(symbol: string): AssetClass {
  const s = symbol.trim().toUpperCase();
  if (s.startsWith("IBOV") || s.startsWith("IFIX") || s.startsWith("SMLL")) {
    return "Indice";
  }
  if (s.endsWith("11")) {
    if (
      s.startsWith("BOVA") ||
      s.startsWith("IVVB") ||
      s.startsWith("SMAL") ||
      s.startsWith("HASH") ||
      s.startsWith("GOLD") ||
      s.startsWith("NASD") ||
      s.startsWith("DIVO") ||
      s.startsWith("BRAX")
    ) {
      return "Etf";
    }
    return "Fii";
  }
  if (s.endsWith("33") || s.endsWith("34") || s.endsWith("35")) {
    return "Bdr";
  }
  return "Acao";
}

export interface B3Asset {
  ticker: string;
  name: string;
  sector: string;
  assetClass: AssetClass;
  basePrice: number;
}

export const B3_ASSET_CATALOG: readonly B3Asset[] = [
  // Ações (127 items)
  { ticker: "PETR4", name: "Petrobras PN", sector: "Petróleo, Gás e Biocombustíveis", assetClass: "Acao", basePrice: 52.71 },
  { ticker: "PETR3", name: "Petrobras ON", sector: "Petróleo, Gás e Biocombustíveis", assetClass: "Acao", basePrice: 60.01 },
  { ticker: "VALE3", name: "Vale ON", sector: "Mineração e Siderurgia", assetClass: "Acao", basePrice: 62.02 },
  { ticker: "ITUB4", name: "Itaú Unibanco PN", sector: "Bancos e Serviços Financeiros", assetClass: "Acao", basePrice: 38.69 },
  { ticker: "ITUB3", name: "Itaú Unibanco ON", sector: "Bancos e Serviços Financeiros", assetClass: "Acao", basePrice: 43.06 },
  { ticker: "BBDC4", name: "Bradesco PN", sector: "Bancos e Serviços Financeiros", assetClass: "Acao", basePrice: 13.86 },
  { ticker: "BBDC3", name: "Bradesco ON", sector: "Bancos e Serviços Financeiros", assetClass: "Acao", basePrice: 12.71 },
  { ticker: "BBAS3", name: "Banco do Brasil ON", sector: "Bancos e Serviços Financeiros", assetClass: "Acao", basePrice: 16.47 },
  { ticker: "WEGE3", name: "WEG ON", sector: "Bens Industriais e Motores", assetClass: "Acao", basePrice: 48.36 },
  { ticker: "ABEV3", name: "Ambev ON", sector: "Bebidas e Consumo Não Cíclico", assetClass: "Acao", basePrice: 14.60 },
  { ticker: "RENT3", name: "Localiza ON", sector: "Aluguel de Carros e Logística", assetClass: "Acao", basePrice: 27.85 },
  { ticker: "BPAC11", name: "BTG Pactual UNT", sector: "Bancos e Serviços Financeiros", assetClass: "Acao", basePrice: 51.52 },
  { ticker: "ITSA4", name: "Itaúsa PN", sector: "Holdings e Serviços Financeiros", assetClass: "Acao", basePrice: 12.45 },
  { ticker: "ITSA3", name: "Itaúsa ON", sector: "Holdings e Serviços Financeiros", assetClass: "Acao", basePrice: 12.93 },
  { ticker: "BBSE3", name: "BB Seguridade ON", sector: "Seguros e Previdência", assetClass: "Acao", basePrice: 41.32 },
  { ticker: "PRIO3", name: "PRIO (PetroRio) ON", sector: "Petróleo, Gás e Biocombustíveis", assetClass: "Acao", basePrice: 71.49 },
  { ticker: "SUZB3", name: "Suzano ON", sector: "Papel e Celulose", assetClass: "Acao", basePrice: 50.14 },
  { ticker: "GGBR4", name: "Gerdau PN", sector: "Mineração e Siderurgia", assetClass: "Acao", basePrice: 25.13 },
  { ticker: "GGBR3", name: "Gerdau ON", sector: "Mineração e Siderurgia", assetClass: "Acao", basePrice: 21.48 },
  { ticker: "CSNA3", name: "CSN Siderúrgica ON", sector: "Mineração e Siderurgia", assetClass: "Acao", basePrice: 3.88 },
  { ticker: "JBSS3", name: "JBS ON", sector: "Alimentos e Frigoríficos", assetClass: "Acao", basePrice: 32.70 },
  { ticker: "BRFS3", name: "BRF ON", sector: "Alimentos e Frigoríficos", assetClass: "Acao", basePrice: 24.10 },
  { ticker: "CPLE6", name: "Copel PNB", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 9.80 },
  { ticker: "CPLE3", name: "Copel ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 14.32 },
  { ticker: "CMIG4", name: "Cemig PN", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 10.07 },
  { ticker: "CMIG3", name: "Cemig ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 16.45 },
  { ticker: "ELET3", name: "Eletrobras ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 38.60 },
  { ticker: "ELET6", name: "Eletrobras PNB", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 42.10 },
  { ticker: "EQTL3", name: "Equatorial Energia ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 32.60 },
  { ticker: "SBSP3", name: "Sabesp ON", sector: "Saneamento Básico", assetClass: "Acao", basePrice: 24.70 },
  { ticker: "EMBR3", name: "Embraer ON", sector: "Bens Industriais e Aviação", assetClass: "Acao", basePrice: 48.30 },
  { ticker: "MGLU3", name: "Magazine Luiza ON", sector: "Varejo e E-commerce", assetClass: "Acao", basePrice: 5.35 },
  { ticker: "LREN3", name: "Lojas Renner ON", sector: "Varejo de Moda", assetClass: "Acao", basePrice: 8.98 },
  { ticker: "RADL3", name: "Raia Drogasil ON", sector: "Saúde e Farmácias", assetClass: "Acao", basePrice: 15.22 },
  { ticker: "VBBR3", name: "Vibra Energia ON", sector: "Petróleo, Gás e Biocombustíveis", assetClass: "Acao", basePrice: 29.60 },
  { ticker: "HYPE3", name: "Hypera Pharma ON", sector: "Saúde e Farmacêutica", assetClass: "Acao", basePrice: 21.27 },
  { ticker: "CCRO3", name: "CCR Rodovias ON", sector: "Concessões e Logística", assetClass: "Acao", basePrice: 12.80 },
  { ticker: "RAIL3", name: "Rumo Logística ON", sector: "Ferrovias e Logística", assetClass: "Acao", basePrice: 13.78 },
  { ticker: "VIVT3", name: "Telefônica Vivo ON", sector: "Telecomunicações", assetClass: "Acao", basePrice: 30.70 },
  { ticker: "TIMS3", name: "TIM Brasil ON", sector: "Telecomunicações", assetClass: "Acao", basePrice: 19.17 },
  { ticker: "KLBN11", name: "Klabin UNT", sector: "Papel e Celulose", assetClass: "Acao", basePrice: 17.20 },
  { ticker: "KLBN4", name: "Klabin PN", sector: "Papel e Celulose", assetClass: "Acao", basePrice: 3.41 },
  { ticker: "SANB11", name: "Santander Brasil UNT", sector: "Bancos e Serviços Financeiros", assetClass: "Acao", basePrice: 23.94 },
  { ticker: "ALOS3", name: "Allos (Aliansce Sonae) ON", sector: "Shoppings e Imobiliário", assetClass: "Acao", basePrice: 23.50 },
  { ticker: "MULT3", name: "Multiplan ON", sector: "Shoppings e Imobiliário", assetClass: "Acao", basePrice: 25.19 },
  { ticker: "CYRE3", name: "Cyrela Realty ON", sector: "Construção Civil", assetClass: "Acao", basePrice: 17.09 },
  { ticker: "MRVE3", name: "MRV Engenharia ON", sector: "Construção Civil", assetClass: "Acao", basePrice: 4.05 },
  { ticker: "EZTC3", name: "EZTEC ON", sector: "Construção Civil", assetClass: "Acao", basePrice: 8.96 },
  { ticker: "USIM5", name: "Usiminas PNA", sector: "Mineração e Siderurgia", assetClass: "Acao", basePrice: 5.00 },
  { ticker: "GOAU4", name: "Metalúrgica Gerdau PN", sector: "Mineração e Siderurgia", assetClass: "Acao", basePrice: 11.11 },
  { ticker: "CSAN3", name: "Cosan ON", sector: "Energia, Gás e Agronegócio", assetClass: "Acao", basePrice: 2.33 },
  { ticker: "SLCE3", name: "SLC Agrícola ON", sector: "Agronegócio", assetClass: "Acao", basePrice: 17.02 },
  { ticker: "SMTO3", name: "São Martinho ON", sector: "Açúcar, Etanol e Agronegócio", assetClass: "Acao", basePrice: 21.09 },
  { ticker: "BEEF3", name: "Minerva Foods ON", sector: "Alimentos e Frigoríficos", assetClass: "Acao", basePrice: 2.55 },
  { ticker: "MRFG3", name: "Marfrig ON", sector: "Alimentos e Frigoríficos", assetClass: "Acao", basePrice: 13.40 },
  { ticker: "DXCO3", name: "Dexco (Duratex) ON", sector: "Madeira e Construção", assetClass: "Acao", basePrice: 4.57 },
  { ticker: "TOTS3", name: "TOTVS ON", sector: "Tecnologia e Software", assetClass: "Acao", basePrice: 32.00 },
  { ticker: "POSI3", name: "Positivo Tecnologia ON", sector: "Tecnologia e Hardware", assetClass: "Acao", basePrice: 2.91 },
  { ticker: "LWSA3", name: "LWSA (Locaweb) ON", sector: "Tecnologia e Software", assetClass: "Acao", basePrice: 3.09 },
  { ticker: "CASH3", name: "Méliuz ON", sector: "Tecnologia e Fintech", assetClass: "Acao", basePrice: 4.13 },
  { ticker: "ENGI11", name: "Energisa UNT", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 44.29 },
  { ticker: "TAEE11", name: "Taesa UNT", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 39.59 },
  { ticker: "TAEE4", name: "Taesa PN", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 13.30 },
  { ticker: "TAEE3", name: "Taesa ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 12.90 },
  { ticker: "EGIE3", name: "Engie Brasil ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 26.26 },
  { ticker: "CPFE3", name: "CPFL Energia ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 41.90 },
  { ticker: "TRPL4", name: "ISA Energia (CTEEP) PN", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 24.60 },
  { ticker: "NEOE3", name: "Neoenergia ON", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 18.90 },
  { ticker: "CXSE3", name: "Caixa Seguridade ON", sector: "Seguros e Previdência", assetClass: "Acao", basePrice: 21.38 },
  { ticker: "PSSA3", name: "Porto Seguro ON", sector: "Seguros e Previdência", assetClass: "Acao", basePrice: 44.75 },
  { ticker: "B3SA3", name: "B3 (Brasil Bolsa Balcão) ON", sector: "Serviços Financeiros e Bolsa", assetClass: "Acao", basePrice: 13.43 },
  { ticker: "CIEL3", name: "Cielo ON", sector: "Meios de Pagamento", assetClass: "Acao", basePrice: 5.60 },
  { ticker: "COGN3", name: "Cogna Educação ON", sector: "Educação", assetClass: "Acao", basePrice: 1.58 },
  { ticker: "YDUQ3", name: "Yduqs Educação ON", sector: "Educação", assetClass: "Acao", basePrice: 8.91 },
  { ticker: "ASAI3", name: "Assaí Atacadista ON", sector: "Varejo Alimentar", assetClass: "Acao", basePrice: 7.88 },
  { ticker: "CRFB3", name: "Carrefour Brasil ON", sector: "Varejo Alimentar", assetClass: "Acao", basePrice: 8.90 },
  { ticker: "AZUL4", name: "Azul Linhas Aéreas PN", sector: "Aviação e Transporte", assetClass: "Acao", basePrice: 5.30 },
  { ticker: "GOLL4", name: "Gol Linhas Aéreas PN", sector: "Aviação e Transporte", assetClass: "Acao", basePrice: 1.15 },
  { ticker: "CMIN3", name: "CSN Mineração ON", sector: "Mineração e Siderurgia", assetClass: "Acao", basePrice: 4.86 },
  { ticker: "BRAP4", name: "Bradespar PN", sector: "Holdings e Mineração", assetClass: "Acao", basePrice: 18.27 },
  { ticker: "FLRY3", name: "Fleury ON", sector: "Saúde e Diagnósticos", assetClass: "Acao", basePrice: 20.47 },
  { ticker: "HAPV3", name: "Hapvida ON", sector: "Saúde e Hospitais", assetClass: "Acao", basePrice: 4.20 },
  { ticker: "RDOR3", name: "Rede D'Or ON", sector: "Saúde e Hospitais", assetClass: "Acao", basePrice: 31.68 },
  { ticker: "ODPV3", name: "OdontoPrev ON", sector: "Saúde e Odontologia", assetClass: "Acao", basePrice: 11.70 },
  { ticker: "VAMO3", name: "Vamos Locação ON", sector: "Aluguel de Caminhões e Máquinas", assetClass: "Acao", basePrice: 2.38 },
  { ticker: "RANI3", name: "Irani Papel e Embalagem ON", sector: "Papel e Celulose", assetClass: "Acao", basePrice: 7.59 },
  { ticker: "MDIA3", name: "M. Dias Branco ON", sector: "Alimentos e Consumo", assetClass: "Acao", basePrice: 15.35 },
  { ticker: "CURY3", name: "Cury Construtora ON", sector: "Construção Civil", assetClass: "Acao", basePrice: 25.90 },
  { ticker: "DIRR3", name: "Direcional Engenharia ON", sector: "Construção Civil", assetClass: "Acao", basePrice: 8.06 },
  { ticker: "PLPL3", name: "Plano & Plano ON", sector: "Construção Civil", assetClass: "Acao", basePrice: 4.47 },
  { ticker: "JHSF3", name: "JHSF Participações ON", sector: "Shoppings e Incorporação", assetClass: "Acao", basePrice: 8.97 },
  { ticker: "LOGG3", name: "LOG Commercial Properties ON", sector: "Galpões Logísticos", assetClass: "Acao", basePrice: 20.23 },
  { ticker: "SIMH3", name: "Simpar ON", sector: "Holdings e Logística", assetClass: "Acao", basePrice: 4.84 },
  { ticker: "STBP3", name: "Santos Brasil ON", sector: "Portos e Logística", assetClass: "Acao", basePrice: 12.95 },
  { ticker: "MOVI3", name: "Movida ON", sector: "Aluguel de Carros", assetClass: "Acao", basePrice: 5.61 },
  { ticker: "ECOR3", name: "EcoRodovias ON", sector: "Concessões Rodoviárias", assetClass: "Acao", basePrice: 4.89 },
  { ticker: "BRKM5", name: "Braskem PNA", sector: "Petroquímica", assetClass: "Acao", basePrice: 2.99 },
  { ticker: "UNIP6", name: "Unipar Carbocloro PNB", sector: "Química e Petroquímica", assetClass: "Acao", basePrice: 51.00 },
  { ticker: "RECV3", name: "PetroReconcavo ON", sector: "Petróleo e Gás", assetClass: "Acao", basePrice: 11.03 },
  { ticker: "BRAV3", name: "Brava Energia (3R + Enauta) ON", sector: "Petróleo e Gás", assetClass: "Acao", basePrice: 18.42 },
  { ticker: "AZZA3", name: "Azzas 2154 (Arezzo + Soma) ON", sector: "Varejo de Moda e Calçados", assetClass: "Acao", basePrice: 14.33 },
  { ticker: "VIVA3", name: "Vivara ON", sector: "Joalheria e Varejo", assetClass: "Acao", basePrice: 17.30 },
  { ticker: "GUAR3", name: "Guararapes (Riachuelo) ON", sector: "Varejo de Moda", assetClass: "Acao", basePrice: 7.40 },
  { ticker: "CEAB3", name: "C&A Modas ON", sector: "Varejo de Moda", assetClass: "Acao", basePrice: 6.55 },
  { ticker: "BHIA3", name: "Casas Bahia ON", sector: "Varejo e E-commerce", assetClass: "Acao", basePrice: 0.69 },
  { ticker: "PCAR3", name: "GPA (Pão de Açúcar) ON", sector: "Varejo Alimentar", assetClass: "Acao", basePrice: 2.08 },
  { ticker: "NTCO3", name: "Natura &Co ON", sector: "Cosméticos e Consumo", assetClass: "Acao", basePrice: 14.20 },
  { ticker: "AGRO3", name: "BrasilAgro ON", sector: "Agronegócio e Terras", assetClass: "Acao", basePrice: 17.31 },
  { ticker: "TTEN3", name: "3tentos Agroindustrial ON", sector: "Agronegócio", assetClass: "Acao", basePrice: 10.28 },
  { ticker: "CAML3", name: "Camil Alimentos ON", sector: "Alimentos e Consumo", assetClass: "Acao", basePrice: 3.98 },
  { ticker: "KEPL3", name: "Kepler Weber ON", sector: "Máquinas Agrícolas e Silos", assetClass: "Acao", basePrice: 6.27 },
  { ticker: "TUPY3", name: "Tupy ON", sector: "Indústria Automotiva e Fundição", assetClass: "Acao", basePrice: 14.46 },
  { ticker: "ROMI3", name: "Indústrias Romi ON", sector: "Máquinas e Equipamentos", assetClass: "Acao", basePrice: 5.32 },
  { ticker: "LEVE3", name: "Mahle Metal Leve ON", sector: "Autopeças e Indústria", assetClass: "Acao", basePrice: 30.26 },
  { ticker: "FRAS3", name: "Fras-le Mobility ON", sector: "Autopeças e Indústria", assetClass: "Acao", basePrice: 20.17 },
  { ticker: "RAPT4", name: "Randoncorp PN", sector: "Implementos Rodoviários", assetClass: "Acao", basePrice: 3.42 },
  { ticker: "POMO4", name: "Marcopolo PN", sector: "Ônibus e Carrocerias", assetClass: "Acao", basePrice: 3.70 },
  { ticker: "INTB3", name: "Intelbras ON", sector: "Tecnologia e Segurança Eletrônica", assetClass: "Acao", basePrice: 13.87 },
  { ticker: "SAPR11", name: "Sanepar UNT", sector: "Saneamento Básico", assetClass: "Acao", basePrice: 30.85 },
  { ticker: "SAPR4", name: "Sanepar PN", sector: "Saneamento Básico", assetClass: "Acao", basePrice: 6.03 },
  { ticker: "CSMG3", name: "Copasa MG ON", sector: "Saneamento Básico", assetClass: "Acao", basePrice: 48.11 },
  { ticker: "ALUP11", name: "Alupar UNT", sector: "Energia Elétrica", assetClass: "Acao", basePrice: 31.40 },
  { ticker: "AMBP3", name: "Ambipar ON", sector: "Gestão Ambiental", assetClass: "Acao", basePrice: 0.14 },
  { ticker: "SOJA3", name: "Boa Safra Sementes ON", sector: "Agronegócio", assetClass: "Acao", basePrice: 5.90 },
  { ticker: "VULC3", name: "Vulcabras ON", sector: "Calçados e Artigos Esportivos", assetClass: "Acao", basePrice: 11.53 },
  { ticker: "GRND3", name: "Grendene ON", sector: "Calçados e Consumo", assetClass: "Acao", basePrice: 3.56 },
  { ticker: "CVCB3", name: "CVC Brasil ON", sector: "Turismo e Viagens", assetClass: "Acao", basePrice: 0.80 },

  // FIIs & Fiagros (43 items)
  { ticker: "HGLG11", name: "CSHG Logística FII", sector: "FII Logística e Galpões", assetClass: "Fii", basePrice: 149.50 },
  { ticker: "KNCR11", name: "Kinea Rendimentos Imobiliários FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 104.98 },
  { ticker: "MXRF11", name: "Maxi Renda FII", sector: "FII Híbrido e CRIs", assetClass: "Fii", basePrice: 8.88 },
  { ticker: "XPML11", name: "XP Malls FII", sector: "FII Shoppings", assetClass: "Fii", basePrice: 96.80 },
  { ticker: "BTLG11", name: "BTG Pactual Logística FII", sector: "FII Logística e Galpões", assetClass: "Fii", basePrice: 97.92 },
  { ticker: "VISC11", name: "Vinci Shopping Centers FII", sector: "FII Shoppings", assetClass: "Fii", basePrice: 99.20 },
  { ticker: "KNRI11", name: "Kinea Renda Imobiliária FII", sector: "FII Híbrido (Lajes e Galpões)", assetClass: "Fii", basePrice: 150.50 },
  { ticker: "HGRU11", name: "CSHG Renda Urbana FII", sector: "FII Renda Urbana e Varejo", assetClass: "Fii", basePrice: 114.00 },
  { ticker: "HGRE11", name: "CSHG Real Estate FII", sector: "FII Lajes Corporativas", assetClass: "Fii", basePrice: 112.50 },
  { ticker: "HGBS11", name: "Hedge Brasil Shopping FII", sector: "FII Shoppings", assetClass: "Fii", basePrice: 18.06 },
  { ticker: "BRCO11", name: "Bresco Logística FII", sector: "FII Logística e Galpões", assetClass: "Fii", basePrice: 105.38 },
  { ticker: "XPLG11", name: "XP Log FII", sector: "FII Logística e Galpões", assetClass: "Fii", basePrice: 91.19 },
  { ticker: "VILG11", name: "Vinci Logística FII", sector: "FII Logística e Galpões", assetClass: "Fii", basePrice: 89.60 },
  { ticker: "PVBI11", name: "VBI Prime Properties FII", sector: "FII Lajes Corporativas", assetClass: "Fii", basePrice: 61.00 },
  { ticker: "JSRE11", name: "JS Real Estate Multigestão FII", sector: "FII Lajes Corporativas", assetClass: "Fii", basePrice: 58.27 },
  { ticker: "RBRR11", name: "RBR Rendimento High Grade FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 65.01 },
  { ticker: "CPTS11", name: "Capitânia Securities II FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 7.00 },
  { ticker: "KNHY11", name: "Kinea High Yield FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 92.65 },
  { ticker: "IRDM11", name: "Iridium Recebíveis Imobiliários FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 58.50 },
  { ticker: "VGIR11", name: "Valora RE III FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 8.90 },
  { ticker: "RECR11", name: "REC Recebíveis Imobiliários FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 71.03 },
  { ticker: "TGAR11", name: "TG Ativo Real FII", sector: "FII Desenvolvimento e Híbrido", assetClass: "Fii", basePrice: 40.32 },
  { ticker: "MALL11", name: "Malls Brasil Plural FII", sector: "FII Shoppings", assetClass: "Fii", basePrice: 112.40 },
  { ticker: "LVBI11", name: "VBI Logístico FII", sector: "FII Logística e Galpões", assetClass: "Fii", basePrice: 95.99 },
  { ticker: "RBRP11", name: "RBR Properties FII", sector: "FII Lajes e Híbrido", assetClass: "Fii", basePrice: 43.59 },
  { ticker: "BCFF11", name: "BTG Pactual Fundo de Fundos FII", sector: "FII Fundo de Fundos (FoF)", assetClass: "Fii", basePrice: 8.95 },
  { ticker: "VRTA11", name: "Fator Verità FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 68.66 },
  { ticker: "VINO11", name: "Vinci Offices FII", sector: "FII Lajes Corporativas", assetClass: "Fii", basePrice: 4.14 },
  { ticker: "GGRC11", name: "GGR Covepi Renda FII", sector: "FII Logística e Industrial", assetClass: "Fii", basePrice: 8.48 },
  { ticker: "TRXF11", name: "TRX Real Estate FII", sector: "FII Renda Urbana e Varejo", assetClass: "Fii", basePrice: 66.45 },
  { ticker: "ALZR11", name: "Alianza Trust Renda Imobiliária FII", sector: "FII Renda Urbana e Contratos Atípicos", assetClass: "Fii", basePrice: 9.70 },
  { ticker: "GALG11", name: "Guardian Logística FII", sector: "FII Logística", assetClass: "Fii", basePrice: 9.15 },
  { ticker: "SNCI11", name: "Suno Recebíveis Imobiliários FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 81.11 },
  { ticker: "SNFF11", name: "Suno Fundo de Fundos FII", sector: "FII Fundo de Fundos (FoF)", assetClass: "Fii", basePrice: 74.00 },
  { ticker: "MCCI11", name: "Mauá Capital Recebíveis FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 89.70 },
  { ticker: "HGCR11", name: "CSHG Recebíveis Imobiliários FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 92.69 },
  { ticker: "KNSC11", name: "Kinea Securities FII", sector: "FII Papéis e CRIs", assetClass: "Fii", basePrice: 8.74 },
  { ticker: "RURA11", name: "Itaú Asset Rural Fiagro", sector: "Fiagro e Agronegócio", assetClass: "Fii", basePrice: 7.90 },
  { ticker: "VGIA11", name: "Valora CRA Fiagro", sector: "Fiagro e Agronegócio", assetClass: "Fii", basePrice: 8.50 },
  { ticker: "KNCA11", name: "Kinea Crédito Agro Fiagro", sector: "Fiagro e Agronegócio", assetClass: "Fii", basePrice: 85.24 },
  { ticker: "XPCA11", name: "XP Crédito Agrícola Fiagro", sector: "Fiagro e Agronegócio", assetClass: "Fii", basePrice: 7.10 },
  { ticker: "SNAG11", name: "Suno Agro Fiagro", sector: "Fiagro e Agronegócio", assetClass: "Fii", basePrice: 9.58 },
  { ticker: "BTAL11", name: "BTG Pactual Agro Logística FII", sector: "FII Agro e Logística", assetClass: "Fii", basePrice: 79.30 },

  // ETFs (8 items)
  { ticker: "BOVA11", name: "iShares Ibovespa ETF", sector: "ETF Índice Ibovespa", assetClass: "Etf", basePrice: 165.40 },
  { ticker: "IVVB11", name: "iShares S&P 500 ETF", sector: "ETF Internacional S&P 500", assetClass: "Etf", basePrice: 466.50 },
  { ticker: "SMAL11", name: "iShares Small Cap ETF", sector: "ETF Small Caps B3", assetClass: "Etf", basePrice: 92.15 },
  { ticker: "HASH11", name: "Hashdex Nasdaq Crypto ETF", sector: "ETF Criptoativos", assetClass: "Etf", basePrice: 53.81 },
  { ticker: "GOLD11", name: "Trend Ouro ETF", sector: "ETF Commodities e Ouro", assetClass: "Etf", basePrice: 21.89 },
  { ticker: "NASD11", name: "Trend Nasdaq 100 ETF", sector: "ETF Internacional Tecnologia", assetClass: "Etf", basePrice: 21.48 },
  { ticker: "DIVO11", name: "It Now IDIV Dividendos ETF", sector: "ETF Dividendos B3", assetClass: "Etf", basePrice: 119.74 },
  { ticker: "BRAX11", name: "iShares IBrX-100 ETF", sector: "ETF Índice Brasil", assetClass: "Etf", basePrice: 140.90 },

  // BDRs (9 items)
  { ticker: "AAPL34", name: "Apple Inc. BDR", sector: "BDR Tecnologia Internacional", assetClass: "Bdr", basePrice: 93.86 },
  { ticker: "MSFT34", name: "Microsoft Corp. BDR", sector: "BDR Tecnologia Internacional", assetClass: "Bdr", basePrice: 118.27 },
  { ticker: "NVDC34", name: "NVIDIA Corp. BDR", sector: "BDR Semicondutores e IA", assetClass: "Bdr", basePrice: 23.33 },
  { ticker: "AMZO34", name: "Amazon.com Inc. BDR", sector: "BDR E-commerce e Cloud", assetClass: "Bdr", basePrice: 65.60 },
  { ticker: "GOOGL34", name: "Alphabet (Google) BDR", sector: "BDR Tecnologia Internacional", assetClass: "Bdr", basePrice: 76.50 },
  { ticker: "META34", name: "Meta Platforms BDR", sector: "BDR Redes Sociais e IA", assetClass: "Bdr", basePrice: 122.00 },
  { ticker: "TSLA34", name: "Tesla Inc. BDR", sector: "BDR Veículos Elétricos", assetClass: "Bdr", basePrice: 67.81 },
  { ticker: "MELI34", name: "Mercado Livre BDR", sector: "BDR E-commerce e Fintech", assetClass: "Bdr", basePrice: 74.10 },
  { ticker: "NU34", name: "Nu Holdings (Nubank) BDR", sector: "BDR Bancos e Fintech", assetClass: "Bdr", basePrice: 14.80 },
];

export function getAllB3Assets(): readonly B3Asset[] {
  return B3_ASSET_CATALOG;
}

export function normalize(s: string): string {
  return s
    .toUpperCase()
    .replace(/[ÁÀÂÃÄ]/g, "A")
    .replace(/[ÉÈÊË]/g, "E")
    .replace(/[ÍÌÎÏ]/g, "I")
    .replace(/[ÓÒÔÕÖ]/g, "O")
    .replace(/[ÚÙÛÜ]/g, "U")
    .replace(/Ç/g, "C");
}

export function findB3Asset(ticker: string): B3Asset | undefined {
  const sym = ticker.trim().toUpperCase();
  const direct = B3_ASSET_CATALOG.find((a) => a.ticker === sym);
  if (direct) return direct;

  if (sym === "ARZZ3" || sym === "ARZZ4" || sym === "SOMA3" || sym === "SOMA4") {
    return B3_ASSET_CATALOG.find((a) => a.ticker === "AZZA3");
  }
  return undefined;
}

export function searchB3Assets(
  query: string,
  filterClass?: AssetClass | null
): B3Asset[] {
  const q = normalize(query.trim());

  const matchesClass = (a: B3Asset) =>
    !filterClass || a.assetClass === filterClass;

  if (!q) {
    return B3_ASSET_CATALOG.filter(matchesClass);
  }

  const results = B3_ASSET_CATALOG.filter((a) => {
    if (!matchesClass(a)) return false;
    return (
      normalize(a.ticker).includes(q) ||
      normalize(a.name).includes(q) ||
      normalize(a.sector).includes(q)
    );
  });

  if (results.length === 0 && q.length >= 4 && /^[A-Z]{4}/.test(q)) {
    const prefix = q.slice(0, 4);
    const prefixResults = B3_ASSET_CATALOG.filter(
      (a) => matchesClass(a) && a.ticker.startsWith(prefix)
    );
    if (prefixResults.length > 0) return prefixResults;
  }

  if (results.length === 0) {
    const alias = findB3Asset(q);
    if (alias && matchesClass(alias)) {
      return [alias];
    }
  }

  return results;
}
