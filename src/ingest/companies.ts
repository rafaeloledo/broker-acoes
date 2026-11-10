export interface Issuer {
  tickers: readonly string[];
  cnpjDigits: string;
  name: string;
}

export const ISSUERS: readonly Issuer[] = [
  {
    tickers: ["PETR3", "PETR4"],
    cnpjDigits: "33000167000101",
    name: "Petróleo Brasileiro S.A. - Petrobras",
  },
  {
    tickers: ["VALE3"],
    cnpjDigits: "33592510000154",
    name: "Vale S.A.",
  },
  {
    tickers: ["WEGE3"],
    cnpjDigits: "84429695000111",
    name: "WEG S.A.",
  },
  {
    tickers: ["ITUB3", "ITUB4"],
    cnpjDigits: "60872504000123",
    name: "Itaú Unibanco Holding S.A.",
  },
  {
    tickers: ["BBAS3"],
    cnpjDigits: "00000000000191",
    name: "Banco do Brasil S.A.",
  },
  {
    tickers: ["BBSE3"],
    cnpjDigits: "17344597000194",
    name: "BB Seguridade Participações S.A.",
  },
  {
    tickers: ["BBDC3", "BBDC4"],
    cnpjDigits: "60746948000112",
    name: "Banco Bradesco S.A.",
  },
  {
    tickers: ["TAEE3", "TAEE4", "TAEE11"],
    cnpjDigits: "07859971000130",
    name: "Transmissora Aliança de Energia Elétrica S.A. - Taesa",
  },
  {
    tickers: ["EGIE3"],
    cnpjDigits: "02474103000119",
    name: "Engie Brasil Energia S.A.",
  },
  {
    tickers: ["B3SA3"],
    cnpjDigits: "09346601000125",
    name: "B3 S.A. - Brasil, Bolsa, Balcão",
  },
  {
    tickers: ["ABEV3"],
    cnpjDigits: "07526557000100",
    name: "Ambev S.A.",
  },
  {
    tickers: ["SUZB3"],
    cnpjDigits: "16404287000155",
    name: "Suzano S.A.",
  },
  {
    tickers: ["GGBR3", "GGBR4"],
    cnpjDigits: "33611500000119",
    name: "Gerdau S.A.",
  },
  {
    tickers: ["CSNA3"],
    cnpjDigits: "33042730000104",
    name: "Companhia Siderúrgica Nacional",
  },
  {
    tickers: ["MGLU3"],
    cnpjDigits: "47960950000121",
    name: "Magazine Luiza S.A.",
  },
  {
    tickers: ["RADL3"],
    cnpjDigits: "61585865000151",
    name: "Raia Drogasil S.A.",
  },
  {
    tickers: ["KLBN3", "KLBN4", "KLBN11"],
    cnpjDigits: "89637490000145",
    name: "Klabin S.A.",
  },
  {
    tickers: ["SANB3", "SANB4", "SANB11"],
    cnpjDigits: "90400888000142",
    name: "Banco Santander (Brasil) S.A.",
  },
  {
    tickers: ["LREN3"],
    cnpjDigits: "92754738000162",
    name: "Lojas Renner S.A.",
  },
  {
    tickers: ["EMBR3"],
    cnpjDigits: "07689002000189",
    name: "Embraer S.A.",
  },
  {
    tickers: ["TIMS3"],
    cnpjDigits: "02421421000111",
    name: "TIM S.A.",
  },
  {
    tickers: ["VBBR3"],
    cnpjDigits: "34274233000102",
    name: "Vibra Energia S.A.",
  },
  {
    tickers: ["AZZA3", "ARZZ3", "SOMA3"],
    cnpjDigits: "16590234000176",
    name: "Azzas 2154 S.A.",
  },
];

export function cnpjDigits(raw: string): string {
  return raw.replace(/\D/g, "");
}

export function issuerForTicker(ticker: string): Issuer | undefined {
  const sym = ticker.trim().toUpperCase();
  return ISSUERS.find((iss) => iss.tickers.includes(sym));
}

export function tickersForCnpj(cnpj: string): readonly string[] {
  const digits = cnpjDigits(cnpj);
  if (!digits) return [];
  const iss = ISSUERS.find((i) => i.cnpjDigits === digits);
  return iss ? iss.tickers : [];
}
