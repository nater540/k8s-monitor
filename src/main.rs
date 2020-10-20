#[allow(dead_code)]
mod error;
mod resource;
use resource::ResourceManager;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  let client = kube::Client::try_default().await.expect("create client");

  let (_manager, drainer) = ResourceManager::new(client, "payments").await;

  tokio::select! {
    _ = drainer => tracing::warn!("controller drained")
  }
  Ok(())
}
