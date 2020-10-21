#[allow(dead_code)]
mod error;
mod reconcile;

#[macro_use]
extern crate tracing;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opts {
  /// The kubernetes namespace to monitor
  #[structopt(short, long)]
  namespace: String,

  /// Verbose mode (-v, -vv, -vvv, etc.)
  #[structopt(short, long, parse(from_occurrences))]
  verbose: u8
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Opts::from_args();
  init_tracing(args.verbose.into());

  let client = kube::Client::try_default().await.expect("uh-oh spaghetti-o's");

  let (_manager, drainer) = reconcile::Manager::new(client, &args.namespace).await;

  tokio::select! {
    _ = drainer => tracing::warn!("controller drained")
  }
  Ok(())
}

fn init_tracing(verbosity: u64) {
  tracing_subscriber::fmt()
    .with_max_level(match verbosity {
      0 => tracing::Level::INFO,
      1 => tracing::Level::DEBUG,
      _ => tracing::Level::TRACE
    })
    .init();
}
