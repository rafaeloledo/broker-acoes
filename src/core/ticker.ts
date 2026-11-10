import { type AssetClass, findB3Asset, assetClassFromSymbol } from "./b3-assets.ts";

export { type AssetClass, assetClassFromSymbol };

export type MarketSector =
  | "Financeiro"
  | "PetroleoGasBiocombustiveis"
  | "MateriaisBasicos"
  | "UtilidadePublica"
  | "ConsumoNaoCiclico"
  | "ConsumoCiclico"
  | "BensIndustriais"
  | "Saude"
  | "TecnologiaInformacao"
  | "Comunicacoes"
  | "Imobiliario"
  | "Outros";

export function marketSectorDisplayName(sector: MarketSector): string {
  switch (sector) {
    case "Financeiro":
      return "Financeiro & Seguros";
    case "PetroleoGasBiocombustiveis":
      return "Petróleo, Gás e Biocombustíveis";
    case "MateriaisBasicos":
      return "Materiais Básicos (Mineração/Siderurgia)";
    case "UtilidadePublica":
      return "Utilidade Pública (Energia/Saneamento)";
    case "ConsumoNaoCiclico":
      return "Consumo Não Cíclico (Alimentos/Bebidas)";
    case "ConsumoCiclico":
      return "Consumo Cíclico (Varejo/Construção)";
    case "BensIndustriais":
      return "Bens Industriais (Maquinário/Transporte)";
    case "Saude":
      return "Saúde (Hospitais/Farmácias)";
    case "TecnologiaInformacao":
      return "Tecnologia da Informação";
    case "Comunicacoes":
      return "Comunicações e Telecom";
    case "Imobiliario":
      return "Imobiliário (FIIs e Shoppings)";
    case "Outros":
      return "Outros Setores";
  }
}

export function marketSectorFromName(name: string): MarketSector {
  const lower = name.toLowerCase();
  if (lower.includes("petr") || lower.includes("gás") || lower.includes("combust")) {
    return "PetroleoGasBiocombustiveis";
  }
  if (lower.includes("financ") || lower.includes("banco") || lower.includes("segur")) {
    return "Financeiro";
  }
  if (
    lower.includes("miner") ||
    lower.includes("sider") ||
    lower.includes("metal") ||
    lower.includes("básico")
  ) {
    return "MateriaisBasicos";
  }
  if (
    lower.includes("energia") ||
    lower.includes("elétr") ||
    lower.includes("saneam") ||
    lower.includes("utilidade")
  ) {
    return "UtilidadePublica";
  }
  if (
    lower.includes("alimento") ||
    lower.includes("bebid") ||
    lower.includes("agro") ||
    lower.includes("não cíclico")
  ) {
    return "ConsumoNaoCiclico";
  }
  if (
    lower.includes("varejo") ||
    lower.includes("constru") ||
    lower.includes("moda") ||
    lower.includes("cíclico")
  ) {
    return "ConsumoCiclico";
  }
  if (
    lower.includes("indústr") ||
    lower.includes("bens ind") ||
    lower.includes("máquin") ||
    lower.includes("transp")
  ) {
    return "BensIndustriais";
  }
  if (
    lower.includes("saúde") ||
    lower.includes("farm") ||
    lower.includes("hosp") ||
    lower.includes("medic")
  ) {
    return "Saude";
  }
  if (lower.includes("tecno") || lower.includes("softw") || lower.includes("inform")) {
    return "TecnologiaInformacao";
  }
  if (lower.includes("telecom") || lower.includes("comunic")) {
    return "Comunicacoes";
  }
  if (
    lower.includes("imobil") ||
    lower.includes("fii") ||
    lower.includes("shopping") ||
    lower.includes("laje")
  ) {
    return "Imobiliario";
  }
  return "Outros";
}

export interface CompanyProfile {
  symbol: string;
  name: string;
  cnpj: string;
  cvmCode: number | null;
  sector: MarketSector;
  subsector: string;
  segment: string;
  totalShares: number;
  commonShares: number;
  preferredShares: number;
  freeFloatPercent: number;
  website: string;
  description: string;
}

