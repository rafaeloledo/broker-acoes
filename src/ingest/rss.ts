import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { type NewsItem } from "../core/news.ts";
import { normalize } from "../core/b3-assets.ts";

export const DEFAULT_NEWS_FEED = "https://www.infomoney.com.br/feed/";

export function loadFeedUrls(path = "news_feeds.txt"): string[] {
  try {
    if (!existsSync(path)) {
      writeFileSync(path, `${DEFAULT_NEWS_FEED}\n`, "utf-8");
      return [DEFAULT_NEWS_FEED];
    }
    const raw = readFileSync(path, "utf-8");
    const urls = raw
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter(
        (l) =>
          l.length > 0 &&
          !l.startsWith("#") &&
          (l.startsWith("http://") || l.startsWith("https://"))
      );
    return urls.length > 0 ? urls : [DEFAULT_NEWS_FEED];
  } catch {
    return [DEFAULT_NEWS_FEED];
  }
}

function cleanXmlField(raw: string): string {
  return raw
    .replace(/<!\[CDATA\[([\s\S]*?)\]\]>/gi, "$1")
    .replace(/<[^>]+>/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&#8211;|&ndash;/g, "–")
    .replace(/&#8217;|&rsquo;/g, "’")
    .replace(/\s+/g, " ")
    .trim();
}

function extractTag(block: string, tag: string): string {
  const re = new RegExp(`<${tag}(?:\\s[^>]*)?>([\\s\\S]*?)<\\/${tag}>`, "i");
  const m = block.match(re);
  return m?.[1] ? cleanXmlField(m[1]) : "";
}

export function containsToken(haystackFolded: string, tokenFolded: string): boolean {
  if (!tokenFolded) return false;
  const escaped = tokenFolded.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(`(^|[^A-Z0-9])${escaped}($|[^A-Z0-9])`);
  return re.test(haystackFolded);
}

function extraMention(haystackFolded: string, ticker: string): boolean {
  const sym = ticker.toUpperCase();
  if (sym.startsWith("ITUB")) return containsToken(haystackFolded, "ITAU");
  if (sym.startsWith("PETR")) return containsToken(haystackFolded, "PETROBRAS");
  if (sym.startsWith("WEGE")) return containsToken(haystackFolded, "WEG");
  if (sym.startsWith("BBAS")) return haystackFolded.includes("BANCO DO BRASIL");
  if (sym.startsWith("VALE")) return containsToken(haystackFolded, "VALE");
  return false;
}

export function filterRss(xml: string, tickers: readonly string[]): NewsItem[] {
  const cleanTickers = tickers.map((t) => t.trim().toUpperCase()).filter(Boolean);
  if (cleanTickers.length === 0) return [];

  const itemBlocks = xml.match(/<item(?:\s[^>]*)?>[\s\S]*?<\/item>/gi) ?? [];
  const out: NewsItem[] = [];

  for (const block of itemBlocks) {
    const title = extractTag(block, "title");
    const link = extractTag(block, "link") || extractTag(block, "guid");
    const description = extractTag(block, "description");
    const pubDate = extractTag(block, "pubDate");

    if (!title || (!link.startsWith("http://") && !link.startsWith("https://"))) {
      continue;
    }

    const parsedDate = pubDate ? Date.parse(pubDate) : NaN;
    const publishedAtMs = Number.isNaN(parsedDate) ? Date.now() : parsedDate;
    const haystack = normalize(`${title} ${description}`);

    for (const sym of cleanTickers) {
      if (containsToken(haystack, sym) || extraMention(haystack, sym)) {
        out.push({
          ticker: sym,
          title,
          url: link,
          source: "RSS",
          publishedAtMs,
          summary: description || null,
          externalId: null,
        });
      }
    }
  }

  return out;
}

export async function downloadRssNews(
  urls: readonly string[],
  tickers: readonly string[]
): Promise<NewsItem[]> {
  const all: NewsItem[] = [];
  for (const url of urls) {
    try {
      const res = await fetch(url, {
        headers: { "User-Agent": "Mozilla/5.0 (X11; Linux x86_64)" },
        signal: AbortSignal.timeout(10000),
      });
      if (!res.ok) continue;
      const xml = await res.text();
      all.push(...filterRss(xml, tickers));
    } catch {
      // Ignore individual feed failures
    }
  }
  return all;
}
