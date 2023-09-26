use anyhow::Context;
use notify::{event::AccessKind, Event, EventKind, RecommendedWatcher, Watcher};
use std::{
  fmt::Debug,
  path::{Path, PathBuf},
  time::Duration,
};
use tokio::runtime::Builder;

use crate::ssr::errors::app::ErrorsAll;

pub fn async_watcher() -> anyhow::Result<(RecommendedWatcher, kanal::AsyncReceiver<notify::Event>)>
{
  let (tx, rx) = kanal::bounded_async::<notify::Event>(9);
  let runtime_threads = Builder::new_current_thread().enable_all().build().unwrap();

  let watcher = RecommendedWatcher::new(
    move |res: Result<notify::Event, notify::Error>| {
      runtime_threads.block_on(async {
        tx.send(res.expect("get event")).await.unwrap();
      })
    },
    notify::Config::default()
      .with_compare_contents(false)
      .with_poll_interval(Duration::from_millis(250)),
  )?;

  Ok((watcher, rx))
}

pub async fn watch_files_changed<TPath: AsRef<Path> + Debug>(
  path: TPath,
  mode: Option<notify::RecursiveMode>,
) -> anyhow::Result<kanal::AsyncReceiver<notify::Event>> {
  let (tx, rx) = kanal::bounded_async::<notify::Event>(9);

  fn is_change(evt: &Event) -> bool {
    if evt.kind == EventKind::Access(AccessKind::Close(notify::event::AccessMode::Write)) {
      return true;
    }

    if evt.kind.is_remove() || evt.kind.is_modify() || evt.kind.is_create() {
      return true;
    }

    return false;
  }

  tokio::spawn({
    let path = PathBuf::from(path.as_ref());

    async move {
      let (mut watcher, rx_watcher) = async_watcher()?;

      watcher
        .watch(
          path.as_ref(),
          mode.unwrap_or_else(|| notify::RecursiveMode::Recursive),
        )
        .with_context(|| format!("watch {}", path.as_os_str().to_string_lossy()))?;

      while let Ok(evt) = rx_watcher.recv().await {
        if is_change(&evt) {
          tx.send(evt).await?;
        }
      }

      drop(tx);

      Ok::<(), ErrorsAll>(())
    }
  });

  Ok(rx)
}
