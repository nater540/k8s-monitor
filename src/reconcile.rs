#![allow(dead_code)]

use std::sync::Arc;
use chrono::prelude::*;
use serde::{Serialize};
use tokio::{sync::RwLock, time::Duration};
use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube_runtime::controller::{Context, Controller, ReconcilerAction};

use k8s_openapi::api::{
  // core::v1::Pod,
  batch::v1::Job
};
use kube::api::{Api, Meta, ListParams};

use crate::error::*;

#[derive(Debug, Clone, Serialize)]
struct ResourceState {
  #[serde(deserialize_with = "from_ts")]
  pub last_event: DateTime<Utc>
}

impl ResourceState {
  fn new() -> Arc<RwLock<Self>> {
    Arc::new(RwLock::new(ResourceState { last_event: Utc::now() }))
  }
}

#[derive(Clone)]
struct ResourceContext {
  client: kube::Client,
  state: Arc<RwLock<ResourceState>>
}

async fn reconcile(job: Job, ctx: Context<ResourceContext>) -> Result<ReconcilerAction, Error> {
  let _client = ctx.get_ref().client.clone();
  ctx.get_ref().state.write().await.last_event = Utc::now();

  let job_name  = Meta::name(&job);
  let namespace = Meta::namespace(&job).expect("job should be namespaced");
  info!("Reconcile Job {}/{}", namespace, job_name);

  let status = job.status.as_ref().unwrap();

  debug!("{:?} : {:?}", job, status);

  Ok(ReconcilerAction {
    requeue_after: Some(Duration::from_secs(60))
  })
}

fn error_policy(error: &Error, _ctx: Context<ResourceContext>) -> ReconcilerAction {
  warn!("reconcile failed: {}", error);
  ReconcilerAction {
    requeue_after: Some(Duration::from_secs(360))
  }
}

pub struct Manager {
  state: Arc<RwLock<ResourceState>>
}

impl Manager {
  pub async fn new(client: kube::Client, namespace: &str) -> (Self, BoxFuture<'static, ()>) {
    let state = ResourceState::new();

    // Create a context object that gets passed to the reconcile function
    let context = Context::new(ResourceContext {
      client: client.clone(),
      state:  state.clone()
    });

    let jobs = Api::<Job>::namespaced(client, namespace);

    let drainer = Controller::new(jobs, ListParams::default())
      .run(reconcile, error_policy, context)
      .filter_map(|x| async move { std::result::Result::ok(x) })
      .for_each(|_| { futures::future::ready(()) })
      .boxed();

    (Self { state }, drainer)
  }
}
