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
    Issuer {
        tickers: &["BBSE3"],
        cnpj_digits: "17344597000194",
        name: "BB Seguridade Participações S.A.",
    },
    Issuer {
        tickers: &["BBDC3", "BBDC4"],
        cnpj_digits: "60746948000112",
        name: "Banco Bradesco S.A.",
    },
    Issuer {
        tickers: &["TAEE3", "TAEE4", "TAEE11"],
        cnpj_digits: "07859971000130",
        name: "Transmissora Aliança de Energia Elétrica S.A. - Taesa",
    },
    Issuer {
        tickers: &["EGIE3"],
        cnpj_digits: "02474103000119",
        name: "Engie Brasil Energia S.A.",
    },
    Issuer {
        tickers: &["B3SA3"],
        cnpj_digits: "09346601000125",
        name: "B3 S.A. - Brasil, Bolsa, Balcão",
    },
    Issuer {
        tickers: &["ABEV3"],
        cnpj_digits: "07526557000100",
        name: "Ambev S.A.",
    },
    Issuer {
        tickers: &["SUZB3"],
        cnpj_digits: "16404287000155",
        name: "Suzano S.A.",
    },
    Issuer {
        tickers: &["GGBR3", "GGBR4"],
        cnpj_digits: "33611500000119",
        name: "Gerdau S.A.",
    },
    Issuer {
        tickers: &["CSNA3"],
        cnpj_digits: "33042730000104",
        name: "Companhia Siderúrgica Nacional",
    },
    Issuer {
        tickers: &["MGLU3"],
        cnpj_digits: "47960950000121",
        name: "Magazine Luiza S.A.",
    },
    Issuer {
        tickers: &["RADL3"],
        cnpj_digits: "61585865000151",
        name: "Raia Drogasil S.A.",
    },
    Issuer {
        tickers: &["KLBN3", "KLBN4", "KLBN11"],
        cnpj_digits: "89637490000145",
        name: "Klabin S.A.",
    },
    Issuer {
        tickers: &["SANB3", "SANB4", "SANB11"],
        cnpj_digits: "90400888000142",
        name: "Banco Santander (Brasil) S.A.",
    },
    Issuer {
        tickers: &["LREN3"],
        cnpj_digits: "92754738000162",
        name: "Lojas Renner S.A.",
    },
    Issuer {
        tickers: &["EMBR3"],
        cnpj_digits: "07689002000189",
        name: "Embraer S.A.",
    },
    Issuer {
        tickers: &["TIMS3"],
        cnpj_digits: "02421421000111",
        name: "TIM S.A.",
    },
    Issuer {
        tickers: &["VBBR3"],
        cnpj_digits: "34274233000102",
        name: "Vibra Energia S.A.",
    },
    Issuer {
        tickers: &["AZZA3", "ARZZ3", "SOMA3"],
        cnpj_digits: "16590234000176",
        name: "Azzas 2154 S.A.",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbse3_issuer() {
        let issuer = issuer_for_ticker("bbse3").expect("BBSE3 issuer should be found");
        assert_eq!(issuer.cnpj_digits, "17344597000194");
        assert_eq!(issuer.name, "BB Seguridade Participações S.A.");
        assert_eq!(tickers_for_cnpj("17.344.597/0001-94"), vec!["BBSE3"]);
    }

    #[test]
    fn test_azza3_issuer() {
        let issuer = issuer_for_ticker("azza3").expect("AZZA3 issuer should be found");
        assert_eq!(issuer.cnpj_digits, "16590234000176");
        assert_eq!(issuer.name, "Azzas 2154 S.A.");
        assert!(tickers_for_cnpj("16.590.234/0001-76").contains(&"AZZA3"));

        // Historical tickers mapped to Azzas 2154
        assert_eq!(issuer_for_ticker("arzz3").map(|i| i.name), Some("Azzas 2154 S.A."));
        assert_eq!(issuer_for_ticker("soma3").map(|i| i.name), Some("Azzas 2154 S.A."));
    }
}
