use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Seek};

use b3_core::financials::cvm_codes;
use chrono::{Datelike, NaiveDate};
use zip::ZipArchive;

use crate::companies::{self, cnpj_digits};

#[derive(Debug, Clone, PartialEq)]
pub struct KeptAccount {
    pub cnpj_digits: String,
    pub year: i32,
    pub reference_date: NaiveDate,
    pub account_code: String,
    pub amount: f64,
}

fn statement_codes() -> HashSet<&'static str> {
    cvm_codes::CORE_ACCOUNTS
        .iter()
        .copied()
        .filter(|code| *code != cvm_codes::SHARES_TOTAL)
        .collect()
}

fn column_index(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|header| header.trim().eq_ignore_ascii_case(name))
}

fn parse_amount(raw: &str) -> Option<f64> {
    let trimmed = raw.trim().trim_matches('"').replace(' ', "");
    if trimmed.is_empty() {
        return None;
    }
    let normalized = if trimmed.contains(',') && trimmed.rfind(',') > trimmed.rfind('.') {
        trimmed.replace('.', "").replace(',', ".")
    } else {
        trimmed.replace(',', "")
    };
    normalized.parse::<f64>().ok()
}

fn is_latest_exerc(raw: &str) -> bool {
    let upper = raw.trim().to_uppercase();
    if upper.is_empty() {
        return true;
    }
    if upper.contains("PEN") {
        return false;
    }
    upper.contains("LTIMO")
}

fn scale_amount(amount: f64, escala: &str) -> f64 {
    if escala.to_uppercase().contains("MIL") {
        amount * 1000.0
    } else {
        amount
    }
}

pub fn filter_statement_csv(content: &str, cnpjs: &HashSet<String>) -> Vec<KeptAccount> {
    let allowed = statement_codes();
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(content.as_bytes());
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return Vec::new(),
    };
    let Some(cnpj_idx) = column_index(&headers, "CNPJ_CIA") else {
        return Vec::new();
    };
    let Some(account_idx) = column_index(&headers, "CD_CONTA") else {
        return Vec::new();
    };
    let Some(amount_idx) = column_index(&headers, "VL_CONTA") else {
        return Vec::new();
    };
    let ordem_idx = column_index(&headers, "ORDEM_EXERC");
    let escala_idx = column_index(&headers, "ESCALA_MOEDA");
    let end_idx = column_index(&headers, "DT_FIM_EXERC");
    let refer_idx = column_index(&headers, "DT_REFER");

    let mut kept = Vec::new();
    for record in reader.records().flatten() {
        let cnpj = cnpj_digits(record.get(cnpj_idx).unwrap_or(""));
        if !cnpjs.contains(&cnpj) {
            continue;
        }
        let code = record.get(account_idx).unwrap_or("").trim();
        if !allowed.contains(code) {
            continue;
        }
        if let Some(idx) = ordem_idx {
            if !is_latest_exerc(record.get(idx).unwrap_or("")) {
                continue;
            }
        }
        let Some(amount) = parse_amount(record.get(amount_idx).unwrap_or("")) else {
            continue;
        };
        let escala = escala_idx.and_then(|idx| record.get(idx)).unwrap_or("");
        let date_raw = end_idx
            .and_then(|idx| record.get(idx))
            .or_else(|| refer_idx.and_then(|idx| record.get(idx)))
            .unwrap_or("");
        let Some(reference_date) = NaiveDate::parse_from_str(date_raw.trim(), "%Y-%m-%d").ok() else {
            continue;
        };
        kept.push(KeptAccount {
            cnpj_digits: cnpj,
            year: reference_date.year(),
            reference_date,
            account_code: code.to_string(),
            amount: scale_amount(amount, escala),
        });
    }
    kept
}

