use anyhow::{Context, Result};
use autometrics::prometheus_exporter;
use axum::{
  error_handling::HandleErrorLayer,
  extract::DefaultBodyLimit,
  routing::{self, post},
  Extension, Router,
};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::info;
use std::{io::BufReader, path::PathBuf, sync::Arc, time::Duration};
use tokio::signal;
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::{
  compression::CompressionLayer, decompression::DecompressionLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use super::config_this;
use crate::{
  client::app::App,
  ssr::{
    errors, features,
    handlers::fallback::{self, Fallback},
  },
};

async fn config_from_der(path_cert: &PathBuf, path_key: &PathBuf) -> Result<rustls::ServerConfig> {
  let cert = tokio::spawn({
    let path_target = path_cert.clone();

    async move {
      let mut reader = BufReader::new(std::fs::OpenOptions::new().read(true).open(path_target)?);
      let raw = rustls_pemfile::certs(&mut reader)?;
      Ok::<Vec<_>, anyhow::Error>(raw.into_iter().map(rustls::Certificate).collect())
    }
  });

  let key = tokio::spawn({
    let path_target = path_key.clone();

    async move {
      let mut reader = BufReader::new(std::fs::OpenOptions::new().read(true).open(path_target)?);
      let mut raw = rustls_pemfile::pkcs8_private_keys(&mut reader)?;
      let raw = raw.swap_remove(0);
      Ok::<rustls::PrivateKey, anyhow::Error>(rustls::PrivateKey(raw))
    }
  });

  let mut config = rustls::ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_single_cert(cert.await??, key.await??)
    .context("build rusttls")?;

  config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

  Ok(config)
}

pub async fn execute() -> Result<()> {
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "this_app=info,tower_http=debug".into())
        .add_directive("this_app=info".parse()?)
        .add_directive("tower_http=info".parse()?),
    )
    .with(tracing_subscriber::fmt::layer().pretty())
    .init();

  prometheus_exporter::init();

  let mut config = config_this::ConfigThis::new()?;

  let mut config_tls = if config.tls.is_some() {
    let tls = config.tls.take().unwrap();
    let config_rustls = config_from_der(&tls.cert, &tls.key).await?;
    Some(RustlsConfig::from_config(Arc::new(config_rustls)))
  } else {
    None
  };

  // Setting get_configuration(None) means we'll be using cargo-leptos's env values
  let conf = get_configuration(None).await?;
  let addr = conf.leptos_options.site_addr;
  let leptos_options = conf.leptos_options;
  let routes = generate_route_list(App);
  let ext_fallback = Fallback::new().await?;
  // let (_db_postgres, db_postgres_connection) =
  //   features::databases::postgres::get_connection_pool(&config).await?;

  let layer_cors = match config.server.cors {
    config_this::CorsSetting::Default => {
      tracing::info!("CORS set to default");
      tower_http::cors::CorsLayer::new()
    }
    config_this::CorsSetting::VeryPermissive => {
      tracing::info!("CORS set to very permissive");
      tower_http::cors::CorsLayer::very_permissive()
    }
  };

  let layers = ServiceBuilder::new()
    // .layer(Extension(db_postgres_connection))
    .layer(Extension(Arc::new(ext_fallback)))
    .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
    .layer(HandleErrorLayer::new(
      errors::handler::Handler::handler_error,
    ))
    .layer(ConcurrencyLimitLayer::new(1024))
    .load_shed()
    .timeout(Duration::from_secs(60))
    .layer(TraceLayer::new_for_http())
    .layer(
      CompressionLayer::new()
        .zstd(true)
        .br(true)
        .no_gzip()
        .no_deflate(),
    )
    .layer(DecompressionLayer::new())
    .layer(layer_cors);

  let app = Router::new()
    // order is important, the fallback needs to be first (everything is revere), then it finds the Extensions
    .fallback(fallback::handler)
    .route(
      "/metrics",
      routing::get(|| async { prometheus_exporter::encode_http_response() }),
    )
    .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
    .leptos_routes(&leptos_options, routes, || view! { <App/> })
    .layer(layers)
    .with_state(leptos_options);

  info!("listening on http://{}", &addr);

  let handle = Handle::new();

  tokio::spawn({
    let handle = handle.clone();
    async move { shutdown_signal(handle).await }
  });

  if config_tls.is_some() {
    info!("Spawning with certificates.");
    axum_server::bind_rustls(addr, config_tls.take().unwrap())
      .handle(handle)
      .serve(app.into_make_service())
      .await?;
  } else {
    info!("Spawning without certificates.");
    axum_server::bind(addr)
      .handle(handle)
      .serve(app.into_make_service())
      .await?;
  }

  info!("Clean exit.");

  Ok(())
}

async fn shutdown_signal(handle: Handle) {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
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

  handle.graceful_shutdown(Some(Duration::from_secs(30)));
}
