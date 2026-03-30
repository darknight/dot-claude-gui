use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use clap::Parser;
use rand::RngCore;
use tracing::info;

use claude_daemon::{server::build_router, state::AppState};

#[derive(Parser, Debug)]
#[command(name = "claude-daemon", about = "Claude Code GUI daemon")]
struct Args {
    /// Port to listen on.
    #[arg(long, default_value_t = 7890)]
    port: u16,

    /// Path to the Claude home directory.
    #[arg(long)]
    claude_home: Option<PathBuf>,

    /// Bearer token for API authentication (auto-generated if not provided).
    #[arg(long)]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "claude_daemon=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();

    // Resolve claude_home: CLI flag → $HOME/.claude/
    let claude_home = args.claude_home.unwrap_or_else(|| {
        dirs_next::home_dir()
            .expect("cannot determine home directory")
            .join(".claude")
    });

    // Ensure the directory exists.
    std::fs::create_dir_all(&claude_home)?;

    // Resolve or generate the auth token.
    let auth_token = match args.token {
        Some(t) => t,
        None => {
            let mut bytes = [0u8; 32];
            rand::rng().fill_bytes(&mut bytes);
            BASE64.encode(bytes)
        }
    };

    // Persist the token so clients can read it.
    let token_path = claude_home.join("daemon-token");
    std::fs::write(&token_path, &auth_token)?;
    info!("auth token written to {}", token_path.display());

    // Build state and load settings from disk.
    let state = AppState::new(claude_home.clone(), auth_token.clone());
    if let Err(e) = state.load_user_settings().await {
        tracing::warn!("could not load user settings: {e}");
    }

    // Build the router.
    let router = build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    info!("claude-daemon listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
