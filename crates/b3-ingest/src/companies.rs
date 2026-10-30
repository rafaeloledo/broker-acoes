/// Emissores conhecidos pelo cenário local. A chave de comparação é o CNPJ só com dígitos.
#[derive(Debug, Clone, Copy)]
pub struct Issuer {
    pub tickers: &'static [&'static str],
    pub cnpj_digits: &'static str,
    pub name: &'static str,
}

const ISSUERS: &[Issuer] = &[
    Issuer {
        tickers: &["PETR3", "PETR4"],
        cnpj_digits: "33000167000101",
        name: "Petróleo Brasileiro S.A. - Petrobras",
    },
    Issuer {
        tickers: &["VALE3"],
        cnpj_digits: "33592510000154",
        name: "Vale S.A.",
    },
    Issuer {
        tickers: &["WEGE3"],
        cnpj_digits: "84429695000111",
        name: "WEG S.A.",
    },
    Issuer {
        tickers: &["ITUB3", "ITUB4"],
        cnpj_digits: "60872504000123",
        name: "Itaú Unibanco Holding S.A.",
    },
    Issuer {
        tickers: &["BBAS3"],
        cnpj_digits: "00000000000191",
        name: "Banco do Brasil S.A.",
    },
];

pub fn cnpj_digits(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub fn issuer_for_ticker(ticker: &str) -> Option<&'static Issuer> {
    let ticker = ticker.trim().to_uppercase();
    ISSUERS.iter().find(|issuer| issuer.tickers.iter().any(|t| *t == ticker))
}

pub fn tickers_for_cnpj(cnpj: &str) -> Vec<&'static str> {
    let digits = cnpj_digits(cnpj);
    ISSUERS
        .iter()
        .find(|issuer| issuer.cnpj_digits == digits)
        .map(|issuer| issuer.tickers.to_vec())
        .unwrap_or_default()
}