pub fn filter_capital_csv(content: &str, cnpjs: &HashSet<String>) -> Vec<KeptAccount> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(content.as_bytes());
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return Vec::new(),
    };
    let Some(cnpj_idx) = column_index(&headers, "CNPJ_CIA") else {
        return Vec::new();
    };
    let total_idx = column_index(&headers, "Quantidade_Total_Acoes_Capital_Integralizado");
    let on_idx = column_index(&headers, "Quantidade_Acoes_Ordinarias_Capital_Integralizado");
    let pn_idx = column_index(&headers, "Quantidade_Acoes_Preferenciais_Capital_Integralizado");
    let refer_idx = column_index(&headers, "DT_REFER");

    let mut kept = Vec::new();
    for record in reader.records().flatten() {
        let cnpj = cnpj_digits(record.get(cnpj_idx).unwrap_or(""));
        if !cnpjs.contains(&cnpj) {
            continue;
        }
        let total = total_idx
            .and_then(|idx| record.get(idx))
            .and_then(parse_amount)
            .or_else(|| {
                let on = on_idx.and_then(|idx| record.get(idx)).and_then(parse_amount);
                let pn = pn_idx.and_then(|idx| record.get(idx)).and_then(parse_amount);
                match (on, pn) {
                    (Some(on), Some(pn)) => Some(on + pn),
                    (Some(on), None) => Some(on),
                    (None, Some(pn)) => Some(pn),
                    _ => None,
                }
            });
        let Some(shares) = total.filter(|shares| *shares > 0.0) else {
            continue;
        };
        let date_raw = refer_idx.and_then(|idx| record.get(idx)).unwrap_or("");
        let Some(reference_date) = NaiveDate::parse_from_str(date_raw.trim(), "%Y-%m-%d").ok() else {
            continue;
        };
        kept.push(KeptAccount {
            cnpj_digits: cnpj,
            year: reference_date.year(),
            reference_date,
            account_code: cvm_codes::SHARES_TOTAL.to_string(),
            amount: shares,
        });
    }
    kept
}

fn decode_cvm_text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| bytes.iter().map(|byte| char::from(*byte)).collect())
}

fn entry_kind(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    if lower.contains("composicao_capital") {
        Some("capital")
    } else if lower.contains("dre_con") || lower.contains("bpp_con") || lower.contains("dfc_mi_con") {
        Some("statement")
    } else {
        None
    }
}

pub fn extract_watchlist_accounts<R: Read + Seek>(
    reader: R,
    cnpjs: &HashSet<String>,
) -> anyhow::Result<Vec<KeptAccount>> {
    let mut archive = ZipArchive::new(reader)?;
    let mut kept = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        if !file.is_file() {
            continue;
        }
        let Some(kind) = entry_kind(file.name()) else {
            continue;
        };
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let text = decode_cvm_text(&bytes);
        let rows = match kind {
            "capital" => filter_capital_csv(&text, cnpjs),
            _ => filter_statement_csv(&text, cnpjs),
        };
        kept.extend(rows);
    }
    Ok(kept)
}

/// Ano candidato mais novo que ainda falta para algum ticker mapeado da lista.
pub fn next_dfp_year(stored: &HashMap<String, i32>, candidates: &[i32], tickers: &[String]) -> Option<i32> {
    for year in candidates {
        let missing = tickers.iter().any(|ticker| {
            companies::issuer_for_ticker(ticker).is_some()
                && stored.get(ticker).copied().unwrap_or(0) < *year
        });
        if missing {
            return Some(*year);
        }
    }
    None
}

pub fn candidate_dfp_years(today_year: i32) -> [i32; 2] {
    [today_year - 1, today_year - 2]
}

