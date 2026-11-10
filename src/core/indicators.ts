import { findB3Asset, assetClassFromSymbol } from "./b3-assets.ts";

export interface StockIndicators {
  symbol: string;
  price: number;
  // Valuation
  pl: number;
  pvp: number;
  evEbitda: number;
  evEbit: number;
  pEbitda: number;
  pEbit: number;
  pSr: number;
  pAtivo: number;
  pCapGiro: number;
  pAtivoCircLiq: number;
  dividendYield: number;
  pegRatio: number;
  // Debt & Liquidity
  dividaLiqPl: number;
  dividaLiqEbitda: number;
  dividaLiqEbit: number;
  plAtivos: number;
  passivosAtivos: number;
  liquidezCorrente: number;
  liquidezSeca: number;
  // Efficiency
  margemBruta: number;
  margemEbitda: number;
  margemEbit: number;
  margemLiquida: number;
  giroAtivos: number;
  // Profitability
  roe: number;
  roic: number;
  roa: number;
  // Growth (5Y)
  cagrReceitas5a: number;
  cagrLucros5a: number;
  // Per Share
  lpa: number;
  vpa: number;
  dpa: number;
  // Quantitative Models
  grahamFairValue: number;
  bazinCeilingPrice: number;
  piotroskiFScore: number;
  altmanZScore: number;
}

