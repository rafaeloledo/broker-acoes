export type FiiSegment =
  | "Logistica"
  | "LajesCorporativas"
  | "Shoppings"
  | "PapeisCRIs"
  | "FoF"
  | "Agro"
  | "EducacionalHospitalar"
  | "Residencial"
  | "Hibrido"
  | "Outros";

export function fiiSegmentDisplayName(segment: FiiSegment): string {
  switch (segment) {
    case "Logistica":
      return "Logística e Galpões Industriais";
    case "LajesCorporativas":
      return "Lajes Corporativas (Escritórios)";
    case "Shoppings":
      return "Shopping Centers e Varejo";
    case "PapeisCRIs":
      return "Títulos e Valores Mobiliários (Papel/CRI)";
    case "FoF":
      return "Fundo de Fundos (FoF)";
    case "Agro":
      return "Fiagro / Agronegócio";
    case "EducacionalHospitalar":
      return "Educacional e Hospitalar";
    case "Residencial":
      return "Residencial e Desenvolvimento";
    case "Hibrido":
      return "Híbrido / Renda Urbana";
    case "Outros":
      return "Outros Segmentos";
  }
}

export interface FiiProperty {
  name: string;
  address: string;
  city: string;
  state: string;
  areaM2: number;
  unitsCount: number;
  tenantsCount: number;
  vacancyPercent: number;
}

export interface FiiDetails {
  symbol: string;
  segment: FiiSegment;
  mandate: string;
  manager: string;
  admin: string;
  netAssetValue: number;
  navPerShare: number;
  totalShares: number;
  physicalVacancyPercent: number | null;
  financialVacancyPercent: number | null;
  shareholdersCount: number | null;
  adminFeePercent: number | null;
  performanceFee: string;
  lastDividend: number;
  dividendYield12m: number;
  properties: FiiProperty[];
}

export function defaultFiiDetails(symbol: string): FiiDetails {
  const sym = symbol.trim().toUpperCase();
  let segment: FiiSegment = "Hibrido";
  let navPerShare = 100.0;
  let shares = 10_000_000;
  let physVac: number | null = null;
  let finVac: number | null = null;
  let shareholders: number | null = null;
  let adminFee: number | null = null;
  let div = 0.85;

  switch (sym) {
    case "HGLG11":
      segment = "Logistica";
      navPerShare = 166.43;
      shares = 45_601_745;
      physVac = 2.7;
      finVac = 3.0;
      shareholders = 608_345;
      adminFee = 0.6;
      div = 1.17;
      break;
    case "KNCR11":
      segment = "PapeisCRIs";
      navPerShare = 101.45;
      shares = 107_089_622;
      physVac = null;
      finVac = null;
      shareholders = 904_998;
      adminFee = 1.0;
      div = 1.02;
      break;
    case "MXRF11":
      segment = "PapeisCRIs";
      navPerShare = 9.85;
      shares = 567_206_273;
      physVac = null;
      finVac = null;
      shareholders = 1_509_087;
      adminFee = 0.9;
      div = 0.10;
      break;
    case "XPML11":
      segment = "Shoppings";
      navPerShare = 112.50;
      shares = 64_308_055;
      physVac = 5.1;
      finVac = null;
      shareholders = 733_101;
      adminFee = 0.75;
      div = 0.92;
      break;
    case "BTLG11":
      segment = "Logistica";
      navPerShare = 99.80;
      shares = 69_389_648;
      physVac = 1.0;
      finVac = 1.1;
      shareholders = 505_908;
      adminFee = 0.9;
      div = 0.78;
      break;
  }

  return {
    symbol: sym,
    segment,
    mandate: "Renda Ativa",
    manager: "Gestora Institucional B3",
    admin: "Administrador Fiduciário S.A.",
    netAssetValue: navPerShare * shares,
    navPerShare,
    totalShares: shares,
    physicalVacancyPercent: physVac,
    financialVacancyPercent: finVac,
    shareholdersCount: shareholders,
    adminFeePercent: adminFee,
    performanceFee: "20% sobre o que exceder 100% do CDI/IPCA+6%",
    lastDividend: div,
    dividendYield12m: ((div * 12.0) / navPerShare) * 100.0,
    properties:
      physVac != null
        ? [
            {
              name: "Centro Logístico Bandeirantes",
              address: "Rodovia dos Bandeirantes, Km 38",
              city: "Cajamar",
              state: "SP",
              areaM2: 85_400.0,
              unitsCount: 24,
              tenantsCount: 8,
              vacancyPercent: physVac,
            },
            {
              name: "Parque Industrial Betim",
              address: "Via Expressa de Betim, 1500",
              city: "Betim",
              state: "MG",
              areaM2: 62_100.0,
              unitsCount: 16,
              tenantsCount: 5,
              vacancyPercent: physVac * 0.5,
            },
          ]
        : [],
  };
}
