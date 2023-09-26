use crate::client::app::App;
use anyhow::Result;
use autometrics::autometrics;
use axum::{
  body::Body,
  http::{self, header, Request, Uri},
  response::IntoResponse,
  Extension,
};
use cowstr::{CowStr, ToCowStr};
use leptos::{get_configuration, leptos_config::ConfFile, view};
use moka::future::Cache;
use std::{
  fmt::{Debug, Display},
  path::{Path, PathBuf},
  sync::{Arc, OnceLock},
};
use tower::Service;
use tower_http::services::{ServeDir, ServeFile};

use crate::ssr::{config_this::ConfigThis, modules::fs::watch::watch_files_changed};

// first is uri path resolved, second is unmodified uri
static CACHES: OnceLock<(Cache<CowStr, CowStr>, Cache<CowStr, CowStr>)> = OnceLock::new();

pub static CACHE_FILE_STATE_HASHING: &str = "$___hashing___$";

#[derive(Clone, Debug)]
pub struct Fallback {
  pub path_public: PathBuf,
  pub service_fallback: ServeDir<ServeFile>,
  pub config_leptos: ConfFile,
}

impl Fallback {
  pub async fn new() -> Result<Self> {
    let config_leptos = get_configuration(None).await?;
    let path_public = Path::new(&config_leptos.leptos_options.site_root)
      .canonicalize()
      .expect("failed to get public directory.");
    let path_fallback_file = path_public.join("index.html");
    let service_base = ServeDir::new(&path_public)
      .precompressed_zstd()
      .precompressed_br();

    CACHES.get_or_init(|| (Cache::new(100_000), Cache::new(100_000)));

    Ok(Self {
      config_leptos,
      path_public: path_public.to_owned(),
      service_fallback: service_base
        .clone()
        .fallback(ServeFile::new(path_fallback_file)),
    })
  }

  pub async fn watch_file_changes(config: &ConfigThis) -> Result<()> {
    let rx_watcher = watch_files_changed(config.server.path_dir_public.clone(), None).await?;

    let dir_target = config.server.path_dir_public.to_string_lossy();

    tracing::info!("watching file changes: {}", dir_target);

    while let Ok(evt) = rx_watcher.recv().await {
      let (cache_file_hash_by_uri_path, cache_file_hash_by_uri) = CACHES.get().unwrap();

      cache_file_hash_by_uri.invalidate_all();

      let tasks = evt
        .paths
        .iter()
        .map(|x| async {
          cache_file_hash_by_uri_path
            .invalidate(&x.to_string_lossy().to_cowstr())
            .await
        })
        .collect::<Vec<_>>();

      for x in tasks {
        x.await;
      }
    }

    Ok(())
  }
}

impl Fallback {
  pub fn handle_file_cache_headers<TStrETAG>(headers: &mut http::HeaderMap, etag: TStrETAG)
  where
    TStrETAG: AsRef<str> + Display + Debug,
  {
    headers.insert(
      header::CACHE_CONTROL,
      "public, max-age=31536000 ".parse().unwrap(),
    );
    headers.insert(header::ETAG, etag.as_ref().parse().unwrap());
  }

  async fn handle_file(&self, uri: &Uri) -> Option<CowStr> {
    let uri_string_original = uri.to_cowstr();
    let (cache_file_hash_by_uri_path, cache_file_hash_by_uri) = CACHES.get().unwrap();

    if let Some(hash) = cache_file_hash_by_uri.get(&uri_string_original).await {
      return Some(hash);
    }

    // needed because of localhost, not using the domain
    let uri_string = if uri_string_original.starts_with("http") {
      uri_string_original.clone()
    } else {
      cowstr::format!("http://a.b{}", uri_string_original)
    };
    let url_parse = url::Url::parse(&uri_string);

    if url_parse.is_err() {
      tracing::error!("failed to parse URI: {}", uri);
      return None;
    }

    let url_parsed = url_parse.unwrap();
    let mut path_temp = url_parsed.path().to_cowstr();
    // remove first slash
    path_temp.remove(0);

    let path_file = self.path_public.join::<&str>(path_temp.as_ref());
    let path_file_str = path_file.to_string_lossy().to_cowstr();

    if let Some(state_file_cache_or_hash) = cache_file_hash_by_uri_path.get(&path_file_str).await {
      if state_file_cache_or_hash == CACHE_FILE_STATE_HASHING {
        return None;
      }

      // handle cache_file_hash_by_uri.invalidate_all();
      cache_file_hash_by_uri
        .insert(uri_string_original, state_file_cache_or_hash.clone())
        .await;

      // in this case the state is not hashing and is an actual hash
      return Some(state_file_cache_or_hash);
    }

    use tokio::fs::File;
    use tokio::io::{AsyncReadExt, BufReader};

    if let Ok(file) = File::open(&path_file).await {
      cache_file_hash_by_uri_path
        .insert(path_file_str.clone(), CACHE_FILE_STATE_HASHING.into())
        .await;
      let mut hasher = blake3::Hasher::new();
      let mut file_buf = BufReader::new(file);

      let mut buf = [0; 1024 * 64];

      while let Ok(bytes_read) = file_buf.read(&mut buf[..]).await {
        if bytes_read < 1 {
          break;
        }
        hasher.update(&buf);
      }

      let hash = hasher.finalize().to_cowstr();

      tokio::join!(
        cache_file_hash_by_uri.insert(uri_string_original, hash.clone()),
        cache_file_hash_by_uri_path.insert(path_file_str, hash.clone())
      );

      return Some(hash);
    }

    None
  }
}

#[autometrics]
pub async fn handler(
  uri: Uri,
  Extension(state): Extension<Arc<Fallback>>,
  request: Request<Body>,
) -> impl IntoResponse {
  let mut request_builder = Request::builder().uri(&uri);
  *request_builder.headers_mut().unwrap() = request.headers().to_owned();
  let mut service = state.service_fallback.clone();

  let handle_hash = tokio::spawn({
    let state = state.clone();
    async move { state.handle_file(&uri).await }
  });

  let (file_hash, result_file_serve) =
    tokio::join!(handle_hash, service.call(request_builder.body(()).unwrap()));

  match result_file_serve {
    Ok(mut response) => {
      if let Ok(Some(file_hash)) = file_hash {
        Fallback::handle_file_cache_headers(response.headers_mut(), file_hash);
      }

      return response.map(axum::body::boxed);
    }
    Err(_) => {
      let handler = leptos_axum::render_app_to_stream(
        state.config_leptos.leptos_options.to_owned(),
        move || view! { <App/> },
      );
      return handler(request).await.into_response();
    }
  }
}
