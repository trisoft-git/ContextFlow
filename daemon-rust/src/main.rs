use clap::Parser;
use std::sync::{Arc, Mutex};
use std::path::Path;
mod db;
mod intelligence;
mod cli;
mod knowledge;
mod config;
mod prompts;

use db::{Database, Event};
use cli::{Cli, Commands};
use config::ConfigManager;
use intelligence::gemini::GeminiProvider;
use intelligence::openai::OpenAIProvider;
use intelligence::anthropic::AnthropicProvider;
use intelligence::IntelligenceProvider;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let db_path = Path::new(".contextflow/memory.db");

    // Ensure .contextflow exists
    if !Path::new(".contextflow").exists() {
        std::fs::create_dir_all(".contextflow").expect("Failed to create .contextflow dir");
    }

    match cli.command {
        Commands::Start => {
            start_daemon(db_path).await;
        }
        Commands::Status => {
            handle_status(db_path).await;
        }
        Commands::Summarize => {
            handle_summarize(db_path).await;
        }
        Commands::Plan => {
            handle_plan(db_path).await;
        }
        Commands::Fix => {
            handle_fix(db_path).await;
        }
        Commands::Knowledge { list, view } => {
            handle_knowledge(list, view).await;
        }
        Commands::Config { set, get } => {
            handle_config(set, get).await;
        }
    }
}

async fn handle_knowledge(list: bool, view: Option<String>) {
    use crate::knowledge::KnowledgeManager;
    let km = KnowledgeManager::new();

    if list {
        let kis = km.list_kis().expect("Failed to list KIs");
        println!("\n📚 Total Knowledge Items: {}", kis.len());
        for ki in kis {
            println!("  - {}", ki.file_name().unwrap().to_string_lossy());
        }
    } else if let Some(id) = view {
        let kis = km.list_kis().expect("Failed to list KIs");
        let target = kis.iter().find(|k| k.to_string_lossy().contains(&id));
        if let Some(path) = target {
            let content = std::fs::read_to_string(path).expect("Failed to read KI");
            println!("\n--- Knowledge Item ---\n{}\n----------------------", content);
        } else {
            println!("❌ KI not found: {}", id);
        }
    } else {
        println!("💡 Use --list to see all KIs or --view <id> to read one.");
    }
}

