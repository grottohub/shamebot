pub mod middleware {
    use std::{convert::Infallible, sync::Arc};

    use database::prelude::Client;
    use warp::Filter;

    use crate::cronjob::Scheduler;

    pub fn with_db(db_client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
        warp::any().map(move || db_client.clone())
    }
    
    pub fn with_scheduler(scheduler: Arc<Scheduler>) -> impl Filter<Extract = (Arc<Scheduler>,), Error = Infallible> + Clone {
        warp::any().map(move ||  scheduler.clone())
    }
    
}

// TODO: implement rejection handling
pub mod handlers {
    use std::sync::Arc;

    use database::prelude::{Client, JobType};
    use log::info;
    use serde_derive::Serialize;
    use uuid::Uuid;
    use warp::{hyper::StatusCode, Reply, Rejection};

    use crate::cronjob::Scheduler;

    #[derive(Serialize)]
    struct JobsResponse {
        pester: Uuid,
        reminder: Uuid,
        overdue: Uuid,
    }

    pub async fn health_handler(db_client: Client, scheduler: Arc<Scheduler>) -> std::result::Result<impl Reply, Rejection> {
        let db_healthy = db_client.healthy().await;
        let sched_healthy = scheduler.healthy().await;
    
        if db_healthy && sched_healthy {
            Ok(StatusCode::OK)
        } else {
            Ok(StatusCode::SERVICE_UNAVAILABLE)
        }
    }

    pub async fn get_jobs_handler(task_id: Uuid, scheduler: Arc<Scheduler>) -> Result<impl Reply, Rejection> {
        let jobs = scheduler.get_jobs(task_id).await;
        
        if let Some(jobs) = jobs {
            let pester = jobs.get(&JobType::Pester)
                .unwrap()
                .unwrap_or_default();
            let reminder = jobs.get(&JobType::Reminder)
                .unwrap()
                .unwrap_or_default();
            let overdue = jobs.get(&JobType::Overdue)
                .unwrap()
                .unwrap_or_default();
            
            Ok(warp::reply::json(&JobsResponse {
                pester,
                reminder,
                overdue,
            }))
        } else {
            Err(warp::reject::reject())
        }
    }

    pub async fn register_handler(task_id: Uuid, scheduler: Arc<Scheduler>) -> Result<impl Reply, Rejection> {
        info!("received request to register jobs for task {:?}", task_id);

        let jobs = scheduler.register_all(task_id).await;

        if let Some(jobs) = jobs {
            let pester = jobs.get(&JobType::Pester)
                .unwrap()
                .unwrap_or_default();
            let reminder = jobs.get(&JobType::Reminder)
                .unwrap()
                .unwrap_or_default();
            let overdue = jobs.get(&JobType::Overdue)
                .unwrap()
                .unwrap_or_default();

            let resp = JobsResponse {
                pester,
                reminder,
                overdue,
            };

            Ok(warp::reply::json(&resp))
        } else {
            Err(warp::reject::reject())
        }
    }
}
