use std::time::Duration;

#[macro_use]
extern crate rocket;

use cronjob::Scheduler;
use database::prelude::Client;
use log::warn;
use utils::logging;

mod cronjob;
mod routes;

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    logging::configure(String::from("cron"));

    let db_client = Client::new().await;

    while !db_client.healthy().await {
        warn!("db not healthy, retrying in 5s...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    let scheduler = Scheduler::new().await;

    scheduler.start().await;

    scheduler.resume_jobs().await;

    rocket::build()
        .manage(db_client)
        .manage(scheduler)
        .mount("/", routes![routes::health])
        .mount(
            "/jobs",
            routes![routes::jobs::get_jobs, routes::jobs::register_jobs],
        )
        .register("/", rocket::catchers![routes::not_found])
}
