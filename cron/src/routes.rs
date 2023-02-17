use database::prelude::Client;
use rocket::{http::Status, State};

use crate::cronjob::Scheduler;

#[get("/health")]
pub async fn health(db_client: &State<Client>, scheduler: &State<Scheduler>) -> Status {
    let db_healthy = db_client.healthy().await;
    let sched_healthy = scheduler.healthy().await;

    if db_healthy && sched_healthy {
        Status::Ok
    } else {
        Status::ServiceUnavailable
    }
}

#[catch(404)]
pub fn not_found() -> &'static str {
    "Not Found"
}

pub mod jobs {
    use database::prelude::{DatabaseError, TaskJobs};
    use log::error;
    use rocket::serde::json::Json;
    use rocket::serde::Serialize;
    use rocket::{http::Status, State};
    use uuid::Uuid;

    use crate::cronjob::Scheduler;

    #[derive(Serialize)]
    #[serde(crate = "rocket::serde")]
    pub struct JobError {
        message: String,
    }

    #[derive(Serialize)]
    #[serde(crate = "rocket::serde")]
    pub struct JobsResponse {
        status: u16,
        data: Vec<TaskJobs>,
        error: Option<JobError>,
    }

    impl From<Result<TaskJobs, DatabaseError>> for JobsResponse {
        fn from(value: Result<TaskJobs, DatabaseError>) -> Self {
            match value {
                Ok(v) => JobsResponse {
                    status: 200,
                    data: vec![v],
                    error: None,
                },
                Err(e) => {
                    error!("{}", e);
                    JobsResponse {
                        status: 500,
                        data: vec![],
                        error: Some(JobError {
                            message: format!("{}", e),
                        }),
                    }
                }
            }
        }
    }

    #[get("/<task_id>")]
    pub async fn get_jobs(
        scheduler: &State<Scheduler>,
        task_id: Uuid,
    ) -> (Status, Json<JobsResponse>) {
        let jobs = scheduler.get_jobs(task_id).await;
        let resp = JobsResponse::from(jobs);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[post("/<task_id>")]
    pub async fn register_jobs(
        scheduler: &State<Scheduler>,
        task_id: Uuid,
    ) -> (Status, Json<JobsResponse>) {
        let jobs = scheduler.register_all(task_id).await;
        let resp = JobsResponse::from(jobs);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }
}
