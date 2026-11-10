export const NEWS_READ_LIMIT = 8;
export const NEWS_READ_MAX_CHARS = 600;

export interface NewsItem {
  ticker: string;
  title: string;
  url: string;
  source: string;
  publishedAtMs: number;
  summary: string | null;
  externalId: string | null;
}

export interface NewsBrief {
  ticker: string;
  title: string;
  source: string;
  publishedAtMs: number;
  url: string;
  text: string;
}

export function newsForReader(
  items: readonly NewsItem[],
  tickers: readonly string[],
  limit = NEWS_READ_LIMIT,
  maxChars = NEWS_READ_MAX_CHARS
): NewsBrief[] {
  const tickerSet = new Set(tickers.map((t) => t.trim().toUpperCase()));

  const filtered = items
    .filter((item) => tickerSet.has(item.ticker.trim().toUpperCase()))
    .slice()
    .sort((a, b) => b.publishedAtMs - a.publishedAtMs)
    .slice(0, limit);

  return filtered.map((item) => {
    const summaryText = item.summary?.trim() ?? "";
    const combined = summaryText
      ? `${item.title.trim()}. ${summaryText}`
      : item.title.trim();

    const chars = Array.from(combined);
    let text = combined;
    if (chars.length > maxChars) {
      text = chars.slice(0, maxChars).join("") + "…";
    }

    return {
      ticker: item.ticker,
      title: item.title,
      source: item.source,
      publishedAtMs: item.publishedAtMs,
      url: item.url,
      text,
    };
  });
}
