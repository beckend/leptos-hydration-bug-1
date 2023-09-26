use anyhow::Result;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<()> {
  use this_app::ssr::app::execute;
  execute().await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
  // no client-side main function
  // unless we want this to work with e.g., Trunk for a purely client-side app
  // see lib.rs for hydration function instead
}