export function defaultStockIndicators(symbol: string): StockIndicators {
  const sym = symbol.trim().toUpperCase();

  switch (sym) {
    case "PETR4": {
      const lpa = 7.50;
      const vpa = 31.65;
      const dpa = 5.38;
      return {
        symbol: sym,
        price: 52.71,
        pl: 4.85,
        pvp: 1.15,
        evEbitda: 2.90,
        evEbit: 3.65,
        pEbitda: 2.10,
        pEbit: 2.65,
        pSr: 0.95,
        pAtivo: 0.45,
        pCapGiro: 12.40,
        pAtivoCircLiq: -0.85,
        dividendYield: 14.80,
        pegRatio: 0.42,
        dividaLiqPl: 0.58,
        dividaLiqEbitda: 0.85,
        dividaLiqEbit: 1.10,
        plAtivos: 0.39,
        passivosAtivos: 0.61,
        liquidezCorrente: 1.18,
        liquidezSeca: 0.92,
        margemBruta: 51.2,
        margemEbitda: 44.5,
        margemEbit: 35.8,
        margemLiquida: 19.6,
        giroAtivos: 0.48,
        roe: 24.2,
        roic: 18.9,
        roa: 9.4,
        cagrReceitas5a: 11.4,
        cagrLucros5a: 22.8,
        lpa,
        vpa,
        dpa,
        grahamFairValue: Math.sqrt(22.5 * lpa * vpa),
        bazinCeilingPrice: dpa / 0.06,
        piotroskiFScore: 7,
        altmanZScore: 2.65,
      };
    }
    case "VALE3": {
      const lpa = 10.51;
      const vpa = 42.07;
      const dpa = 5.24;
      return {
        symbol: sym,
        price: 62.02,
        pl: 5.60,
        pvp: 1.40,
        evEbitda: 3.80,
        evEbit: 4.50,
        pEbitda: 3.20,
        pEbit: 3.80,
        pSr: 1.28,
        pAtivo: 0.58,
        pCapGiro: 8.90,
        pAtivoCircLiq: -1.20,
        dividendYield: 8.90,
        pegRatio: 0.65,
        dividaLiqPl: 0.28,
        dividaLiqEbitda: 0.62,
        dividaLiqEbit: 0.75,
        plAtivos: 0.42,
        passivosAtivos: 0.58,
        liquidezCorrente: 1.42,
        liquidezSeca: 1.10,
        margemBruta: 42.0,
        margemEbitda: 38.5,
        margemEbit: 31.2,
        margemLiquida: 21.8,
        giroAtivos: 0.46,
        roe: 25.1,
        roic: 20.4,
        roa: 10.5,
        cagrReceitas5a: 7.2,
        cagrLucros5a: 14.1,
        lpa,
        vpa,
        dpa,
        grahamFairValue: Math.sqrt(22.5 * lpa * vpa),
        bazinCeilingPrice: dpa / 0.06,
        piotroskiFScore: 6,
        altmanZScore: 3.12,
      };
    }
    case "WEGE3": {
      const lpa = 1.46;
      const vpa = 5.12;
      const dpa = 0.97;
      return {
        symbol: sym,
        price: 48.36,
        pl: 35.80,
        pvp: 10.20,
        evEbitda: 24.50,
        evEbit: 27.10,
        pEbitda: 25.10,
        pEbit: 27.80,
        pSr: 6.40,
        pAtivo: 6.10,
        pCapGiro: 18.20,
        pAtivoCircLiq: 32.00,
        dividendYield: 1.85,
        pegRatio: 1.85,
        dividaLiqPl: -0.22,
        dividaLiqEbitda: -0.45,
        dividaLiqEbit: -0.50,
        plAtivos: 0.60,
        passivosAtivos: 0.40,
        liquidezCorrente: 1.95,
        liquidezSeca: 1.38,
        margemBruta: 33.8,
        margemEbitda: 22.4,
        margemEbit: 20.1,
        margemLiquida: 17.8,
        giroAtivos: 0.95,
        roe: 30.5,
        roic: 28.2,
        roa: 18.3,
        cagrReceitas5a: 19.8,
        cagrLucros5a: 24.5,
        lpa,
        vpa,
        dpa,
        grahamFairValue: Math.sqrt(22.5 * lpa * vpa),
        bazinCeilingPrice: dpa / 0.06,
        piotroskiFScore: 8,
        altmanZScore: 5.80,
      };
    }
    case "ITUB4": {
      const lpa = 3.84;
      const vpa = 19.54;
      const dpa = 2.46;
      return {
        symbol: sym,
        price: 38.69,
        pl: 8.90,
        pvp: 1.75,
        evEbitda: 0.0,
        evEbit: 0.0,
        pEbitda: 0.0,
        pEbit: 0.0,
        pSr: 2.10,
        pAtivo: 0.14,
        pCapGiro: 0.0,
        pAtivoCircLiq: 0.0,
        dividendYield: 7.20,
        pegRatio: 0.78,
        dividaLiqPl: 0.0,
        dividaLiqEbitda: 0.0,
        dividaLiqEbit: 0.0,
        plAtivos: 0.08,
        passivosAtivos: 0.92,
        liquidezCorrente: 1.0,
        liquidezSeca: 1.0,
        margemBruta: 48.0,
        margemEbitda: 35.0,
        margemEbit: 32.0,
        margemLiquida: 22.5,
        giroAtivos: 0.07,
        roe: 21.4,
        roic: 21.4,
        roa: 1.5,
        cagrReceitas5a: 10.5,
        cagrLucros5a: 13.8,
        lpa,
        vpa,
        dpa,
        grahamFairValue: Math.sqrt(22.5 * lpa * vpa),
        bazinCeilingPrice: dpa / 0.06,
        piotroskiFScore: 8,
        altmanZScore: 0.0,
      };
    }
    case "BBAS3": {
      const lpa = 4.02;
      const vpa = 23.50;
      const dpa = 1.72;
      return {
        symbol: sym,
        price: 16.47,
        pl: 4.10,
        pvp: 0.70,
        evEbitda: 0.0,
        evEbit: 0.0,
        pEbitda: 0.0,
        pEbit: 0.0,
        pSr: 1.15,
        pAtivo: 0.07,
        pCapGiro: 0.0,
        pAtivoCircLiq: 0.0,
        dividendYield: 10.45,
        pegRatio: 0.38,
        dividaLiqPl: 0.0,
        dividaLiqEbitda: 0.0,
        dividaLiqEbit: 0.0,
        plAtivos: 0.08,
        passivosAtivos: 0.92,
        liquidezCorrente: 1.0,
        liquidezSeca: 1.0,
        margemBruta: 44.0,
        margemEbitda: 33.0,
        margemEbit: 30.0,
        margemLiquida: 21.0,
        giroAtivos: 0.06,
        roe: 21.8,
        roic: 21.8,
        roa: 1.6,
        cagrReceitas5a: 12.1,
        cagrLucros5a: 18.2,
        lpa,
        vpa,
        dpa,
        grahamFairValue: Math.sqrt(22.5 * lpa * vpa),
        bazinCeilingPrice: dpa / 0.06,
        piotroskiFScore: 7,
        altmanZScore: 0.0,
      };
    }
    default: {
      const catalogAsset = findB3Asset(sym);
      const price = catalogAsset ? catalogAsset.basePrice : 25.0;
      const isFii =
        catalogAsset?.assetClass === "Fii" ||
        (!catalogAsset && assetClassFromSymbol(sym) === "Fii");

      if (isFii) {
        const pl = 9.8;
        const pvp = 0.98;
        const dy = 10.80;
        const dpa = price * 0.108;
        const vpa = price / pvp;
        const lpa = price / pl;
        return {
          symbol: sym,
          price,
          pl,
          pvp,
          evEbitda: 0.0,
          evEbit: 0.0,
          pEbitda: 0.0,
          pEbit: 0.0,
          pSr: 0.0,
          pAtivo: 0.95,
          pCapGiro: 0.0,
          pAtivoCircLiq: 0.0,
          dividendYield: dy,
          pegRatio: 1.0,
          dividaLiqPl: 0.05,
          dividaLiqEbitda: 0.0,
          dividaLiqEbit: 0.0,
          plAtivos: 0.92,
          passivosAtivos: 0.08,
          liquidezCorrente: 2.1,
          liquidezSeca: 2.1,
          margemBruta: 88.0,
          margemEbitda: 82.0,
          margemEbit: 82.0,
          margemLiquida: 80.0,
          giroAtivos: 0.12,
          roe: 11.8,
          roic: 11.2,
          roa: 10.5,
          cagrReceitas5a: 8.5,
          cagrLucros5a: 9.0,
          lpa,
          vpa,
          dpa,
          grahamFairValue: Math.sqrt(22.5 * lpa * vpa),
          bazinCeilingPrice: dpa / 0.06,
          piotroskiFScore: 7,
          altmanZScore: 0.0,
        };
      }

      const pl = 8.5;
      const pvp = 1.35;
      const dy = 6.80;
      const dpa = price * 0.068;
      const vpa = price / pvp;
      const lpa = price / pl;
      return {
        symbol: sym,
        price,
        pl,
        pvp,
        evEbitda: 5.8,
        evEbit: 6.9,
        pEbitda: 4.9,
        pEbit: 5.8,
        pSr: 1.4,
        pAtivo: 0.65,
        pCapGiro: 7.2,
        pAtivoCircLiq: 1.8,
        dividendYield: dy,
        pegRatio: 0.85,
        dividaLiqPl: 0.45,
        dividaLiqEbitda: 1.25,
        dividaLiqEbit: 1.55,
        plAtivos: 0.48,
        passivosAtivos: 0.52,
        liquidezCorrente: 1.55,
        liquidezSeca: 1.20,
        margemBruta: 34.0,
        margemEbitda: 24.0,
        margemEbit: 19.0,
        margemLiquida: 14.5,
        giroAtivos: 0.62,
        roe: 18.5,
        roic: 15.2,
        roa: 8.9,
        cagrReceitas5a: 10.2,
        cagrLucros5a: 12.4,
        lpa,
        vpa,
        dpa,
        grahamFairValue: Math.sqrt(22.5 * lpa * vpa),
        bazinCeilingPrice: dpa / 0.06,
        piotroskiFScore: 7,
        altmanZScore: 2.85,
      };
    }
  }
}
