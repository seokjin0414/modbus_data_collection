use crate::service::server::health_check::health_check;
use crate::service::{
    server::get_state::{ServerState, get_state},
    task::task_init::task_init,
};
use anyhow::{Result, anyhow};
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderName, header};
use axum::routing::get;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tracing::info;

#[inline]
pub async fn server_initializer() -> Result<String> {
    let start = Instant::now();

    let hosting_address = SocketAddr::from_str("[::]:30000")
        .map_err(|e| anyhow!("Could not parse socket address: {:?}", e))?;

    let state: Arc<ServerState> = match get_state().await {
        Ok(state) => Arc::new(state),
        Err(e) => return Err(anyhow!("Could not create ServerState: {:?}", e)),
    };

    let healthcheck_router: axum::Router = axum::Router::new()
        .route("/healthcheck", get(health_check))
        // .route("/healthcheck/healthcheck", get(healthcheck_handler)) // simple healthcheck
        .with_state(Arc::clone(&state)); // system diagnosis

    let cors_layer: CorsLayer = CorsLayer::very_permissive().expose_headers([
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        header::ACCESS_CONTROL_REQUEST_HEADERS,
        header::ACCESS_CONTROL_REQUEST_METHOD,
        header::CACHE_CONTROL,
        header::CONTENT_TYPE,
        header::CONTENT_LENGTH,
        header::HOST,
        header::USER_AGENT,
        header::ACCEPT,
        header::ACCEPT_ENCODING,
        header::CONNECTION,
        header::ORIGIN,
        header::ACCEPT_LANGUAGE,
        header::DNT,
        header::REFERER,
        header::USER_AGENT,
        header::TE,
        HeaderName::from_static("sec-fetch-dest"),
        HeaderName::from_static("sec-fetch-mode"),
        HeaderName::from_static("sec-fetch-site"),
        HeaderName::from_static("x-api-key"),
    ]);

    // The final router.
    let app: axum::Router = axum::Router::new()
        .merge(healthcheck_router)
        .layer(cors_layer)
        .layer(DefaultBodyLimit::disable()); // 64MB

    // Tokio TCP listener에 IP를 연결해주고 오류처리.
    // Bind IP address to the Tokio TCP listener here.
    let listener = tokio::net::TcpListener::bind(hosting_address)
        .await
        .map_err(|e| anyhow!("Could not initialize TcpListener: {:?}", e))?;

    task_init(Arc::clone(&state))
        .await
        .map_err(|e| anyhow!("Could not schedule tasks: {:?}", e))?;

    info!(
        "sever started successfully on {} in {:?}.",
        hosting_address,
        start.elapsed()
    );

    info!("###### server version test- 1.0.3");

    // 여기서 앱을 Axum으로 서빙.
    // Serve app with Axum here.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
        .await
        .map_err(|e| anyhow!("Axum could not serve app: {:?}", e))?;

    Ok(String::from("Server exiting."))
}