async fn post_event_handler(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<Database>>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<impl axum::response::IntoResponse, impl axum::response::IntoResponse> {
    use axum::http::StatusCode;

    // 1. JSON 필드 검증
    let event_type = match payload["type"].as_str() {
        Some(t) => t.to_string(),
        None => return Err((StatusCode::BAD_REQUEST, "Missing 'type' field".to_string())),
    };
    
    let content = payload["content"].as_str().unwrap_or("").to_string();
    
    // metadata 필드 파싱 및 직렬화
    let metadata = if let Some(m) = payload.get("metadata") {
        if m.is_null() {
            None
        } else {
            serde_json::to_string(m).ok()
        }
    } else {
        None
    };

    // 2. 동기 데이터베이스 쓰기 작업을 spawn_blocking으로 격리
    let db = Arc::clone(&state);
    let result = tokio::task::spawn_blocking(move || {
        let db_lock = db.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let session_id = db_lock.get_active_session_id()
            .map_err(|e| format!("DB Session Error: {}", e))?
            .unwrap_or_else(|| "default".to_string());
            
        db_lock.record_event(&Event {
            session_id,
            event_type,
            content,
            metadata,
        }).map_err(|e| format!("DB Write Error: {}", e))
    }).await;

    match result {
        Ok(Ok(())) => Ok((StatusCode::OK, "OK".to_string())),
        Ok(Err(err_msg)) => {
            eprintln!("❌ Event write error: {}", err_msg);
            Err((StatusCode::INTERNAL_SERVER_ERROR, err_msg))
        }
        Err(_) => {
            eprintln!("❌ Task execution panic");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Worker thread panicked".to_string()))
        }
    }
}

async fn start_daemon(db_path: &Path) {
    use axum::{routing::{get, post}, Router};
    use notify::{Watcher, RecursiveMode, Config};

    // 1. 포트 설정 (환경변수 CF_PORT 확인, 기본값 49152)
    let port: u16 = std::env::var("CF_PORT")
        .unwrap_or_else(|_| "49152".to_string())
        .parse()
        .expect("Invalid CF_PORT value");

    // 2. 절대 경로 확보
    let abs_db_path = std::fs::canonicalize(db_path)
        .unwrap_or_else(|_| db_path.to_path_buf());
    
    println!("🗄️  Database Path: {}", abs_db_path.display());

    let database = Database::open(&abs_db_path).expect("Failed to open database");
    let shared_state = Arc::new(Mutex::new(database));

    // File Watcher (패닉 없는 감시자 구동)
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let mut _watcher = None;
    let watcher_res = notify::RecommendedWatcher::new(move |res| {
        if let Ok(event) = res {
            let _ = tx.blocking_send(event);
        }
    }, Config::default());

    match watcher_res {
        Ok(mut w) => {
            if let Err(e) = w.watch(Path::new("."), RecursiveMode::Recursive) {
                eprintln!("⚠️ File watcher failed to watch current directory: {}", e);
            } else {
                _watcher = Some(w);
            }
        }
        Err(e) => {
            eprintln!("⚠️ Failed to initialize recommended file watcher: {}", e);
        }
    }
    
    let watcher_state = Arc::clone(&shared_state);
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            for path in event.paths {
                let path_str = path.to_string_lossy().to_string();
                if !path_str.contains("node_modules") && !path_str.contains(".git") && !path_str.contains("target") && !path_str.contains(".contextflow") {
                    println!("📝 Recording file event: {}", path_str);
                    let db_clone = Arc::clone(&watcher_state);
                    let path_str_clone = path_str.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        if let Ok(db) = db_clone.lock() {
                            let session_id = db.get_active_session_id().ok().flatten().unwrap_or_else(|| "default".into());
                            let _ = db.record_event(&Event {
                                session_id,
                                event_type: "file_change".into(),
                                content: format!("File activity: {}", path_str_clone),
                                metadata: None,
                            });
                        }
                    }).await;
                }
            }
        }
    });

    let app = Router::new()
        .route("/", get(|| async { "ContextFlow Rust Core is running!" }))
        .route("/event", post(post_event_handler))
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024))
        .with_state(shared_state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("📡 ContextFlow Core (Rust) listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn get_provider(config: &config::Config) -> Box<dyn IntelligenceProvider> {
    let provider_name = config.provider.clone().unwrap_or_else(|| "gemini".to_string());
    let model_name = config.model.clone();
    
    match provider_name.as_str() {
        "openai" => {
            let api_key = config.api_key.clone().or_else(|| std::env::var("OPENAI_API_KEY").ok()).expect("OpenAI API Key not set");
            Box::new(OpenAIProvider::new(api_key, model_name, config.base_url.clone()))
        }
        "anthropic" => {
            let api_key = config.api_key.clone().or_else(|| std::env::var("ANTHROPIC_API_KEY").ok()).expect("Anthropic API Key not set");
            Box::new(AnthropicProvider::new(api_key, model_name))
        }
        _ => {
            let api_key = config.api_key.clone().or_else(|| std::env::var("GEMINI_API_KEY").ok()).expect("Gemini API Key not set");
            Box::new(GeminiProvider::new(api_key, model_name))
        }
    }
}

async fn handle_summarize(db_path: &Path) {
    let db = Database::open(db_path).expect("Failed to open database");
    let config = ConfigManager::load();
    let provider = get_provider(&config);
    
    let events = db.get_recent_events(30).unwrap_or_default();
    let context = format_events_to_context(&events);
    
    println!("📝 Analyzing real-time context (Events: {})", events.len());
    let res = provider.summarize(&context).await.unwrap();
    println!("\n--- Summary ---\n{}\n", res.content);
}

async fn handle_plan(db_path: &Path) {
    let db = Database::open(db_path).expect("Failed to open database");
    let config = ConfigManager::load();
    let provider = get_provider(&config);
    
    let events = db.get_recent_events(30).unwrap_or_default();
    let context = format_events_to_context(&events);
    
    println!("📋 Generating plan based on context (Events: {})", events.len());
    let res = provider.generate_plan(&context).await.unwrap();
    println!("\n--- Plan ---\n{}\n", res.content);
}

fn format_events_to_context(events: &[Event]) -> String {
    let raw_context = events.iter()
        .map(|e| {
            let meta_suffix = if let Some(ref m) = e.metadata {
                format!(" (metadata: {})", m)
            } else {
                "".to_string()
            };
            format!("[{}]{} {}", e.event_type, meta_suffix, e.content)
        })
        .collect::<Vec<_>>()
        .join("\n");

    crate::intelligence::privacy::PrivacyFilter::mask_sensitive_data(&raw_context)
}

async fn handle_status(db_path: &Path) {
    let db = match Database::open(db_path) {
        Ok(d) => d,
        Err(e) => {
            println!("ContextFlow Status: Offline (Database error: {})", e);
            return;
        }
    };

    let active_session = db.get_active_session_id().unwrap_or(None);
    let event_count = db.get_event_count().unwrap_or(0);

    println!("\n--- ContextFlow Daemon Status ---");
    println!("Status: Active (listening on localhost)");
    if let Some(session) = active_session {
        println!("Active Session: {}", session);
    } else {
        println!("Active Session: None (idle)");
    }
    println!("Total Stored Events: {}", event_count);
    println!("Database Path: {}", db_path.to_string_lossy());
    println!("---------------------------------\n");
}

async fn handle_fix(db_path: &Path) {
    let db = Database::open(db_path).expect("Failed to open database");
    let config = ConfigManager::load();
    let provider = get_provider(&config);

    let events = db.get_recent_events(30).unwrap_or_default();
    let context = format_events_to_context(&events);

    // Try to find the most recent failed command to display
    let failed_cmd = events.iter()
        .find(|e| {
            e.event_type == "terminal_command" && 
            e.metadata.as_ref().map(|m| m.contains("\"exitCode\"") && !m.contains("\"exitCode\":0")).unwrap_or(false)
        });

    if let Some(cmd) = failed_cmd {
        println!("🛠️  Analyzing failure for command: '{}'", cmd.content);
    } else {
        println!("🛠️  No recent command failure detected. Analyzing general workspace state...");
    }

    let source_path = Path::new("src/main.rs");
    let source_context = if source_path.exists() {
        std::fs::read_to_string(source_path).unwrap_or_default()
    } else {
        "".to_string()
    };

    println!("🤖 Requesting agentic fix recommendation...");
    match provider.generate_fix(&context, &source_context).await {
        Ok(res) => {
            println!("\n--- Recommended Fix ---\n{}\n", res.content);
        }
        Err(e) => {
            println!("❌ Failed to generate fix: {}", e);
        }
    }
}

async fn handle_config(set: Option<String>, get: Option<String>) {
    let mut config = ConfigManager::load();
    if let Some(key_value) = set {
        let parts: Vec<&str> = key_value.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim().to_string();
            
            let value_str = value.clone();
            match key {
                "provider" => config.provider = Some(value),
                "model" => config.model = Some(value),
                "baseUrl" | "base_url" => config.base_url = Some(value),
                "api_key" | "apiKey" => config.api_key = Some(value),
                _ => {
                    println!("❌ Unknown configuration key: '{}'", key);
                    return;
                }
            }
            if let Err(e) = ConfigManager::save(&config) {
                println!("❌ Failed to save configuration: {}", e);
            } else {
                println!("✅ Configuration updated: {} = '{}'", key, value_str);
            }
        } else {
            println!("❌ Invalid format for --set. Use key=value format (e.g. --set provider=openai)");
        }
    } else if let Some(key) = get {
        let value = match key.as_str() {
            "provider" => config.provider.as_ref(),
            "model" => config.model.as_ref(),
            "baseUrl" | "base_url" => config.base_url.as_ref(),
            "api_key" | "apiKey" => config.api_key.as_ref(),
            _ => {
                println!("❌ Unknown configuration key: '{}'", key);
                return;
            }
        };
        if let Some(v) = value {
            println!("{}", v);
        } else {
            println!("(not set)");
        }
    } else {
        println!("\n--- ContextFlow Local Configuration ---");
        println!("Provider:  {}", config.provider.unwrap_or_else(|| "gemini (default)".to_string()));
        println!("Model:     {}", config.model.unwrap_or_else(|| "gemini-2.5-flash (default)".to_string()));
        println!("Base URL:  {}", config.base_url.unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string()));
        println!("API Key:   {}", config.api_key.map(|_| "********".to_string()).unwrap_or_else(|| "(not set)".to_string()));
        println!("---------------------------------------\n");
    }
}
