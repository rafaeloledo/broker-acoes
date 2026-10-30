use serde::{Deserialize, Serialize};

/// Quantidade máxima de notícias entregues de uma vez a um leitor (briefing, alerta).
pub const NEWS_READ_LIMIT: usize = 8;
/// Tamanho máximo do texto de cada notícia entregue ao leitor.
pub const NEWS_READ_MAX_CHARS: usize = 600;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsItem {
    pub ticker: String,
    pub title: String,
    pub url: String,
    pub source: String,
    pub published_at_ms: i64,
    pub summary: Option<String>,
    pub external_id: Option<String>,
}

/// Notícia já filtrada e truncada, pronta para um prompt citar a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewsBrief {
    pub ticker: String,
    pub title: String,
    pub source: String,
    pub published_at_ms: i64,
    pub url: String,
    pub text: String,
}

/// Mais recentes primeiro, só dos tickers pedidos, no máximo `limit`, com texto cortado.
pub fn news_for_reader(items: &[NewsItem], tickers: &[&str], limit: usize, max_chars: usize) -> Vec<NewsBrief> {
    let mut matched: Vec<&NewsItem> = items
        .iter()
        .filter(|item| tickers.iter().any(|ticker| ticker.eq_ignore_ascii_case(&item.ticker)))
        .collect();
    matched.sort_by(|left, right| right.published_at_ms.cmp(&left.published_at_ms));
    matched
        .into_iter()
        .take(limit)
        .map(|item| NewsBrief {
            ticker: item.ticker.clone(),
            title: item.title.clone(),
            source: item.source.clone(),
            published_at_ms: item.published_at_ms,
            url: item.url.clone(),
            text: truncate_news_text(&item.title, item.summary.as_deref().unwrap_or(""), max_chars),
        })
        .collect()
}

fn truncate_news_text(title: &str, summary: &str, max_chars: usize) -> String {
    let combined = if summary.is_empty() {
        title.to_string()
    } else {
        format!("{title}. {summary}")
    };
    if combined.chars().count() <= max_chars {
        return combined;
    }
    let mut cut: String = combined.chars().take(max_chars).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(ticker: &str, title: &str, published_at_ms: i64) -> NewsItem {
        NewsItem {
            ticker: ticker.into(),
            title: title.into(),
            url: format!("https://exemplo.test/{ticker}/{published_at_ms}"),
            source: "Teste".into(),
            published_at_ms,
            summary: Some("resumo longo ".repeat(20)),
            external_id: None,
        }
    }

    #[test]
    fn reader_returns_newest_of_requested_ticker_and_truncates() {
        let items = vec![
            item("PETR4", "antiga", 1),
            item("VALE3", "outra", 3),
            item("PETR4", "recente", 2),
        ];
        let briefs = news_for_reader(&items, &["PETR4"], 1, 24);
        assert_eq!(briefs.len(), 1);
        assert_eq!(briefs[0].ticker, "PETR4");
        assert_eq!(briefs[0].title, "recente");
        assert!(briefs[0].text.chars().count() <= 25);
        assert!(briefs[0].text.ends_with('…'));
        assert!(briefs[0].url.contains("PETR4"));
    }
}