pub async fn download_dfp_accounts(
    client: &reqwest::Client,
    year: i32,
    cnpjs: &HashSet<String>,
) -> anyhow::Result<Vec<KeptAccount>> {
    let url = format!(
        "https://dados.cvm.gov.br/dados/CIA_ABERTA/DOC/DFP/DADOS/dfp_cia_aberta_{year}.zip"
    );
    let response = client.get(url).send().await?;
    if response.status().as_u16() == 404 {
        return Ok(Vec::new());
    }
    if !response.status().is_success() {
        anyhow::bail!("CVM respondeu {}", response.status());
    }
    let bytes = response.bytes().await?;
    extract_watchlist_accounts(Cursor::new(bytes.to_vec()), cnpjs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const STATEMENT: &str = "\
CNPJ_CIA;DT_REFER;VERSAO;DENOM_CIA;CD_CVM;GRUPO_DFP;MOEDA;ESCALA_MOEDA;ORDEM_EXERC;DT_INI_EXERC;DT_FIM_EXERC;CD_CONTA;DS_CONTA;VL_CONTA
33.000.167/0001-01;2024-12-31;1;PETROBRAS;9512;DRE;REAL;MIL;ÚLTIMO;2024-01-01;2024-12-31;3.11;Lucro;1000.50
33.000.167/0001-01;2024-12-31;1;PETROBRAS;9512;DRE;REAL;MIL;PENÚLTIMO;2023-01-01;2023-12-31;3.11;Lucro;900
00.000.000/0001-91;2024-12-31;1;BB;1023;DRE;REAL;MIL;ÚLTIMO;2024-01-01;2024-12-31;3.11;Lucro;50
33.000.167/0001-01;2024-12-31;1;PETROBRAS;9512;DRE;REAL;MIL;ÚLTIMO;2024-01-01;2024-12-31;3.01;Receita;99999
";

    #[test]
    fn test_statement_keeps_watchlist_core_accounts_only() {
        let cnpjs = HashSet::from([cnpj_digits("33.000.167/0001-01")]);
        let kept = filter_statement_csv(STATEMENT, &cnpjs);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].account_code, "3.11");
        assert_eq!(kept[0].year, 2024);
        assert!((kept[0].amount - 1_000_500.0).abs() < 1e-6);
    }

    #[test]
    fn test_zip_extract_and_year_skip() {
        let csv = STATEMENT;
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut cursor);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            writer
                .start_file("dfp_cia_aberta_DRE_con_2024.csv", options)
                .unwrap();
            writer.write_all(csv.as_bytes()).unwrap();
            writer.finish().unwrap();
        }
        cursor.set_position(0);
        let cnpjs = HashSet::from([cnpj_digits("33.000.167/0001-01")]);
        let kept = extract_watchlist_accounts(cursor, &cnpjs).unwrap();
        assert_eq!(kept.len(), 1);

        let stored = HashMap::from([("PETR4".to_string(), 2024), ("VALE3".to_string(), 2024)]);
        assert_eq!(next_dfp_year(&stored, &[2024, 2023], &["PETR4".into(), "VALE3".into()]), None);
        let partial = HashMap::from([("PETR4".to_string(), 2023)]);
        assert_eq!(
            next_dfp_year(&partial, &[2024, 2023], &["PETR4".into(), "HGLG11".into()]),
            Some(2024)
        );
    }

    #[test]
    fn test_latin1_ultimo_is_kept() {
        let csv = "CNPJ_CIA;DT_REFER;VERSAO;DENOM_CIA;CD_CVM;GRUPO_DFP;MOEDA;ESCALA_MOEDA;ORDEM_EXERC;DT_INI_EXERC;DT_FIM_EXERC;CD_CONTA;DS_CONTA;VL_CONTA\n\
33.000.167/0001-01;2024-12-31;1;PETROBRAS;9512;DRE;REAL;MIL;\u{da}LTIMO;2024-01-01;2024-12-31;3.11;Lucro;10\n";
        let bytes = csv.chars().map(|ch| ch as u8).collect::<Vec<u8>>();
        let text = decode_cvm_text(&bytes);
        let cnpjs = HashSet::from([cnpj_digits("33.000.167/0001-01")]);
        let kept = filter_statement_csv(&text, &cnpjs);
        assert_eq!(kept.len(), 1);
        assert!((kept[0].amount - 10_000.0).abs() < 1e-6);
    }
}
