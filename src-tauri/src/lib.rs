use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::PathBuf,
};
use tauri::Manager;

#[cfg(target_os = "windows")]
use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: *mut std::ffi::c_void,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_command: i32,
    ) -> isize;
}

const CACHE_TTL_MINUTES: i64 = 30;
const EVENT_WINDOW_DAYS: i64 = 30;
const MAX_EVENTS: usize = 40;
const MAX_AIBOT_ITEMS_PER_SYNC: usize = 24;
const ASSISTANT_KEYRING_SERVICE: &str = "Nexus Learning";
const ASSISTANT_KEYRING_USER: &str = "cloud-model-api-key";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssistantConfigStatus {
    configured: bool,
    base_url: String,
    model: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveAssistantConfigInput {
    base_url: String,
    model: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssistantRequest {
    question: String,
    context_title: String,
    context_text: String,
    mode: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssistantAnswer {
    content: String,
    model: String,
}

#[derive(Debug, Clone)]
struct Candidate {
    source_id: String,
    external_id: String,
    kind: String,
    canonical_key: String,
    canonical_url: String,
    title: String,
    excerpt: String,
    published_at: String,
    topics: Vec<String>,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct SourceResult {
    source_id: &'static str,
    candidates: Vec<Candidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryEvidence {
    name: String,
    kind: String,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryCard {
    id: i64,
    title: String,
    kind: String,
    summary: String,
    primary_url: String,
    topics: Vec<String>,
    published_at: String,
    hot_score: i64,
    status: String,
    weekly_stars: i64,
    why_now: String,
    learning_value: String,
    source_count: usize,
    sources: Vec<String>,
    evidence: Vec<DiscoveryEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryPayload {
    events: Vec<DiscoveryCard>,
    refreshed_at: String,
    is_stale: bool,
    failed_sources: usize,
}

fn database_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join("nexus.db"))
}

fn open_database(app: &tauri::AppHandle) -> Result<Connection, String> {
    let connection = Connection::open(database_path(app)?).map_err(|error| error.to_string())?;
    connection
        .execute_batch(
            "
    PRAGMA journal_mode = WAL;
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS source_definitions (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      kind TEXT NOT NULL,
      enabled INTEGER NOT NULL DEFAULT 1,
      last_success_at TEXT
    );

    CREATE TABLE IF NOT EXISTS sync_runs (
      id INTEGER PRIMARY KEY,
      source_id TEXT NOT NULL,
      started_at TEXT NOT NULL,
      completed_at TEXT NOT NULL,
      status TEXT NOT NULL,
      fetched_count INTEGER NOT NULL,
      error_message TEXT
    );

    CREATE TABLE IF NOT EXISTS candidates (
      id INTEGER PRIMARY KEY,
      source_id TEXT NOT NULL,
      external_id TEXT NOT NULL,
      kind TEXT NOT NULL,
      canonical_key TEXT NOT NULL,
      canonical_url TEXT NOT NULL,
      title TEXT NOT NULL,
      excerpt TEXT NOT NULL DEFAULT '',
      published_at TEXT NOT NULL,
      discovered_at TEXT NOT NULL,
      topics_json TEXT NOT NULL,
      metadata_json TEXT NOT NULL,
      UNIQUE(source_id, external_id)
    );
    CREATE INDEX IF NOT EXISTS candidates_canonical_key_index ON candidates(canonical_key);
    CREATE INDEX IF NOT EXISTS candidates_published_at_index ON candidates(published_at);

    CREATE TABLE IF NOT EXISTS repo_snapshots (
      repo_key TEXT NOT NULL,
      observed_at TEXT NOT NULL,
      stars INTEGER NOT NULL,
      forks INTEGER NOT NULL,
      pushed_at TEXT,
      release_tag TEXT,
      PRIMARY KEY(repo_key, observed_at)
    );

    CREATE TABLE IF NOT EXISTS events (
      id INTEGER PRIMARY KEY,
      canonical_key TEXT NOT NULL UNIQUE,
      title TEXT NOT NULL,
      kind TEXT NOT NULL,
      summary TEXT NOT NULL DEFAULT '',
      primary_url TEXT NOT NULL,
      topics_json TEXT NOT NULL,
      published_at TEXT NOT NULL,
      hot_score INTEGER NOT NULL,
      status TEXT NOT NULL,
      weekly_stars INTEGER NOT NULL DEFAULT 0,
      why_now TEXT NOT NULL,
      learning_value TEXT NOT NULL,
      source_count INTEGER NOT NULL,
      first_seen_at TEXT NOT NULL,
      last_seen_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS event_evidence (
      event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
      source_id TEXT NOT NULL,
      kind TEXT NOT NULL,
      url TEXT NOT NULL,
      PRIMARY KEY(event_id, source_id, url)
    );

    CREATE TABLE IF NOT EXISTS discovery_metadata (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );
    ",
        )
        .map_err(|error| error.to_string())?;
    let _ = connection.execute(
        "ALTER TABLE events ADD COLUMN summary TEXT NOT NULL DEFAULT ''",
        [],
    );
    let _ = connection.execute(
        "ALTER TABLE events ADD COLUMN weekly_stars INTEGER NOT NULL DEFAULT 0",
        [],
    );
    Ok(connection)
}

fn topic_names(value: &str) -> Vec<String> {
    let normalized = value.to_lowercase();
    let definitions: [(&str, &[&str]); 12] = [
        (
            "Agent",
            &["agent", "multi-agent", "agentic", "orchestration"],
        ),
        (
            "RAG",
            &["rag", "retrieval", "rerank", "embedding", "vector search"],
        ),
        (
            "MCP",
            &["mcp", "model context protocol", "tool calling", "skills"],
        ),
        (
            "评估",
            &["eval", "evaluation", "benchmark", "observability"],
        ),
        ("推理", &["inference", "reasoning", "quantization", "vllm"]),
        (
            "模型",
            &[
                "llm",
                "gpt",
                "language model",
                "transformer",
                "fine-tun",
                "post-training",
            ],
        ),
        (
            "生成式媒体",
            &[
                "diffusion",
                "image generation",
                "video generation",
                "text-to-image",
                "text-to-video",
                "stable diffusion",
            ],
        ),
        (
            "多模态 / 视觉",
            &[
                "computer vision",
                "vision-language",
                "vision language",
                "multimodal",
                "ocr",
            ],
        ),
        (
            "语音",
            &["speech", "text-to-speech", "tts", "asr", "voice ai"],
        ),
        (
            "机器学习",
            &[
                "machine learning",
                "deep learning",
                "neural network",
                "reinforcement learning",
            ],
        ),
        (
            "AI 编程",
            &[
                "code agent",
                "coding agent",
                "code generation",
                "ai coding",
                "coding assistant",
            ],
        ),
        (
            "本地 AI",
            &["local llm", "ollama", "llama.cpp", "on-device", "edge ai"],
        ),
    ];
    definitions
        .into_iter()
        .filter_map(|(topic, keywords)| {
            keywords
                .iter()
                .any(|keyword| normalized.contains(keyword))
                .then(|| topic.to_string())
        })
        .collect()
}

fn aibot_topics(title: &str, excerpt: &str) -> Vec<String> {
    let topics = topic_names(&format!("{title} {excerpt}"));
    if topics.is_empty() {
        vec!["AI 综合".to_string()]
    } else {
        topics
    }
}

fn decode_html_entities(value: &str) -> String {
    let mut decoded = value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#8211;", "–")
        .replace("&#8212;", "—")
        .replace("&#8216;", "‘")
        .replace("&#8217;", "’");
    while let Some(start) = decoded.find("&#") {
        let Some(end_offset) = decoded[start..].find(';') else {
            break;
        };
        let end = start + end_offset;
        let number = &decoded[start + 2..end];
        let character = number
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|value| value.to_string());
        let Some(character) = character else {
            break;
        };
        decoded.replace_range(start..=end, &character);
    }
    decoded
}

fn html_text(value: &str) -> String {
    let mut text = String::new();
    let mut inside_tag = false;
    for character in value.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                text.push(' ');
            }
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }
    decode_html_entities(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn html_attribute(value: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=\"");
    let start = value.find(&prefix)? + prefix.len();
    let end = value[start..].find('"')? + start;
    Some(decode_html_entities(&value[start..end]))
}

fn aibot_candidate(
    source_id: &str,
    kind: &str,
    url: String,
    title: String,
    excerpt: String,
) -> Candidate {
    let topics = aibot_topics(&title, &excerpt);
    Candidate {
        source_id: source_id.to_string(),
        external_id: url.clone(),
        kind: kind.to_string(),
        canonical_key: canonical_key_for_url(&url, &format!("{source_id}:{title}")),
        canonical_url: url,
        title,
        excerpt,
        published_at: Utc::now().to_rfc3339(),
        topics,
        metadata: json!({ "source": "ai-bot" }),
    }
}

fn parse_aibot_daily_news(document: &str) -> Vec<Candidate> {
    document
        .split("class=\"news-item")
        .skip(1)
        .take(MAX_AIBOT_ITEMS_PER_SYNC)
        .filter_map(|fragment| {
            let heading = fragment.find("<h2")?;
            let anchor = fragment[heading..].find("<a ")? + heading;
            let open_end = fragment[anchor..].find('>')? + anchor;
            let close_end = fragment[open_end + 1..].find("</a>")? + open_end + 1;
            let url = html_attribute(&fragment[anchor..open_end], "href")?;
            let title = html_text(&fragment[open_end + 1..close_end]);
            if url.is_empty() || title.is_empty() {
                return None;
            }
            let excerpt = fragment[close_end..]
                .find("<p")
                .and_then(|paragraph| {
                    let start = close_end + paragraph;
                    let content_start = fragment[start..].find('>')? + start + 1;
                    let content_end = fragment[content_start..].find("</p>")? + content_start;
                    Some(html_text(&fragment[content_start..content_end]))
                })
                .unwrap_or_else(|| "AI工具集每日 AI 资讯条目。".to_string());
            Some(aibot_candidate(
                "aibot-daily",
                "technology",
                url,
                title,
                excerpt,
            ))
        })
        .collect()
}

fn parse_aibot_latest_projects(document: &str) -> Vec<Candidate> {
    document
        .split("<div class=\"list-grid")
        .skip(1)
        .take(MAX_AIBOT_ITEMS_PER_SYNC)
        .filter_map(|fragment| {
            let title_class = fragment.find("class=\"list-title")?;
            let anchor = fragment[..title_class].rfind("<a ")?;
            let open_end = fragment[anchor..].find('>')? + anchor;
            let close_end = fragment[open_end + 1..].find("</a>")? + open_end + 1;
            let url = html_attribute(&fragment[anchor..open_end], "href")?;
            let title = html_text(&fragment[open_end + 1..close_end]);
            if url.is_empty() || title.is_empty() {
                return None;
            }
            let excerpt = fragment[close_end..]
                .find("class=\"list-desc")
                .and_then(|description| {
                    let start = close_end + description;
                    let content_start = fragment[start..].find('>')? + start + 1;
                    let content_end = fragment[content_start..].find("</div>")? + content_start;
                    Some(html_text(&fragment[content_start..content_end]))
                })
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "AI工具集收录的最新 AI 项目。".to_string());
            Some(aibot_candidate(
                "aibot-projects",
                "project",
                url,
                title,
                excerpt,
            ))
        })
        .collect()
}

fn github_key_from_url(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    let path = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))?;
    let mut segments = path.split('/');
    let owner = segments.next()?.trim();
    let repository = segments.next()?.trim_end_matches(".git");
    (!owner.is_empty() && !repository.is_empty()).then(|| {
        format!(
            "github:{}/{}",
            owner.to_lowercase(),
            repository.to_lowercase()
        )
    })
}

fn canonical_key_for_url(value: &str, fallback: &str) -> String {
    github_key_from_url(value).unwrap_or_else(|| {
        let without_fragment = value.split('#').next().unwrap_or(value);
        let without_query = without_fragment
            .split('?')
            .next()
            .unwrap_or(without_fragment);
        if without_query.is_empty() {
            fallback.to_string()
        } else {
            format!("url:{}", without_query.to_lowercase())
        }
    })
}

fn parse_timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn github_queries() -> [&'static str; 11] {
    [
        "agent created:>=DATE stars:>=30 fork:false archived:false",
        "rag created:>=DATE stars:>=25 fork:false archived:false",
        "mcp created:>=DATE stars:>=20 fork:false archived:false",
        "model context protocol created:>=DATE stars:>=20 fork:false archived:false",
        "llm eval created:>=DATE stars:>=20 fork:false archived:false",
        "inference created:>=DATE stars:>=30 fork:false archived:false",
        r#""generative ai" created:>=DATE stars:>=25 fork:false archived:false"#,
        r#""machine learning" created:>=DATE stars:>=30 fork:false archived:false"#,
        r#""computer vision" created:>=DATE stars:>=25 fork:false archived:false"#,
        "speech created:>=DATE stars:>=25 fork:false archived:false",
        r#""local llm" created:>=DATE stars:>=25 fork:false archived:false"#,
    ]
}

fn is_quality_project(title: &str, description: &str, stars: i64, forks: i64) -> bool {
    let normalized = format!("{title} {description}").to_lowercase();
    let excluded_terms = [
        "free api",
        "free keys",
        "api key pool",
    ];
    stars >= 20
        && forks >= 2
        && description.chars().count() >= 36
        && !excluded_terms.iter().any(|term| normalized.contains(term))
}

fn is_eligible_event_candidate(candidate: &Candidate) -> bool {
    if candidate.source_id == "aibot-projects" {
        return true;
    }
    if candidate.kind != "project" && candidate.kind != "trend" {
        return true;
    }
    let stars = candidate
        .metadata
        .get("stars")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let forks = candidate
        .metadata
        .get("forks")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    is_quality_project(&candidate.title, &candidate.excerpt, stars, forks)
}

async fn fetch_github_candidates(client: &Client) -> Result<SourceResult, String> {
    let created_after = (Utc::now() - Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();
    let mut candidates = Vec::new();
    let mut seen_repositories = HashSet::new();

    for query_template in github_queries() {
        let query = query_template.replace("DATE", &created_after);
        let response = client
            .get("https://api.github.com/search/repositories")
            .query(&[
                ("q", query.as_str()),
                ("sort", "stars"),
                ("order", "desc"),
                ("per_page", "20"),
            ])
            .send()
            .await
            .map_err(|error| format!("GitHub 请求失败: {error}"))?
            .error_for_status()
            .map_err(|error| format!("GitHub 返回错误: {error}"))?;
        let payload: Value = response
            .json()
            .await
            .map_err(|error| format!("GitHub 响应解析失败: {error}"))?;
        let Some(items) = payload.get("items").and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            let Some(repository_id) = item.get("id").and_then(Value::as_i64) else {
                continue;
            };
            if !seen_repositories.insert(repository_id) {
                continue;
            }
            let Some(full_name) = item.get("full_name").and_then(Value::as_str) else {
                continue;
            };
            let Some(url) = item.get("html_url").and_then(Value::as_str) else {
                continue;
            };
            let description = item
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let topics = topic_names(&format!("{full_name} {description}"));
            if topics.is_empty() {
                continue;
            }
            let published_at = item
                .get("pushed_at")
                .and_then(Value::as_str)
                .or_else(|| item.get("created_at").and_then(Value::as_str))
                .map(str::to_string)
                .unwrap_or_else(|| Utc::now().to_rfc3339());
            let stars = item
                .get("stargazers_count")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let forks = item
                .get("forks_count")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            if !is_quality_project(full_name, description, stars, forks) {
                continue;
            }
            candidates.push(Candidate {
                source_id: "github-search".to_string(),
                external_id: repository_id.to_string(),
                kind: "project".to_string(),
                canonical_key: format!("github:{}", full_name.to_lowercase()),
                canonical_url: url.to_string(),
                title: full_name.to_string(),
                excerpt: description.to_string(),
                published_at,
                topics,
                metadata: json!({
                  "stars": stars,
                  "forks": forks,
                  "pushed_at": item.get("pushed_at").and_then(Value::as_str),
                  "language": item.get("language").and_then(Value::as_str),
                  "full_name": full_name,
                }),
            });
        }
    }
    Ok(SourceResult {
        source_id: "github-search",
        candidates,
    })
}

fn parse_weekly_stars(article: &str) -> i64 {
    let Some(prefix) = article.split("stars this week").next() else {
        return 0;
    };
    let digits = prefix
        .chars()
        .rev()
        .take_while(|character| {
            character.is_ascii_digit() || *character == ',' || character.is_whitespace()
        })
        .collect::<String>();
    digits
        .chars()
        .rev()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or_default()
}

fn parse_trending_repositories(document: &str) -> Vec<(String, i64)> {
    let mut repositories = BTreeSet::new();
    for article in document.split("<article").skip(1) {
        let article = article.split("</article>").next().unwrap_or(article);
        let Some(heading_start) = article.find("<h2") else {
            continue;
        };
        let heading = &article[heading_start..];
        let Some(href_start) = heading.find("href=\"/") else {
            continue;
        };
        let repository_start = href_start + "href=\"/".len();
        let Some(repository_end) = heading[repository_start..].find('"') else {
            continue;
        };
        let repository = &heading[repository_start..repository_start + repository_end];
        if repository.split('/').count() != 2 || repository.contains('?') {
            continue;
        }
        repositories.insert((repository.to_string(), parse_weekly_stars(article)));
    }
    repositories.into_iter().collect()
}

async fn fetch_github_trending_candidates(client: &Client) -> Result<SourceResult, String> {
    let document = client
        .get("https://github.com/trending?since=weekly")
        .send()
        .await
        .map_err(|error| format!("GitHub Trending 请求失败: {error}"))?
        .error_for_status()
        .map_err(|error| format!("GitHub Trending 返回错误: {error}"))?
        .text()
        .await
        .map_err(|error| format!("GitHub Trending 响应读取失败: {error}"))?;
    let repositories = parse_trending_repositories(&document);
    let mut candidates = Vec::new();
    for (repository, weekly_stars) in repositories.into_iter().take(15) {
        let response = client
            .get(format!("https://api.github.com/repos/{repository}"))
            .send()
            .await
            .map_err(|error| format!("GitHub Trending 项目读取失败: {error}"))?
            .error_for_status()
            .map_err(|error| format!("GitHub Trending 项目返回错误: {error}"))?;
        let item: Value = response
            .json()
            .await
            .map_err(|error| format!("GitHub Trending 项目解析失败: {error}"))?;
        let description = item
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let topics = topic_names(&format!("{repository} {description}"));
        if topics.is_empty() {
            continue;
        }
        let stars = item
            .get("stargazers_count")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let forks = item
            .get("forks_count")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if !is_quality_project(&repository, description, stars, forks) {
            continue;
        }
        let Some(repository_id) = item.get("id").and_then(Value::as_i64) else {
            continue;
        };
        candidates.push(Candidate {
            source_id: "github-trending".to_string(),
            external_id: repository_id.to_string(),
            kind: "trend".to_string(),
            canonical_key: format!("github:{}", repository.to_lowercase()),
            canonical_url: format!("https://github.com/{repository}"),
            title: repository,
            excerpt: description.to_string(),
            published_at: Utc::now().to_rfc3339(),
            topics,
            metadata: json!({
              "stars": stars,
              "forks": forks,
              "weekly_stars": weekly_stars,
            }),
        });
    }
    Ok(SourceResult {
        source_id: "github-trending",
        candidates,
    })
}

async fn fetch_aibot_daily_candidates(client: &Client) -> Result<SourceResult, String> {
    let document = client
        .get("https://ai-bot.cn/daily-ai-news/")
        .send()
        .await
        .map_err(|error| format!("AI工具集每日资讯请求失败: {error}"))?
        .error_for_status()
        .map_err(|error| format!("AI工具集每日资讯返回错误: {error}"))?
        .text()
        .await
        .map_err(|error| format!("AI工具集每日资讯读取失败: {error}"))?;
    Ok(SourceResult {
        source_id: "aibot-daily",
        candidates: parse_aibot_daily_news(&document),
    })
}

async fn fetch_aibot_project_candidates(client: &Client) -> Result<SourceResult, String> {
    let document = client
        .get("https://ai-bot.cn/the-latest-ai-projects/")
        .send()
        .await
        .map_err(|error| format!("AI工具集最新项目请求失败: {error}"))?
        .error_for_status()
        .map_err(|error| format!("AI工具集最新项目返回错误: {error}"))?
        .text()
        .await
        .map_err(|error| format!("AI工具集最新项目读取失败: {error}"))?;
    Ok(SourceResult {
        source_id: "aibot-projects",
        candidates: parse_aibot_latest_projects(&document),
    })
}

async fn fetch_hacker_news_candidates(client: &Client) -> Result<SourceResult, String> {
    let since = (Utc::now() - Duration::days(7)).timestamp().to_string();
    let numeric_filter = format!("created_at_i>{since}");
    let mut candidates = Vec::new();
    let mut seen_stories = HashSet::new();
    for query in ["agent", "rag", "mcp", "llm", "inference"] {
        let response = client
            .get("https://hn.algolia.com/api/v1/search_by_date")
            .query(&[
                ("query", query),
                ("tags", "story"),
                ("numericFilters", numeric_filter.as_str()),
                ("hitsPerPage", "30"),
            ])
            .send()
            .await
            .map_err(|error| format!("Hacker News 请求失败: {error}"))?
            .error_for_status()
            .map_err(|error| format!("Hacker News 返回错误: {error}"))?;
        let payload: Value = response
            .json()
            .await
            .map_err(|error| format!("Hacker News 响应解析失败: {error}"))?;
        let Some(hits) = payload.get("hits").and_then(Value::as_array) else {
            continue;
        };
        for hit in hits {
            let Some(story_id) = hit.get("objectID").and_then(Value::as_str) else {
                continue;
            };
            if !seen_stories.insert(story_id.to_string()) {
                continue;
            }
            let title = hit.get("title").and_then(Value::as_str).unwrap_or_default();
            if title.is_empty() {
                continue;
            }
            let destination = hit
                .get("url")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={story_id}"));
            let topics = topic_names(title);
            if topics.is_empty() {
                continue;
            }
            let points = hit
                .get("points")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let comments = hit
                .get("num_comments")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            if points < 12 && comments < 8 {
                continue;
            }
            candidates.push(Candidate {
                source_id: "hacker-news".to_string(),
                external_id: story_id.to_string(),
                kind: "discussion".to_string(),
                canonical_key: canonical_key_for_url(&destination, &format!("hn:{story_id}")),
                canonical_url: destination,
                title: title.to_string(),
                excerpt: hit
                    .get("story_text")
                    .and_then(Value::as_str)
                    .filter(|text| !text.trim().is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        format!("Hacker News 上正在讨论：{title}")
                    }),
                published_at: hit
                    .get("created_at")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                topics,
                metadata: json!({ "points": points, "comments": comments, "story_id": story_id }),
            });
        }
    }
    Ok(SourceResult {
        source_id: "hacker-news",
        candidates,
    })
}

