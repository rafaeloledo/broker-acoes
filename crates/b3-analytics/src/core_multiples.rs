use b3_core::financials::cvm_codes;

/// Primeiro conjunto de indicadores. `None` significa conta ou preço ausente.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoreMultiples {
    pub lpa: Option<f64>,
    pub vpa: Option<f64>,
    pub dpa: Option<f64>,
    pub pl: Option<f64>,
    pub pvp: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub graham_fair_value: Option<f64>,
    pub bazin_ceiling_price: Option<f64>,
}

pub fn core_multiples(
    price: f64,
    lucro: Option<f64>,
    patrimonio: Option<f64>,
    dividendos: Option<f64>,
    shares: Option<f64>,
) -> CoreMultiples {
    let shares = shares.filter(|s| *s > 0.0);
    let lpa = match (lucro, shares) {
        (Some(lucro), Some(shares)) => Some(lucro / shares),
        _ => None,
    };
    let vpa = match (patrimonio, shares) {
        (Some(patrimonio), Some(shares)) => Some(patrimonio / shares),
        _ => None,
    };
    let dpa = match (dividendos, shares) {
        (Some(dividendos), Some(shares)) => Some(dividendos.abs() / shares),
        _ => None,
    };

    let pl = match lpa {
        Some(lpa) if lpa > 0.0 && price > 0.0 => Some(price / lpa),
        _ => None,
    };
    let pvp = match vpa {
        Some(vpa) if vpa > 0.0 && price > 0.0 => Some(price / vpa),
        _ => None,
    };
    let dividend_yield = match dpa {
        Some(dpa) if price > 0.0 => Some((dpa / price) * 100.0),
        _ => None,
    };
    let graham_fair_value = match (lpa, vpa) {
        (Some(lpa), Some(vpa)) if lpa > 0.0 && vpa > 0.0 => Some((22.5 * lpa * vpa).sqrt()),
        _ => None,
    };
    let bazin_ceiling_price = dpa.map(|dpa| dpa / 0.06);

    CoreMultiples {
        lpa,
        vpa,
        dpa,
        pl,
        pvp,
        dividend_yield,
        graham_fair_value,
        bazin_ceiling_price,
    }
}

fn account(accounts: &[(String, f64)], code: &str) -> Option<f64> {
    accounts
        .iter()
        .find(|(c, _)| c == code)
        .map(|(_, value)| *value)
}

pub fn multiples_from_accounts(price: f64, accounts: &[(String, f64)]) -> CoreMultiples {
    let lucro = account(accounts, cvm_codes::LUCRO_LIQUIDO_CONSOLIDADO)
        .or_else(|| account(accounts, cvm_codes::LUCRO_LIQUIDO_CONTINUADO));
    let patrimonio = account(accounts, cvm_codes::PATRIMONIO_LIQUIDO);
    let dividendos = account(accounts, cvm_codes::DIVIDENDOS_PAGOS)
        .or_else(|| account(accounts, cvm_codes::DIVIDENDOS_PAGOS_ALT));
    let shares = account(accounts, cvm_codes::SHARES_TOTAL);
    core_multiples(price, lucro, patrimonio, dividendos, shares)
}
