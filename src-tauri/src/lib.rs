use chrono::{DateTime, Duration, Utc};
use feed_rs::parser;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};
use tauri::Manager;

const CACHE_TTL_MINUTES: i64 = 20;
const MAX_ITEMS: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Happening {
  title: String,
  source: String,
  url: String,
  published_at: String,
  summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HappeningsPayload {
  items: Vec<Happening>,
  refreshed_at: String,
  is_stale: bool,
  failed_sources: usize,
}

#[derive(Clone, Copy)]
struct FeedSource {
  name: &'static str,
  url: &'static str,
}

const FEED_SOURCES: [FeedSource; 5] = [
  FeedSource { name: "OpenAI", url: "https://openai.com/news/rss.xml" },
  FeedSource { name: "LlamaIndex", url: "https://github.com/run-llama/llama_index/releases.atom" },
  FeedSource { name: "LangChain", url: "https://github.com/langchain-ai/langchain/releases.atom" },
  FeedSource { name: "Next.js", url: "https://github.com/vercel/next.js/releases.atom" },
  FeedSource { name: "arXiv cs.AI", url: "https://export.arxiv.org/rss/cs.AI" },
];

fn cache_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
  let directory = app.path().app_cache_dir().map_err(|error| error.to_string())?;
  fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
  Ok(directory.join("happenings.json"))
}

fn read_cache(app: &tauri::AppHandle) -> Option<HappeningsPayload> {
  let path = cache_path(app).ok()?;
  let content = fs::read_to_string(path).ok()?;
  serde_json::from_str(&content).ok()
}

fn cache_is_fresh(payload: &HappeningsPayload) -> bool {
  DateTime::parse_from_rfc3339(&payload.refreshed_at)
    .map(|timestamp| Utc::now() - timestamp.with_timezone(&Utc) < Duration::minutes(CACHE_TTL_MINUTES))
    .unwrap_or(false)
}

fn save_cache(app: &tauri::AppHandle, payload: &HappeningsPayload) -> Result<(), String> {
  let path = cache_path(app)?;
  let serialized = serde_json::to_string(payload).map_err(|error| error.to_string())?;
  fs::write(path, serialized).map_err(|error| error.to_string())
}

fn clean_text(value: &str) -> String {
  let mut output = String::new();
  let mut in_tag = false;
  for character in value.chars() {
    match character {
      '<' => in_tag = true,
      '>' => in_tag = false,
      _ if !in_tag => output.push(character),
      _ => {}
    }
  }
  output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_feed(source: FeedSource, content: &[u8]) -> Result<Vec<Happening>, String> {
  let feed = parser::parse(content).map_err(|error| error.to_string())?;
  let items = feed.entries.into_iter().filter_map(|entry| {
    let title = entry.title?.content.trim().to_string();
    let url = entry.links.iter()
      .find(|link| link.rel.as_deref().map(|relation| relation == "alternate").unwrap_or(true))
      .or_else(|| entry.links.first())?.href.clone();
    let published_at = entry.published.or(entry.updated)?.to_rfc3339();
    let summary = entry.summary
      .map(|text| clean_text(&text.content))
      .or_else(|| entry.content.and_then(|content| content.body).map(|body| clean_text(&body)))
      .unwrap_or_default();
    if title.is_empty() || url.is_empty() { return None; }
    Some(Happening { title, source: source.name.to_string(), url, published_at, summary })
  }).collect();
  Ok(items)
}

#[tauri::command]
async fn refresh_happenings(app: tauri::AppHandle, force: bool) -> Result<HappeningsPayload, String> {
  if !force {
    if let Some(cached) = read_cache(&app) {
      if cache_is_fresh(&cached) { return Ok(cached); }
    }
  }
  let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(12))
    .user_agent("Nexus-Learning-MVP/0.1")
    .build()
    .map_err(|error| error.to_string())?;
  let mut items = Vec::new();
  let mut failed_sources = 0;
  for source in FEED_SOURCES {
    match client.get(source.url).send().await.and_then(|response| response.error_for_status()) {
      Ok(response) => match response.bytes().await {
        Ok(content) => match parse_feed(source, &content) {
          Ok(mut parsed) => items.append(&mut parsed),
          Err(error) => { log::warn!("Could not parse {}: {error}", source.name); failed_sources += 1; }
        },
        Err(error) => { log::warn!("Could not read {}: {error}", source.name); failed_sources += 1; }
      },
      Err(error) => { log::warn!("Could not fetch {}: {error}", source.name); failed_sources += 1; }
    }
  }
  let mut seen_urls = HashSet::new();
  items.retain(|item| seen_urls.insert(item.url.clone()));
  items.sort_by(|left, right| right.published_at.cmp(&left.published_at));
  items.truncate(MAX_ITEMS);
  if items.is_empty() {
    if let Some(mut cached) = read_cache(&app) {
      cached.is_stale = true;
      cached.failed_sources = failed_sources;
      return Ok(cached);
    }
    return Err("暂时无法获取资讯，请检查网络后重试。".to_string());
  }
  let payload = HappeningsPayload { items, refreshed_at: Utc::now().to_rfc3339(), is_stale: false, failed_sources };
  if let Err(error) = save_cache(&app, &payload) { log::warn!("Could not cache happenings feed: {error}"); }
  Ok(payload)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![refresh_happenings])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn parses_atom_items_into_happenings() {
    let xml = br#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"><entry><title>Release note</title><link href="https://example.com/release"/><updated>2026-08-30T00:00:00Z</updated><summary>News <b>summary</b></summary></entry></feed>"#;
    let source = FeedSource { name: "Example", url: "https://example.com/feed" };
    let parsed = parse_feed(source, xml).expect("feed should parse");
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title, "Release note");
    assert_eq!(parsed[0].summary, "News summary");
  }
}