export function defaultCompanyProfile(symbol: string): CompanyProfile {
  const sym = symbol.trim().toUpperCase();
  let name = "Ativo B3";
  let sector: MarketSector = "Outros";
  let subsector = "Mercado à Vista";
  let shares = 100_000_000;

  switch (sym) {
    case "PETR4":
    case "PETR3":
      name = "Petróleo Brasileiro S.A. - Petrobras";
      sector = "PetroleoGasBiocombustiveis";
      subsector = "Exploração, Refino e Distribuição";
      shares = 13_044_496_930;
      break;
    case "VALE3":
      name = "Vale S.A.";
      sector = "MateriaisBasicos";
      subsector = "Minerais Metálicos";
      shares = 4_539_007_568;
      break;
    case "WEGE3":
      name = "WEG S.A.";
      sector = "BensIndustriais";
      subsector = "Motores, Compressores e Geradores";
      shares = 4_197_297_600;
      break;
    case "ITUB4":
    case "ITUB3":
      name = "Itaú Unibanco Holding S.A.";
      sector = "Financeiro";
      subsector = "Bancos Comerciais e Múltiplos";
      shares = 9_804_134_442;
      break;
    case "BBAS3":
      name = "Banco do Brasil S.A.";
      sector = "Financeiro";
      subsector = "Bancos Comerciais e Múltiplos";
      shares = 5_737_928_828;
      break;
    case "TAEE11":
      name = "Transmissora Aliança de Energia Elétrica S.A. - Taesa";
      sector = "UtilidadePublica";
      subsector = "Transmissão de Energia Elétrica";
      shares = 1_033_496_721;
      break;
    case "EGIE3":
      name = "Engie Brasil Energia S.A.";
      sector = "UtilidadePublica";
      subsector = "Geração de Energia Elétrica";
      shares = 815_927_740;
      break;
    case "AZZA3":
    case "ARZZ3":
    case "SOMA3":
      name = "Azzas 2154 S.A.";
      sector = "ConsumoCiclico";
      subsector = "Varejo de Moda e Calçados";
      shares = 205_800_000;
      break;
    case "HGLG11":
      name = "CSHG Logística Fundo de Investimento Imobiliário";
      sector = "Imobiliario";
      subsector = "Logística e Galpões Industriais";
      shares = 33_787_545;
      break;
    case "KNCR11":
      name = "Kinea Rendimentos Imobiliários FII";
      sector = "Imobiliario";
      subsector = "Títulos e Valores Mobiliários (Papel)";
      shares = 57_094_570;
      break;
    case "MXRF11":
      name = "Maxi Renda FII";
      sector = "Imobiliario";
      subsector = "Híbrido e CRIs";
      shares = 334_655_903;
      break;
    case "XPML11":
      name = "XP Malls FII";
      sector = "Imobiliario";
      subsector = "Shoppings";
      shares = 60_796_275;
      break;
    default: {
      const asset = findB3Asset(sym);
      if (asset) {
        name = asset.name;
        subsector = asset.sector;
        shares = 500_000_000;
        if (asset.assetClass === "Fii") sector = "Imobiliario";
        else if (asset.assetClass === "Etf") sector = "Financeiro";
        else if (asset.assetClass === "Bdr") sector = "TecnologiaInformacao";
        else sector = marketSectorFromName(asset.sector);
      }
      break;
    }
  }

  return {
    symbol: sym,
    name,
    cnpj: "00.000.000/0001-00",
    cvmCode: 9512,
    sector,
    subsector,
    segment: "Novo Mercado",
    totalShares: shares,
    commonShares: Math.floor(shares / 2),
    preferredShares: Math.floor(shares / 2),
    freeFloatPercent: 45.0,
    website: "https://ri.b3.com.br",
    description: `Ativo listado na B3 (${sym}) - ${name}. Cobertura de análise fundamentalista e dados de mercado.`,
  };
}
