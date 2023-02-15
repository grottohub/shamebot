use std::{sync::Arc, time::Duration};

use cronjob::Scheduler;
use database::prelude::Client;
use log::{info, warn};
use utils::logging;
use uuid::Uuid;
use warp::Filter;

mod cronjob;
mod server;

#[tokio::main]
async fn main() {
    logging::configure(String::from("cron"));

    let db_client = Client::new().await;

    while !db_client.healthy().await {
        warn!("db not healthy, retrying in 5s...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    let scheduler = Arc::new(Scheduler::new().await);

    scheduler
        .start()
        .await;
    
    scheduler
        .resume_jobs()
        .await;
    
    let health_route = warp::get()
        .and(warp::path("health"))
        .and(server::middleware::with_db(db_client))
        .and(server::middleware::with_scheduler(Arc::clone(&scheduler)))
        .and_then(server::handlers::health_handler);

    let register_route = warp::post()
        .and(warp::path("task"))
        .and(warp::path::param::<Uuid>())
        .and(server::middleware::with_scheduler(Arc::clone(&scheduler)))
        .and_then(server::handlers::register_handler);
    
    let get_jobs_route = warp::get()
        .and(warp::path("task"))
        .and(warp::path::param::<Uuid>())
        .and(server::middleware::with_scheduler(Arc::clone(&scheduler)))
        .and_then(server::handlers::get_jobs_handler);
    
    let routes = health_route
        .or(register_route)
        .or(get_jobs_route);

    info!("listening on port 3030");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
