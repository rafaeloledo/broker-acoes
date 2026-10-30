use std::path::Path;

use b3_core::news::NewsItem;
use chrono::{DateTime, Utc};

pub const DEFAULT_NEWS_FEED: &str = "https://www.infomoney.com.br/feed/";

pub fn load_feed_urls(path: &Path) -> Vec<String> {
    if !path.exists() {
        let _ = std::fs::write(path, format!("{DEFAULT_NEWS_FEED}\n"));
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return vec![DEFAULT_NEWS_FEED.into()];
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| line.starts_with("https://") || line.starts_with("http://"))
        .map(str::to_string)
        .collect()
}

pub fn filter_rss(xml: &str, tickers: &[String]) -> Vec<NewsItem> {
    item_blocks(xml)
        .into_iter()
        .filter_map(|block| item_from_block(block, tickers))
        .collect()
}

pub async fn download_rss_news(
    client: &reqwest::Client,
    feed_url: &str,
    tickers: &[String],
) -> anyhow::Result<Vec<NewsItem>> {
    let response = client.get(feed_url).send().await?;
    if !response.status().is_success() {
        anyhow::bail!("RSS respondeu {}", response.status());
    }
    let body = response.text().await?;
    Ok(filter_rss(&body, tickers))
}

fn item_from_block(block: &str, tickers: &[String]) -> Option<NewsItem> {
    let title = tag_text(block, "title")?;
    let url = tag_text(block, "link").or_else(|| tag_text(block, "guid"))?;
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return None;
    }
    let summary = tag_text(block, "description");
    let haystack = format!("{title} {}", summary.as_deref().unwrap_or(""));
    let ticker = tickers.iter().find(|ticker| mentions(&haystack, ticker))?.trim().to_uppercase();
    let published_at_ms = tag_text(block, "pubDate").and_then(|raw| parse_pub_date(&raw))?;
    Some(NewsItem {
        ticker,
        title,
        url,
        source: "RSS".into(),
        published_at_ms,
        summary,
        external_id: None,
    })
}

fn mentions(text: &str, ticker: &str) -> bool {
    let folded = fold(text);
    let ticker = ticker.trim().to_uppercase();
    if contains_token(&folded, &ticker) {
        return true;
    }
    let phrase = extra_mention(&ticker);
    !phrase.is_empty() && folded.contains(phrase)
}

fn extra_mention(ticker: &str) -> &'static str {
    if ticker.starts_with("ITUB") {
        "ITAU"
    } else if ticker.starts_with("PETR") {
        "PETROBRAS"
    } else if ticker.starts_with("WEGE") {
        "WEG"
    } else if ticker.starts_with("BBAS") {
        "BANCO DO BRASIL"
    } else {
        ""
    }
}

fn contains_token(folded: &str, token: &str) -> bool {
    folded.match_indices(token).any(|(index, _)| {
        let before_ok = index == 0 || !folded.as_bytes()[index - 1].is_ascii_alphanumeric();
        let after = index + token.len();
        let after_ok = after >= folded.len() || !folded.as_bytes()[after].is_ascii_alphanumeric();
        before_ok && after_ok
    })
}

fn item_blocks(xml: &str) -> Vec<&str> {
    let lower = xml.to_ascii_lowercase();
    let mut blocks = Vec::new();
    let mut from = 0;
    while let Some(relative) = lower[from..].find("<item") {
        let start = from + relative;
        let Some(relative_end) = lower[start..].find("</item>") else {
            break;
        };
        let end = start + relative_end;
        blocks.push(&xml[start..end]);
        from = end + "</item>".len();
    }
    blocks
}

fn tag_text(block: &str, tag: &str) -> Option<String> {
    let lower = block.to_ascii_lowercase();
    let open = format!("<{tag}").to_ascii_lowercase();
    let start = lower.find(&open)?;
    let content_at = block[start..].find('>')? + start + 1;
    let close = format!("</{tag}>").to_ascii_lowercase();
    let end = lower[content_at..].find(&close)? + content_at;
    let raw = block[content_at..end]
        .trim()
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>")
        .trim();
    let text = decode_entities(raw);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn decode_entities(raw: &str) -> String {
    raw.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn parse_pub_date(raw: &str) -> Option<i64> {
    DateTime::parse_from_rfc2822(raw.trim())
        .ok()
        .map(|date| date.with_timezone(&Utc).timestamp_millis())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_keeps_watchlist_ticker_and_ignores_others() {
        let xml = r#"
        <rss><channel>
          <item>
            <title><![CDATA[Itaú anuncia recompra]]></title>
            <link>https://exemplo.test/itau</link>
            <description>Resultado do trimestre</description>
            <pubDate>Wed, 05 Feb 2025 12:00:00 GMT</pubDate>
          </item>
          <item>
            <title>Magazine Luiza sobe</title>
            <link>https://exemplo.test/mglu</link>
            <description>MGLU3 lidera altas</description>
            <pubDate>Wed, 05 Feb 2025 13:00:00 GMT</pubDate>
          </item>
          <item>
            <title>PETR4 paga dividendos</title>
            <link>https://exemplo.test/petr</link>
            <pubDate>Thu, 06 Feb 2025 12:00:00 GMT</pubDate>
          </item>
        </channel></rss>
        "#;
        let items = filter_rss(xml, &["ITUB4".into()]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].ticker, "ITUB4");
        assert_eq!(items[0].url, "https://exemplo.test/itau");
        assert_eq!(items[0].source, "RSS");
    }
}
