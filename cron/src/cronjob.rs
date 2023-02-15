use std::sync::Arc;

use chrono::{Datelike, TimeZone, Timelike, Utc};
use database::prelude::{Client, JobType, Task, TaskJobs};
use discord::bot::Bot;
use log::{error, info};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{
    Job, JobScheduler, PostgresMetadataStore, PostgresNotificationStore, SimpleJobCode,
    SimpleNotificationCode,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Scheduler {
    scheduler: JobScheduler,
    db_client: Client,
}

impl Scheduler {
    pub async fn new() -> Self {
        let metadata_storage = Box::<PostgresMetadataStore>::default();
        let notification_storage = Box::<PostgresNotificationStore>::default();
        let simple_job_code = Box::<SimpleJobCode>::default();
        let simple_notification_code = Box::<SimpleNotificationCode>::default();
        let scheduler = JobScheduler::new_with_storage_and_code(
            metadata_storage,
            notification_storage,
            simple_job_code,
            simple_notification_code,
        )
        .await
        .map_err(|e| error!("{:?}", e))
        .unwrap();

        let db_client = Client::new().await;

        Scheduler {
            scheduler,
            db_client,
        }
    }

    pub async fn start(&self) {
        self.scheduler
            .start()
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();
    }

    pub async fn healthy(&self) -> bool {
        self.scheduler.inited().await
    }

    pub async fn resume_jobs(&self) {
        info!("attempting to resume existing jobs");

        let all_jobs = Task::collect_all_jobs(&self.db_client)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let Some(jobs) = all_jobs {
            if jobs.is_empty() {
                info!("no existing jobs found to resume");
                return;
            }

            for job in jobs {
                let task_id = job.0;
                let task_jobs = job.1;

                for task_job in task_jobs {
                    if let Some(job_id) = task_job.1 {
                        Task::remove_job(&self.db_client, task_id, job_id, task_job.0)
                            .await
                            .map_err(|e| error!("{:?}", e))
                            .ok();
                    }
                }

                self.register_all(task_id).await;
            }
        }
    }

    pub async fn register_all(&self, task_id: Uuid) -> Option<TaskJobs> {
        let task = Task::get(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let discord_mtx = Arc::new(Mutex::new(Bot::new().await));

        if let Some(task) = task {
            if let Some(pester_interval) = task.pester {
                // TODO: change this back to hours after testing
                let cron_schedule = format!("1/{:?} * * * * *", pester_interval);
                self.register_pester_job(Arc::clone(&discord_mtx), task_id, cron_schedule.as_str())
                    .await;
            }

            if let Some(due_at) = task.due_at {
                let five_min_after = Utc.timestamp_opt(due_at + 300, 0).unwrap();
                // cron format = seconds, minutes, hours, day of month, month, day of week
                // this sets the cron to execute once, five minutes after the due date
                let cron_schedule = format!(
                    "0 {} {} {} {} *",
                    five_min_after.minute(),
                    five_min_after.hour(),
                    five_min_after.day(),
                    five_min_after.month(),
                );
                self.register_overdue_job(
                    Arc::clone(&discord_mtx),
                    task_id,
                    cron_schedule.as_str(),
                )
                .await;

                let one_hour_before = Utc.timestamp_opt(due_at - 3600, 0).unwrap();
                let cron_schedule = format!(
                    "0 {} {} {} {} *",
                    one_hour_before.minute(),
                    one_hour_before.hour(),
                    one_hour_before.day(),
                    one_hour_before.month(),
                );
                self.register_reminder_job(
                    Arc::clone(&discord_mtx),
                    task_id,
                    cron_schedule.as_str(),
                )
                .await;
            }
        }

        Task::collect_jobs(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok()
    }

    pub async fn register_pester_job(
        &self,
        discord_mtx: Arc<Mutex<Bot>>,
        task_id: Uuid,
        cron_schedule: &str,
    ) {
        info!("registering pester cron for task {:?}", task_id);

        let job = Job::new_async(cron_schedule, move |uuid, _| {
            let discord_clone = Arc::clone(&discord_mtx);
            Box::pin(async move {
                let discord_lock = discord_clone.lock().await;

                discord_lock.send_pester_message(task_id).await;

                info!("triggered cron {:?}", uuid);
            })
        })
        .map_err(|e| error!("{:?}", e))
        .ok();

        if let Some(job) = job {
            let uuid = self
                .scheduler
                .add(job)
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();

            if let Some(uuid) = uuid {
                Task::attach_job(&self.db_client, task_id, uuid, JobType::Pester)
                    .await
                    .map_err(|e| error!("{:?}", e))
                    .ok();

                info!("registered cron {:?} for task {:?}", uuid, task_id);
            }
        }
    }

    pub async fn register_reminder_job(
        &self,
        discord_mtx: Arc<Mutex<Bot>>,
        task_id: Uuid,
        cron_schedule: &str,
    ) {
        info!("registering reminder cron for task {:?}", task_id);

        let job = Job::new_async(cron_schedule, move |uuid, _| {
            let discord_clone = Arc::clone(&discord_mtx);
            Box::pin(async move {
                let discord_lock = discord_clone.lock().await;

                discord_lock.send_reminder(task_id).await;

                discord_lock.send_task(task_id).await;

                info!("triggered cron {:?}", uuid);
            })
        })
        .map_err(|e| error!("{:?}", e))
        .ok();

        if let Some(job) = job {
            let uuid = self
                .scheduler
                .add(job)
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();

            if let Some(uuid) = uuid {
                Task::attach_job(&self.db_client, task_id, uuid, JobType::Reminder)
                    .await
                    .map_err(|e| error!("{:?}", e))
                    .ok();

                info!("registered cron {:?} for task {:?}", uuid, task_id);
            }
        }
    }

    pub async fn register_overdue_job(
        &self,
        discord_mtx: Arc<Mutex<Bot>>,
        task_id: Uuid,
        cron_schedule: &str,
    ) {
        info!("registering overdue cron for task {:?}", task_id);

        let job = Job::new_async(cron_schedule, move |uuid, _| {
            let discord_clone = Arc::clone(&discord_mtx);
            Box::pin(async move {
                let discord_lock = discord_clone.lock().await;

                discord_lock.send_overdue_notice(task_id).await;

                discord_lock.send_task(task_id).await;

                info!("triggered cron {:?}", uuid);
            })
        })
        .map_err(|e| error!("{:?}", e))
        .ok();

        if let Some(job) = job {
            let uuid = self
                .scheduler
                .add(job)
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();

            if let Some(uuid) = uuid {
                Task::attach_job(&self.db_client, task_id, uuid, JobType::Overdue)
                    .await
                    .map_err(|e| error!("{:?}", e))
                    .ok();

                info!("registered cron {:?} for task {:?}", uuid, task_id);
            }
        }
    }
}