fn setup_sources(connection: &Connection) -> Result<(), String> {
    for (id, name, kind) in [
        ("github-search", "GitHub 项目趋势", "github"),
        ("github-trending", "GitHub 本周 Trending", "github"),
        ("hacker-news", "Hacker News 讨论", "community"),
        ("aibot-daily", "AI工具集每日 AI 资讯", "curated"),
        ("aibot-projects", "AI工具集最新 AI 项目", "curated"),
    ] {
        connection
            .execute(
                "INSERT INTO source_definitions (id, name, kind) VALUES (?1, ?2, ?3)
       ON CONFLICT(id) DO UPDATE SET name = excluded.name, kind = excluded.kind",
                params![id, name, kind],
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn save_source_result(
    connection: &Connection,
    result: &SourceResult,
    completed_at: &str,
) -> Result<(), String> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM candidates WHERE source_id = ?1",
            params![result.source_id],
        )
        .map_err(|error| error.to_string())?;
    for candidate in &result.candidates {
        transaction
            .execute(
                "INSERT INTO candidates (
        source_id, external_id, kind, canonical_key, canonical_url, title, excerpt,
        published_at, discovered_at, topics_json, metadata_json
      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
      ON CONFLICT(source_id, external_id) DO UPDATE SET
        kind = excluded.kind,
        canonical_key = excluded.canonical_key,
        canonical_url = excluded.canonical_url,
        title = excluded.title,
        excerpt = excluded.excerpt,
        published_at = excluded.published_at,
        topics_json = excluded.topics_json,
        metadata_json = excluded.metadata_json",
                params![
                    candidate.source_id,
                    candidate.external_id,
                    candidate.kind,
                    candidate.canonical_key,
                    candidate.canonical_url,
                    candidate.title,
                    candidate.excerpt,
                    candidate.published_at,
                    completed_at,
                    serde_json::to_string(&candidate.topics).map_err(|error| error.to_string())?,
                    serde_json::to_string(&candidate.metadata).map_err(|error| error.to_string())?,
                ],
            )
            .map_err(|error| error.to_string())?;
        if candidate.kind == "project" {
            let stars = candidate
                .metadata
                .get("stars")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let forks = candidate
                .metadata
                .get("forks")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let pushed_at = candidate.metadata.get("pushed_at").and_then(Value::as_str);
            transaction.execute(
        "INSERT OR REPLACE INTO repo_snapshots (repo_key, observed_at, stars, forks, pushed_at, release_tag)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
        params![candidate.canonical_key, completed_at, stars, forks, pushed_at],
      ).map_err(|error| error.to_string())?;
        }
    }
    transaction
        .execute(
            "INSERT INTO sync_runs (source_id, started_at, completed_at, status, fetched_count)
     VALUES (?1, ?2, ?2, 'success', ?3)",
            params![
                result.source_id,
                completed_at,
                result.candidates.len() as i64
            ],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE source_definitions SET last_success_at = ?2 WHERE id = ?1",
            params![result.source_id, completed_at],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(())
}

fn save_failed_source_run(
    connection: &Connection,
    source_id: &str,
    completed_at: &str,
    error: &str,
) -> Result<(), String> {
    connection.execute(
    "INSERT INTO sync_runs (source_id, started_at, completed_at, status, fetched_count, error_message)
     VALUES (?1, ?2, ?2, 'failed', 0, ?3)",
    params![source_id, completed_at, error],
  ).map_err(|database_error| database_error.to_string())?;
    Ok(())
}

fn previous_star_delta(connection: &Connection, key: &str, current_stars: i64) -> i64 {
    let previous = connection.query_row(
    "SELECT stars FROM repo_snapshots WHERE repo_key = ?1 ORDER BY observed_at DESC LIMIT 1 OFFSET 1",
    params![key],
    |row| row.get::<_, i64>(0),
  ).optional().ok().flatten();
    previous
        .map(|stars| (current_stars - stars).max(0))
        .unwrap_or_default()
}

fn learning_value(topics: &[String]) -> String {
    if topics.iter().any(|topic| topic == "MCP") {
        return "可结合工具发现、权限边界与 Skills 设计做一个小型 Agent 实验。".to_string();
    }
    if topics.iter().any(|topic| topic == "Agent") {
        return "与当前 Agent 工程主线直接相关，适合从工具调用、工作流或记忆机制切入实践。"
            .to_string();
    }
    if topics.iter().any(|topic| topic == "RAG") {
        return "可用于理解检索、重排和上下文构建如何影响 RAG 系统效果。".to_string();
    }
    if topics.iter().any(|topic| topic == "评估") {
        return "适合转化为一个可重复的质量、成本或延迟评估实验。".to_string();
    }
    if topics.iter().any(|topic| topic == "推理") {
        return "适合结合模型部署、量化或推理性能的工程取舍学习。".to_string();
    }
    "可先快速理解核心问题，再决定是否加入学习路径或实践清单。".to_string()
}

fn rank_event(
    connection: &Connection,
    key: &str,
    candidates: &[Candidate],
) -> (i64, String, String) {
    let now = Utc::now();
    let newest = candidates
        .iter()
        .map(|candidate| parse_timestamp(&candidate.published_at))
        .max()
        .unwrap_or(now);
    let age_hours = (now - newest).num_minutes().max(0) as f64 / 60.0;
    let freshness = (-age_hours / 120.0).exp();
    let topic_score = (candidates
        .iter()
        .flat_map(|candidate| candidate.topics.iter())
        .collect::<HashSet<_>>()
        .len() as f64
        / 3.0)
        .min(1.0);
    let source_count = candidates
        .iter()
        .map(|candidate| candidate.source_id.as_str())
        .collect::<HashSet<_>>()
        .len();
    let corroboration = ((source_count.saturating_sub(1)) as f64 / 2.0).min(1.0);
    let mut community_score: f64 = 0.0;
    let mut momentum_score: f64 = 0.0;
    let mut trending_score: f64 = 0.0;
    let mut curated_score: f64 = 0.0;
    let mut has_measured_growth = false;
    let mut has_strong_discussion = false;
    let mut has_community_signal = false;
    let mut has_curated_signal = false;
    let mut facts = Vec::new();
    for candidate in candidates {
        if candidate.kind == "discussion" {
            let points = candidate
                .metadata
                .get("points")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let comments = candidate
                .metadata
                .get("comments")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            community_score =
                community_score.max(((points as f64 + comments as f64 * 2.0) / 180.0).min(1.0));
            has_community_signal |= points >= 25 || comments >= 15;
            has_strong_discussion |= points >= 80 || comments >= 30;
            if points > 0 || comments > 0 {
                facts.push(format!("Hacker News 有 {points} points、{comments} 条评论"));
            }
        }
        if candidate.kind == "project" {
            let stars = candidate
                .metadata
                .get("stars")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let delta = previous_star_delta(connection, key, stars);
            if delta > 0 {
                momentum_score = momentum_score.max(((delta as f64 + 1.0).ln() / 7.0).min(1.0));
                facts.push(format!("最近一次快照新增 {delta} stars"));
                has_measured_growth = delta >= 20;
            }
        }
        if candidate.kind == "trend" {
            let weekly_stars = candidate
                .metadata
                .get("weekly_stars")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            if weekly_stars > 0 {
                trending_score =
                    trending_score.max(((weekly_stars as f64 + 1.0).ln() / 9.0).min(1.0));
                facts.push(format!("GitHub 本周 Trending 新增 {weekly_stars} stars"));
            }
        }
        if candidate.source_id == "aibot-daily" {
            curated_score = curated_score.max(0.18);
            has_curated_signal = true;
            facts.push("AI工具集每日 AI 资讯已收录".to_string());
        }
        if candidate.source_id == "aibot-projects" {
            curated_score = curated_score.max(0.24);
            has_curated_signal = true;
            facts.push("AI工具集最新 AI 项目已收录".to_string());
        }
    }
    if source_count > 1 {
        facts.push(format!("获得 {source_count} 个独立来源印证"));
    }
    let score = (freshness * 0.25
        + momentum_score * 0.25
        + topic_score * 0.15
        + community_score * 0.20
        + trending_score * 0.10
        + curated_score * 0.15
        + corroboration * 0.05)
        * 100.0;
    let mut score = score.round().min(100.0) as i64;
    let has_trending_signal = trending_score >= 0.50;
    let has_multi_source_evidence =
        source_count >= 2 && (has_community_signal || has_trending_signal);
    let status = if has_measured_growth
        || has_strong_discussion
        || has_trending_signal
        || has_multi_source_evidence
    {
        score = score.max(65);
        "hot"
    } else if momentum_score > 0.0 || has_community_signal || has_curated_signal {
        "watch"
    } else {
        "candidate"
    }
    .to_string();
    let why_now = if facts.is_empty() {
        "新发现：已通过项目质量与学习主题筛选，等待后续同步记录真实增长。".to_string()
    } else {
        facts.join("；")
    };
    (score, status, why_now)
}

fn rebuild_events(connection: &Connection) -> Result<(), String> {
    let cutoff = (Utc::now() - Duration::days(EVENT_WINDOW_DAYS)).to_rfc3339();
    let mut statement = connection
        .prepare(
            "SELECT source_id, external_id, kind, canonical_key, canonical_url, title, excerpt,
            published_at, topics_json, metadata_json
     FROM candidates WHERE published_at >= ?1",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![cutoff], |row| {
            let topics_json: String = row.get(8)?;
            let metadata_json: String = row.get(9)?;
            Ok(Candidate {
                source_id: row.get(0)?,
                external_id: row.get(1)?,
                kind: row.get(2)?,
                canonical_key: row.get(3)?,
                canonical_url: row.get(4)?,
                title: row.get(5)?,
                excerpt: row.get(6)?,
                published_at: row.get(7)?,
                topics: serde_json::from_str(&topics_json).unwrap_or_default(),
                metadata: serde_json::from_str(&metadata_json).unwrap_or(Value::Null),
            })
        })
        .map_err(|error| error.to_string())?;
    let mut groups: HashMap<String, Vec<Candidate>> = HashMap::new();
    for row in rows {
        let candidate = row.map_err(|error| error.to_string())?;
        if !is_eligible_event_candidate(&candidate) {
            continue;
        }
        groups
            .entry(candidate.canonical_key.clone())
            .or_default()
            .push(candidate);
    }
    drop(statement);
    connection
        .execute("DELETE FROM event_evidence", [])
        .map_err(|error| error.to_string())?;
    connection
        .execute("DELETE FROM events", [])
        .map_err(|error| error.to_string())?;

    for (key, mut candidates) in groups {
        candidates.sort_by_key(|candidate| match candidate.kind.as_str() {
            "project" => 0,
            "trend" => 1,
            "discussion" => 2,
            _ => 3,
        });
        let representative = candidates.first().expect("candidate groups are non-empty");
        let topics = candidates
            .iter()
            .flat_map(|candidate| candidate.topics.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let source_count = candidates
            .iter()
            .map(|candidate| candidate.source_id.as_str())
            .collect::<HashSet<_>>()
            .len();
        let published_at = candidates
            .iter()
            .map(|candidate| candidate.published_at.clone())
            .max()
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let (hot_score, status, why_now) = rank_event(connection, &key, &candidates);
        let weekly_stars = candidates
            .iter()
            .filter_map(|candidate| candidate.metadata.get("weekly_stars").and_then(Value::as_i64))
            .max()
            .unwrap_or_default();
        let value = learning_value(&topics);
        connection
            .execute(
                "INSERT INTO events (
        canonical_key, title, kind, primary_url, topics_json, published_at, hot_score, status,
        summary, weekly_stars, why_now, learning_value, source_count, first_seen_at, last_seen_at
      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
                params![
                    key,
                    representative.title,
                    representative.kind,
                    representative.canonical_url,
                    serde_json::to_string(&topics).map_err(|error| error.to_string())?,
                    published_at,
                    hot_score,
                    status,
                    representative.excerpt,
                    weekly_stars,
                    why_now,
                    value,
                    source_count as i64,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|error| error.to_string())?;
        let event_id = connection.last_insert_rowid();
        for candidate in &candidates {
            connection.execute(
        "INSERT OR IGNORE INTO event_evidence (event_id, source_id, kind, url) VALUES (?1, ?2, ?3, ?4)",
        params![event_id, candidate.source_id, candidate.kind, evidence_url(candidate)],
      ).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn evidence_name(source_id: &str) -> String {
    match source_id {
        "github-search" => "GitHub 项目趋势",
        "github-trending" => "GitHub 本周 Trending",
        "hacker-news" => "Hacker News 讨论",
        "aibot-daily" => "AI工具集 · 每日 AI 资讯",
        "aibot-projects" => "AI工具集 · 最新 AI 项目",
        _ => source_id,
    }
    .to_string()
}

fn evidence_url(candidate: &Candidate) -> String {
    if candidate.source_id == "hacker-news" {
        return format!(
            "https://news.ycombinator.com/item?id={}",
            candidate.external_id
        );
    }
    candidate.canonical_url.clone()
}

fn load_discovery_feed(connection: &Connection) -> Result<Vec<DiscoveryCard>, String> {
    let mut statement = connection.prepare(
    "SELECT id, title, kind, summary, primary_url, topics_json, published_at, hot_score, status,
            weekly_stars, why_now, learning_value, source_count
     FROM events
     ORDER BY CASE status WHEN 'hot' THEN 0 WHEN 'watch' THEN 1 ELSE 2 END, hot_score DESC, published_at DESC
     LIMIT ?1"
  ).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![MAX_EVENTS as i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, i64>(12)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut events = Vec::new();
    for row in rows {
        let (
            id,
            title,
            kind,
            summary,
            primary_url,
            topics_json,
            published_at,
            hot_score,
            status,
            weekly_stars,
            why_now,
            learning_value,
            source_count,
        ) = row.map_err(|error| error.to_string())?;
        let mut evidence_statement = connection.prepare(
      "SELECT source_id, kind, url FROM event_evidence WHERE event_id = ?1 ORDER BY source_id"
    ).map_err(|error| error.to_string())?;
        let evidence = evidence_statement
            .query_map(params![id], |evidence_row| {
                let source_id: String = evidence_row.get(0)?;
                Ok(DiscoveryEvidence {
                    name: evidence_name(&source_id),
                    kind: evidence_row.get(1)?,
                    url: evidence_row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        let evidence = if evidence.is_empty() {
            vec![DiscoveryEvidence {
                name: "原始项目".to_string(),
                kind: kind.clone(),
                url: primary_url.clone(),
            }]
        } else {
            evidence
        };
        let sources = evidence
            .iter()
            .map(|entry| entry.name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        events.push(DiscoveryCard {
            id,
            title,
            kind,
            summary,
            primary_url,
            topics: serde_json::from_str(&topics_json).unwrap_or_default(),
            published_at,
            hot_score,
            status,
            weekly_stars,
            why_now,
            learning_value,
            source_count: source_count as usize,
            sources,
            evidence,
        });
    }
    Ok(events)
}

fn last_refresh(connection: &Connection) -> Option<String> {
    connection
        .query_row(
            "SELECT value FROM discovery_metadata WHERE key = 'last_refresh_at'",
            [],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten()
}

fn refresh_is_fresh(connection: &Connection) -> bool {
    last_refresh(connection)
        .map(|value| Utc::now() - parse_timestamp(&value) < Duration::minutes(CACHE_TTL_MINUTES))
        .unwrap_or(false)
}

fn save_last_refresh(connection: &Connection, refreshed_at: &str) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO discovery_metadata (key, value) VALUES ('last_refresh_at', ?1)
     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![refreshed_at],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn metadata_value(connection: &Connection, key: &str) -> Result<Option<String>, String> {
    connection
        .query_row(
            "SELECT value FROM discovery_metadata WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())
}

fn save_metadata_value(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO discovery_metadata (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn safe_assistant_base_url(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.len() > 512
        || !trimmed.starts_with("https://")
        || trimmed.chars().any(|character| character.is_control() || character.is_whitespace())
    {
        return None;
    }
    let parsed = reqwest::Url::parse(trimmed).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".local") || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    Some(trimmed.to_string())
}

fn assistant_key_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(ASSISTANT_KEYRING_SERVICE, ASSISTANT_KEYRING_USER)
        .map_err(|_| "无法访问 Windows 凭据管理器。".to_string())
}

#[tauri::command]
fn get_assistant_config(app: tauri::AppHandle) -> Result<AssistantConfigStatus, String> {
    let connection = open_database(&app)?;
    let base_url = metadata_value(&connection, "assistant_base_url")?.unwrap_or_default();
    let model = metadata_value(&connection, "assistant_model")?.unwrap_or_default();
    let configured = !base_url.is_empty()
        && !model.is_empty()
        && assistant_key_entry()
            .and_then(|entry| entry.get_password().map_err(|_| "未配置密钥".to_string()))
            .is_ok();
    Ok(AssistantConfigStatus { configured, base_url, model })
}

#[tauri::command]
fn save_assistant_config(app: tauri::AppHandle, input: SaveAssistantConfigInput) -> Result<AssistantConfigStatus, String> {
    let base_url = safe_assistant_base_url(&input.base_url)
        .ok_or_else(|| "服务地址必须是有效的 HTTPS 公网地址。".to_string())?;
    let model = input.model.trim();
    if model.is_empty() || model.len() > 160 || model.chars().any(|character| character.is_control()) {
        return Err("请填写有效的模型名称。".to_string());
    }
    if input.api_key.trim().len() < 8 || input.api_key.len() > 1024 {
        return Err("请填写有效的 API Key。".to_string());
    }
    assistant_key_entry()?
        .set_password(input.api_key.trim())
        .map_err(|_| "无法将 API Key 保存到 Windows 凭据管理器。".to_string())?;
    let connection = open_database(&app)?;
    save_metadata_value(&connection, "assistant_base_url", &base_url)?;
    save_metadata_value(&connection, "assistant_model", model)?;
    Ok(AssistantConfigStatus { configured: true, base_url, model: model.to_string() })
}

#[tauri::command]
async fn refresh_discoveries(
    app: tauri::AppHandle,
    force: bool,
) -> Result<DiscoveryPayload, String> {
    let connection = open_database(&app)?;
    setup_sources(&connection)?;
    if !force && refresh_is_fresh(&connection) {
        return Ok(DiscoveryPayload {
            events: load_discovery_feed(&connection)?,
            refreshed_at: last_refresh(&connection).unwrap_or_else(|| Utc::now().to_rfc3339()),
            is_stale: false,
            failed_sources: 0,
        });
    }
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Nexus-Learning-Discovery/0.1")
        .build()
        .map_err(|error| error.to_string())?;
    let refreshed_at = Utc::now().to_rfc3339();
    let mut failed_sources = 0;
    let mut successful_sources = 0;
    match fetch_github_candidates(&client).await {
        Ok(result) => {
            save_source_result(&connection, &result, &refreshed_at)?;
            successful_sources += 1;
        }
        Err(error) => {
            failed_sources += 1;
            save_failed_source_run(&connection, "github-search", &refreshed_at, &error)?;
            log::warn!("GitHub discovery failed: {error}");
        }
    }
    match fetch_github_trending_candidates(&client).await {
        Ok(result) => {
            save_source_result(&connection, &result, &refreshed_at)?;
            successful_sources += 1;
        }
        Err(error) => {
            failed_sources += 1;
            save_failed_source_run(&connection, "github-trending", &refreshed_at, &error)?;
            log::warn!("GitHub Trending discovery failed: {error}");
        }
    }
    match fetch_aibot_daily_candidates(&client).await {
        Ok(result) => {
            save_source_result(&connection, &result, &refreshed_at)?;
            successful_sources += 1;
        }
        Err(error) => {
            failed_sources += 1;
            save_failed_source_run(&connection, "aibot-daily", &refreshed_at, &error)?;
            log::warn!("AI-Bot daily discovery failed: {error}");
        }
    }
    match fetch_aibot_project_candidates(&client).await {
        Ok(result) => {
            save_source_result(&connection, &result, &refreshed_at)?;
            successful_sources += 1;
        }
        Err(error) => {
            failed_sources += 1;
            save_failed_source_run(&connection, "aibot-projects", &refreshed_at, &error)?;
            log::warn!("AI-Bot project discovery failed: {error}");
        }
    }
    match fetch_hacker_news_candidates(&client).await {
        Ok(result) => {
            save_source_result(&connection, &result, &refreshed_at)?;
            successful_sources += 1;
        }
        Err(error) => {
            failed_sources += 1;
            save_failed_source_run(&connection, "hacker-news", &refreshed_at, &error)?;
            log::warn!("Hacker News discovery failed: {error}");
        }
    }
    if successful_sources == 0 && load_discovery_feed(&connection)?.is_empty() {
        return Err("暂时无法获取趋势信号，请检查网络后重试。".to_string());
    }
    rebuild_events(&connection)?;
    save_last_refresh(&connection, &refreshed_at)?;
    Ok(DiscoveryPayload {
        events: load_discovery_feed(&connection)?,
        refreshed_at,
        is_stale: successful_sources == 0,
        failed_sources,
    })
}

fn is_safe_external_url(url: &str) -> bool {
    url.len() <= 2_048
        && url.starts_with("https://")
        && !url.chars().any(|character| character.is_control() || character.is_whitespace())
}

#[tauri::command]
async fn ask_assistant(app: tauri::AppHandle, input: AssistantRequest) -> Result<AssistantAnswer, String> {
    let question = input.question.trim();
    if question.is_empty() || question.len() > 8_000 {
        return Err("请输入不超过 8000 个字符的问题。".to_string());
    }
    let connection = open_database(&app)?;
    let base_url = metadata_value(&connection, "assistant_base_url")?
        .and_then(|value| safe_assistant_base_url(&value))
        .ok_or_else(|| "请先在设置中配置 AI 模型服务。".to_string())?;
    let model = metadata_value(&connection, "assistant_model")?
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "请先在设置中填写模型名称。".to_string())?;
    let api_key = assistant_key_entry()?
        .get_password()
        .map_err(|_| "未找到 API Key，请重新保存模型配置。".to_string())?;
    let context_title = input.context_title.trim().chars().take(300).collect::<String>();
    let context_text = input.context_text.trim().chars().take(8_000).collect::<String>();
    let mode = match input.mode.as_str() {
        "quiz" => "出一道能检验理解的小题；先只给题目与答题要求，等待用户回答后再反馈。",
        "review" => "审阅用户的理解或方案，先肯定正确部分，再指出一个最重要的改进点与可执行下一步。",
        _ => "用中文分层解释：先给直觉，再给关键机制、一个小例子和一个可执行练习。避免编造来源或事实。",
    };
    let system = format!(
        "你是 Nexus 的 AI 学习导师，服务于已有 RAG/Agent 经验的个人学习者。{mode} 只依据用户提供的上下文和通用稳定知识回答；不确定时明确说明。回答要具体、紧凑，使用 Markdown。"
    );
    let user = format!(
        "当前学习主题：{context_title}\n\n学习上下文：\n{context_text}\n\n用户请求：\n{question}"
    );
    let endpoint = format!("{base_url}/chat/completions");
    let response = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|_| "无法初始化 AI 请求。".to_string())?
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ],
            "temperature": 0.35
        }))
        .send()
        .await
        .map_err(|_| "无法连接模型服务，请检查服务地址与网络。".to_string())?
        .error_for_status()
        .map_err(|_| "模型服务拒绝了请求，请检查模型名称、API Key 与账户额度。".to_string())?;
    let payload: Value = response
        .json()
        .await
        .map_err(|_| "模型服务返回了无法识别的响应。".to_string())?;
    let content = payload
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "模型没有返回可用内容。".to_string())?;
    Ok(AssistantAnswer { content: content.to_string(), model })
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    if !is_safe_external_url(&url) {
        return Err("只能打开有效的 HTTPS 来源链接。".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(once(0)).collect();
        let target: Vec<u16> = OsStr::new(&url).encode_wide().chain(once(0)).collect();
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                operation.as_ptr(),
                target.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        };
        if result <= 32 {
            return Err("无法调用系统默认浏览器打开该链接。".to_string());
        }
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = url;
        Err("当前平台暂未配置系统浏览器打开能力。".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            refresh_discoveries,
            open_external_url,
            get_assistant_config,
            save_assistant_config,
            ask_assistant
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_a_stable_github_key() {
        assert_eq!(
            github_key_from_url("https://github.com/OpenAI/Codex/issues/1"),
            Some("github:openai/codex".to_string())
        );
    }

    #[test]
    fn classifies_learning_topics() {
        let topics = topic_names("An MCP agent for RAG evaluation");
        assert!(topics.contains(&"MCP".to_string()));
        assert!(topics.contains(&"Agent".to_string()));
        assert!(topics.contains(&"RAG".to_string()));
        assert!(topics.contains(&"评估".to_string()));
    }

    #[test]
    fn parses_weekly_stars_from_a_trending_card() {
        let html = r#"<article><a href="/login?return_to=%2Fopenai%2Fcodex">Log in</a><h2><a href="/openai/codex">Codex</a></h2><span>1,234 stars this week</span></article>"#;
        assert_eq!(
            parse_trending_repositories(html),
            vec![("openai/codex".to_string(), 1234)]
        );
    }

    #[test]
    fn keeps_substantive_learning_resource_candidates() {
        assert!(is_quality_project(
            "example/awesome-agent-prompts",
            "A collection of useful prompt templates for developers.",
            500,
            20
        ));
        assert!(is_quality_project(
            "example/agent-runtime",
            "A durable runtime for composing observable agent workflows in production.",
            120,
            12
        ));
    }

    #[test]
    fn rejects_unsafe_external_urls() {
        assert!(is_safe_external_url("https://github.com/openai/codex"));
        assert!(!is_safe_external_url("http://github.com/openai/codex"));
        assert!(!is_safe_external_url("https://github.com/openai/codex\nnot-safe"));
    }

    #[test]
    fn parses_aibot_daily_news_cards() {
        let html = r#"<div class="news-item"><h2><a href="https://example.com/news">AI 模型发布</a></h2><p>带有可学习价值的新闻摘要。</p></div>"#;
        let candidates = parse_aibot_daily_news(html);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].title, "AI 模型发布");
        assert_eq!(candidates[0].canonical_url, "https://example.com/news");
    }

    #[test]
    fn parses_aibot_latest_project_cards() {
        let html = r#"<div class="list-grid"><a href="https://ai-bot.cn/project/" class="list-title text-lg">新 AI Agent 框架</a><div class="list-desc">适合实践工具调用与工作流的项目。</div></div>"#;
        let candidates = parse_aibot_latest_projects(html);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].title, "新 AI Agent 框架");
        assert_eq!(candidates[0].source_id, "aibot-projects");
    }
}
