use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::routing::get;
use rmcp::transport::streamable_http_server::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::session::store::SessionStore;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::audit::TracingAuditSink;
use crate::auth::AuthContext;
use crate::auth::AuthMode;
use crate::catalog::source::load_catalog;
use crate::config::Config;
use crate::config::SessionStoreConfig;
use crate::mcp::SupernodeMcpServer;
use crate::policy::Policy;
use crate::session::SqliteSessionStore;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    let cancellation_token = CancellationToken::new();
    let app = router(config.clone(), cancellation_token.child_token()).await?;
    let listener = TcpListener::bind(config.bind_addr).await?;

    info!(
        bind_addr = %config.bind_addr,
        auth_mode = ?config.auth_mode,
        log_level = %config.log_level,
        "starting supernode MCP server"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(cancellation_token))
        .await?;

    Ok(())
}

async fn router(config: Config, cancellation_token: CancellationToken) -> anyhow::Result<Router> {
    let auth_mode = config.auth_mode;
    let auth_context = match auth_mode {
        AuthMode::Trusted => AuthContext::trusted(),
        AuthMode::OAuth => anyhow::bail!("MCP_AUTH_MODE=oauth is not implemented yet"),
    };
    let session_store = session_store(&config.session_store)?;
    let catalog = load_catalog(&config.extension_catalog).await?;

    let mcp_state = SupernodeMcpServer::new(
        auth_context.clone(),
        Policy,
        Arc::new(TracingAuditSink),
        Arc::new(catalog),
    );
    let mut mcp_config = StreamableHttpServerConfig::default()
        .with_cancellation_token(cancellation_token)
        .with_allowed_hosts([
            "localhost",
            "127.0.0.1",
            "::1",
            "supernode-mcp",
            "supernode-mcp.control-plane",
            "supernode-mcp.control-plane.svc",
            "supernode-mcp.control-plane.svc.cluster.local",
        ]);
    mcp_config.session_store = session_store;

    let mcp_service = StreamableHttpService::new(
        move || Ok(mcp_state.clone()),
        Arc::new(LocalSessionManager::default()),
        mcp_config,
    );

    Ok(Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .nest_service("/mcp", mcp_service)
        .layer(Extension(auth_context)))
}

fn session_store(config: &SessionStoreConfig) -> anyhow::Result<Option<Arc<dyn SessionStore>>> {
    match config {
        SessionStoreConfig::Memory => Ok(None),
        SessionStoreConfig::Sqlite { path, ttl_seconds } => {
            info!(path = %path.display(), ?ttl_seconds, "using SQLite MCP session store");
            let store = SqliteSessionStore::new(path, *ttl_seconds)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            Ok(Some(Arc::new(store)))
        }
    }
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn readyz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn shutdown_signal(cancellation_token: CancellationToken) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    cancellation_token.cancel();
}
