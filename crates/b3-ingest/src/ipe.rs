use std::io::{Cursor, Read, Seek};

use b3_core::news::NewsItem;
use chrono::NaiveDate;
use zip::ZipArchive;

use crate::companies::{cnpj_digits, issuer_for_ticker};

pub fn filter_ipe_csv(content: &str, tickers: &[String]) -> Vec<NewsItem> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(content.as_bytes());
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(_) => return Vec::new(),
    };
    let Some(cnpj_idx) = column(&headers, "CNPJ_Companhia") else {
        return Vec::new();
    };
    let Some(category_idx) = column(&headers, "Categoria") else {
        return Vec::new();
    };
    let subject_idx = column(&headers, "Assunto");
    let date_idx = column(&headers, "Data_Referencia");
    let delivery_idx = column(&headers, "Data_Entrega");
    let link_idx = column(&headers, "Link_Download");
    let protocol_idx = column(&headers, "Protocolo_Entrega");

    let mut items = Vec::new();
    for record in reader.records().flatten() {
        if fold(record.get(category_idx).unwrap_or("")) != "FATO RELEVANTE" {
            continue;
        }
        let Some(ticker) = ticker_for_cnpj(record.get(cnpj_idx).unwrap_or(""), tickers) else {
            continue;
        };
        let url = link_idx.and_then(|idx| record.get(idx)).unwrap_or("").trim();
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            continue;
        }
        let title = subject_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or("Fato relevante");
        let date_raw = date_idx
            .and_then(|idx| record.get(idx))
            .filter(|value| !value.trim().is_empty())
            .or_else(|| delivery_idx.and_then(|idx| record.get(idx)))
            .unwrap_or("");
        let Some(published_at_ms) = parse_date_ms(date_raw) else {
            continue;
        };
        let external_id = protocol_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        items.push(NewsItem {
            ticker,
            title: title.to_string(),
            url: url.to_string(),
            source: "CVM".into(),
            published_at_ms,
            summary: None,
            external_id,
        });
    }
    items
}

pub fn extract_ipe_news<R: Read + Seek>(reader: R, tickers: &[String]) -> anyhow::Result<Vec<NewsItem>> {
    let mut archive = ZipArchive::new(reader)?;
    let mut items = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        if !file.is_file() || !file.name().to_lowercase().ends_with(".csv") {
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let text = decode_latin1(&bytes);
        items.extend(filter_ipe_csv(&text, tickers));
    }
    Ok(items)
}

pub async fn download_ipe_news(
    client: &reqwest::Client,
    year: i32,
    tickers: &[String],
) -> anyhow::Result<Vec<NewsItem>> {
    let url = format!("https://dados.cvm.gov.br/dados/CIA_ABERTA/DOC/IPE/DADOS/ipe_cia_aberta_{year}.zip");
    let response = client.get(url).send().await?;
    if response.status().as_u16() == 404 {
        return Ok(Vec::new());
    }
    if !response.status().is_success() {
        anyhow::bail!("CVM IPE respondeu {}", response.status());
    }
    let bytes = response.bytes().await?;
    extract_ipe_news(Cursor::new(bytes.to_vec()), tickers)
}

fn ticker_for_cnpj(cnpj: &str, tickers: &[String]) -> Option<String> {
    let digits = cnpj_digits(cnpj);
    tickers.iter().find_map(|ticker| {
        issuer_for_ticker(ticker)
            .filter(|issuer| issuer.cnpj_digits == digits)
            .map(|_| ticker.trim().to_uppercase())
    })
}

fn parse_date_ms(raw: &str) -> Option<i64> {
    let date = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").ok()?;
    Some(date.and_hms_opt(15, 0, 0)?.and_utc().timestamp_millis())
}

fn column(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|header| header.trim().eq_ignore_ascii_case(name))
}

fn fold(raw: &str) -> String {
    raw.chars()
        .filter_map(|ch| {
            Some(match ch {
                'Á' | 'À' | 'Â' | 'Ã' | 'á' | 'à' | 'â' | 'ã' => 'A',
                'É' | 'Ê' | 'é' | 'ê' => 'E',
                'Í' | 'í' => 'I',
                'Ó' | 'Ô' | 'Õ' | 'ó' | 'ô' | 'õ' => 'O',
                'Ú' | 'ú' => 'U',
                'Ç' | 'ç' => 'C',
                other if other.is_ascii() => other.to_ascii_uppercase(),
                _ => return None,
            })
        })
        .collect()
}

fn decode_latin1(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| bytes.iter().map(|byte| char::from(*byte)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "\
CNPJ_Companhia;Nome_Companhia;Codigo_CVM;Data_Referencia;Categoria;Tipo;Especie;Assunto;Data_Entrega;Tipo_Apresentacao;Protocolo_Entrega;Versao;Link_Download
60.872.504/0001-23;ITAU;19348;2025-02-05;Fato Relevante;;;Programa de Recompra;2025-02-05;AP;PROTO-1;1;https://www.rad.cvm.gov.br/doc/1
60.872.504/0001-23;ITAU;19348;2025-02-05;Comunicado ao Mercado;Outros;;;2025-02-05;AP;PROTO-2;1;https://www.rad.cvm.gov.br/doc/2
33.000.167/0001-01;PETROBRAS;9512;2025-03-01;Fato Relevante;;;Dividendo;2025-03-01;AP;PROTO-3;1;https://www.rad.cvm.gov.br/doc/3
60.872.504/0001-23;ITAU;19348;2025-04-01;Fato Relevante;;;Sem link;2025-04-01;AP;PROTO-4;1;
";

    #[test]
    fn test_ipe_keeps_watchlist_fatos_with_url() {
        let tickers = vec!["ITUB4".into(), "VALE3".into()];
        let items = filter_ipe_csv(CSV, &tickers);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].ticker, "ITUB4");
        assert_eq!(items[0].title, "Programa de Recompra");
        assert_eq!(items[0].source, "CVM");
        assert!(items[0].url.starts_with("https://"));
    }
}
