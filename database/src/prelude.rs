use std::collections::HashMap;

use mobc::{Connection, Pool};
use mobc_postgres::tokio_postgres::{NoTls, Row};
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use postgres_types::{FromSql, ToSql};
use thiserror::Error;
use tokio::task::JoinError;
use uuid::Uuid;

pub use crate::client::Client;

#[derive(Debug, Clone)]
pub struct Guild {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub send_to: Option<i64>,
}

impl Guild {
    pub async fn new(
        db_client: &Client,
        id: i64,
        name: String,
        icon: Option<String>,
        send_to: Option<i64>,
    ) -> Result<Self, DatabaseError> {
        let guild = Guild::insert(db_client, id, name, icon, send_to).await?;

        Ok(guild.into())
    }

    pub async fn get(db_client: &Client, id: i64) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM guilds WHERE id = $1";
        let guild = db_client.query_one(query, &[&id]).await?;

        Ok(guild.into())
    }

    pub async fn delete(db_client: &Client, id: i64) -> Result<(), DatabaseError> {
        let query = "DELETE FROM guilds WHERE id = $1";
        db_client.query_opt(query, &[&id]).await?;

        Ok(())
    }

    async fn insert(
        db_client: &Client,
        id: i64,
        name: String,
        icon: Option<String>,
        send_to: Option<i64>,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            guilds (id, name, icon, send_to)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE
            SET
                name = EXCLUDED.name,
                icon = EXCLUDED.icon,
                send_tp = EXCLUDED.send_to
            RETURNING *";
        db_client
            .query_one(query, &[&id, &name, &icon, &send_to])
            .await
    }
}

impl From<Row> for Guild {
    fn from(row: Row) -> Self {
        let id = row.get("id");
        let name = row.get("name");
        let icon = row.get("icon");
        let send_to = row.get("send_to");

        Guild {
            id,
            name,
            icon,
            send_to,
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub discriminator: String,
    pub avatar_hash: String,
}

impl User {
    pub async fn new(
        db_client: &Client,
        id: i64,
        username: String,
        discriminator: String,
        avatar_hash: String,
    ) -> Result<Self, DatabaseError> {
        let user = User::insert(db_client, id, username, discriminator, avatar_hash).await?;

        Ok(user.into())
    }

    pub async fn new_batch(
        db_client: &Client,
        ids: Vec<i64>,
        usernames: Vec<String>,
        discriminators: Vec<String>,
        avatar_hashes: Vec<String>,
    ) -> Result<Vec<User>, DatabaseError> {
        let zipped = ids
            .iter()
            .zip(usernames)
            .zip(discriminators)
            .zip(avatar_hashes);

        let mut user_instantiations = Vec::new();

        for user in zipped {
            let (((id, username), discriminator), avatar_hash) = user;
            user_instantiations.push(User::new(
                db_client,
                *id,
                username,
                discriminator,
                avatar_hash,
            ));
        }

        futures::future::try_join_all(user_instantiations).await
    }

    pub async fn batch_associate(
        db_client: &Client,
        users: Vec<User>,
        guild: Guild,
    ) -> Result<(), DatabaseError> {
        for user in users {
            user.associate(db_client, guild.clone()).await?;
        }
        Ok(())
    }

    pub async fn associate(&self, db_client: &Client, guild: Guild) -> Result<(), DatabaseError> {
        let query = "INSERT INTO user_guild (user_id, guild_id) VALUES ($1, $2)";
        db_client.query_opt(query, &[&self.id, &guild.id]).await?;

        Ok(())
    }

    pub async fn get(db_client: &Client, id: i64) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM users WHERE id = $1";
        let user = db_client.query_one(query, &[&id]).await?;

        Ok(user.into())
    }

    pub async fn delete(db_client: &Client, id: i64) -> Result<(), DatabaseError> {
        let query = "DELETE FROM users WHERE id = $1";
        db_client.query_opt(query, &[&id]).await?;

        Ok(())
    }

    async fn insert(
        db_client: &Client,
        id: i64,
        username: String,
        discriminator: String,
        avatar_hash: String,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            users (id, username, discriminator, avatar_hash)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE
            SET
                username = EXCLUDED.username,
                discriminator = EXCLUDED.discriminator,
                avatar_hash = EXCLUDED.avatar_hash
            RETURNING *";
        db_client
            .query_one(query, &[&id, &username, &discriminator, &avatar_hash])
            .await
    }
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        let id = row.get("id");
        let username = row.get("username");
        let discriminator = row.get("discriminator");
        let avatar_hash = row.get("avatar_hash");

        User {
            id,
            username,
            discriminator,
            avatar_hash,
        }
    }
}

#[derive(Debug, Clone)]
pub struct List {
    pub id: Uuid,
    pub title: String,
    pub user_id: i64,
}

impl List {
    pub async fn new(
        db_client: &Client,
        title: String,
        user_id: i64,
    ) -> Result<Self, DatabaseError> {
        let list = List::insert(db_client, title, user_id).await?;

        Ok(list.into())
    }

    pub async fn get(db_client: &Client, id: Uuid) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM lists WHERE id = $1";
        let list = db_client.query_one(query, &[&id]).await?;

        Ok(list.into())
    }

    pub async fn delete(db_client: &Client, id: Uuid) -> Result<(), DatabaseError> {
        let query = "DELETE FROM lists WHERE id = $1";
        db_client.query_opt(query, &[&id]).await?;

        Ok(())
    }

    pub async fn get_tasks(db_client: &Client, id: Uuid) -> Result<Vec<Task>, DatabaseError> {
        let query = "SELECT * FROM tasks WHERE list_id = $1";
        let mut tasks: Vec<Task> = Vec::new();
        let result = db_client.query(query, &[&id]).await?;

        for row in result {
            tasks.push(row.into())
        }

        Ok(tasks)
    }

    async fn insert(db_client: &Client, title: String, user_id: i64) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            lists (title, user_id)
            VALUES ($1, $2)
            RETURNING *";
        db_client.query_one(query, &[&title, &user_id]).await
    }
}

impl From<Row> for List {
    fn from(row: Row) -> Self {
        let id = row.get("id");
        let title = row.get("title");
        let user_id = row.get("user_id");

        List { id, title, user_id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobType {
    Pester,
    Overdue,
    Reminder,
    Unknown,
}

impl JobType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pester => "pester",
            Self::Overdue => "overdue",
            Self::Reminder => "reminder",
            Self::Unknown => "unknown",
        }
    }
}

impl From<&str> for JobType {
    fn from(value: &str) -> Self {
        match value {
            "pester" => Self::Pester,
            "overdue" => Self::Overdue,
            "reminder" => Self::Reminder,
            _ => Self::Unknown,
        }
    }
}

pub type TaskJobs = HashMap<JobType, Option<Uuid>>;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub list_id: Uuid,
    pub user_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub checked: bool,
    pub pester: Option<i16>,
    pub due_at: Option<i64>,
    pub proof_id: Option<Uuid>,
    pub pester_job: Option<Uuid>,
    pub overdue_job: Option<Uuid>,
    pub reminder_job: Option<Uuid>,
}

impl Task {
    pub async fn new(
        db_client: &Client,
        list_id: Uuid,
        user_id: i64,
        title: String,
        content: Option<String>,
        pester: Option<i16>,
        due_at: Option<i64>,
    ) -> Result<Self, DatabaseError> {
        let task =
            Task::insert(db_client, list_id, user_id, title, content, pester, due_at).await?;

        Ok(task.into())
    }

    pub async fn get(db_client: &Client, id: Uuid) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM tasks WHERE id = $1";
        let task = db_client.query_one(query, &[&id]).await?;

        Ok(task.into())
    }

    pub async fn delete(db_client: &Client, id: Uuid) -> Result<(), DatabaseError> {
        let query = "DELETE FROM tasks WHERE id = $1";
        db_client.query_opt(query, &[&id]).await?;

        Ok(())
    }

    pub async fn attach_job(
        db_client: &Client,
        task_id: Uuid,
        job_id: Uuid,
        job_type: JobType,
    ) -> Result<(), DatabaseError> {
        let query = format!(
            "UPDATE tasks SET {}_job = $1 WHERE id = $2",
            job_type.as_str()
        );
        db_client
            .query_opt(query.as_str(), &[&job_id, &task_id])
            .await?;

        Ok(())
    }

    pub async fn remove_job(
        db_client: &Client,
        task_id: Uuid,
        job_id: Uuid,
        job_type: JobType,
    ) -> Result<(), DatabaseError> {
        let query = format!(
            "UPDATE tasks SET {}_job = NULL WHERE id = $1",
            job_type.as_str()
        );
        db_client.query_opt(query.as_str(), &[&task_id]).await?;

        let remove_job_query = "DELETE FROM job WHERE id = $1";
        db_client.query_opt(remove_job_query, &[&job_id]).await?;

        Ok(())
    }

    pub async fn collect_jobs(
        db_client: &Client,
        task_id: Uuid,
    ) -> Result<TaskJobs, DatabaseError> {
        let mut result: HashMap<JobType, Option<Uuid>> = HashMap::new();
        let query = "SELECT pester_job, reminder_job, overdue_job FROM tasks WHERE id = $1";
        let row = db_client.query_one(query, &[&task_id]).await?;

        let pester_job: Option<Uuid> = row.get("pester_job");
        let reminder_job: Option<Uuid> = row.get("reminder_job");
        let overdue_job: Option<Uuid> = row.get("overdue_job");

        result.insert(JobType::Pester, pester_job);
        result.insert(JobType::Reminder, reminder_job);
        result.insert(JobType::Overdue, overdue_job);

        Ok(result)
    }

    pub async fn collect_all_jobs(
        db_client: &Client,
    ) -> Result<HashMap<Uuid, TaskJobs>, DatabaseError> {
        let mut result: HashMap<Uuid, TaskJobs> = HashMap::new();
        let query = "SELECT 
            id, pester_job, reminder_job, overdue_job 
            FROM tasks
            WHERE pester_job IS NOT NULL OR
                  reminder_job IS NOT NULL OR
                  overdue_job IS NOT NULL";
        let rows = db_client.query(query, &[]).await?;

        for row in rows {
            let pester_job: Option<Uuid> = row.get("pester_job");
            let reminder_job: Option<Uuid> = row.get("reminder_job");
            let overdue_job: Option<Uuid> = row.get("overdue_job");

            let jobs = HashMap::from([
                (JobType::Pester, pester_job),
                (JobType::Reminder, reminder_job),
                (JobType::Overdue, overdue_job),
            ]);

            result.insert(row.get("id"), jobs);
        }

        Ok(result)
    }

    async fn insert(
        db_client: &Client,
        list_id: Uuid,
        user_id: i64,
        title: String,
        content: Option<String>,
        pester: Option<i16>,
        due_at: Option<i64>,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO
            tasks (list_id, user_id, title, content, pester, due_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *";
        db_client
            .query_one(
                query,
                &[&list_id, &user_id, &title, &content, &pester, &due_at],
            )
            .await
    }
}

impl From<Row> for Task {
    fn from(row: Row) -> Self {
        let id = row.get("id");
        let list_id = row.get("list_id");
        let user_id = row.get("user_id");
        let title = row.get("title");
        let content = row.get("content");
        let checked = row.get("checked");
        let pester = row.get("pester");
        let due_at = row.get("due_at");
        let proof_id = row.get("proof_id");
        let pester_job = row.get("pester_job");
        let overdue_job = row.get("overdue_job");
        let reminder_job = row.get("reminder_job");

        Task {
            id,
            list_id,
            user_id,
            title,
            content,
            checked,
            pester,
            due_at,
            proof_id,
            pester_job,
            overdue_job,
            reminder_job,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Proof {
    pub id: Uuid,
    pub content: Option<String>,
    pub image: Option<String>,
    pub approved: bool,
}

impl Proof {
    pub async fn new(
        db_client: &Client,
        content: Option<String>,
        image: Option<String>,
    ) -> Result<Self, DatabaseError> {
        let proof = Proof::insert(db_client, content, image).await?;

        Ok(proof.into())
    }

    pub async fn get(db_client: &Client, id: Uuid) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM proof WHERE id = $1";
        let proof = db_client.query_one(query, &[&id]).await?;

        Ok(proof.into())
    }

    pub async fn approve(db_client: &Client, id: Uuid) -> Result<(), DatabaseError> {
        let query = "UPDATE proof SET approved = true WHERE id = $1";
        db_client.query_opt(query, &[&id]).await?;

        Ok(())
    }

    pub async fn delete(db_client: &Client, id: Uuid) -> Result<(), DatabaseError> {
        let query = "DELETE FROM proof WHERE id = $1";
        db_client.query_opt(query, &[&id]).await?;

        Ok(())
    }

    async fn insert(
        db_client: &Client,
        content: Option<String>,
        image: Option<String>,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            proof (content, image)
            VALUES ($1, $2)
            RETURNING *";
        db_client.query_one(query, &[&content, &image]).await
    }
}

impl From<Row> for Proof {
    fn from(row: Row) -> Self {
        let id = row.get("id");
        let content = row.get("content");
        let image = row.get("image");
        let approved = row.get("approved");

        Proof {
            id,
            content,
            image,
            approved,
        }
    }
}

#[derive(Debug, Clone, ToSql, FromSql, PartialEq)]
#[postgres(name = "accepted")]
pub enum RequestStatus {
    #[postgres(name = "accepted")]
    Accepted,
    #[postgres(name = "pending")]
    Pending,
    #[postgres(name = "rejected")]
    Rejected,
}

#[derive(Debug, Clone)]
pub struct AccountabilityRequest {
    pub requesting_user: i64,
    pub requested_user: i64,
    pub task_id: Uuid,
    pub status: RequestStatus,
}

impl AccountabilityRequest {
    pub async fn new(
        db_client: &Client,
        requesting_user: i64,
        requested_user: i64,
        task_id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let result =
            AccountabilityRequest::insert(db_client, requesting_user, requested_user, task_id)
                .await?;

        Ok(result.into())
    }

    pub async fn get(db_client: &Client, task_id: Uuid) -> Result<Option<Self>, DatabaseError> {
        let query = "SELECT * FROM accountability_requests WHERE task_id = $1";
        let result = db_client.query_opt(query, &[&task_id]).await?;

        if let Some(result) = result {
            Ok(Some(result.into()))
        } else {
            Ok(None)
        }
    }

    pub async fn update_status(
        db_client: &Client,
        task_id: Uuid,
        status: RequestStatus,
    ) -> Result<(), DatabaseError> {
        let query = "UPDATE accountability_requests SET status = $1 WHERE task_id = $2";
        db_client.query_opt(query, &[&status, &task_id]).await?;

        Ok(())
    }

    pub async fn delete(db_client: &Client, task_id: Uuid) -> Result<(), DatabaseError> {
        let query = "DELETE FROM accountability_requests WHERE task_id = $1";
        db_client.query_opt(query, &[&task_id]).await?;

        Ok(())
    }

    async fn insert(
        db_client: &Client,
        requesting_user: i64,
        requested_user: i64,
        task_id: Uuid,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            accountability_requests (requesting_user, requested_user, task_id)
            VALUES ($1, $2, $3)
            RETURNING *";
        db_client
            .query_one(query, &[&requesting_user, &requested_user, &task_id])
            .await
    }
}

impl From<Row> for AccountabilityRequest {
    fn from(row: Row) -> Self {
        let requesting_user = row.get("requesting_user");
        let requested_user = row.get("requested_user");
        let task_id = row.get("task_id");
        let status = row.get("status");

        AccountabilityRequest {
            requesting_user,
            requested_user,
            task_id,
            status,
        }
    }
}

pub type DatabaseConnection = Connection<PgConnectionManager<NoTls>>;
pub type DatabasePool = Pool<PgConnectionManager<NoTls>>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("error getting connection from DB pool: {0}")]
    DBPoolError(mobc::Error<tokio_postgres::Error>),
    #[error("error executing of preparing DB query: {0}")]
    DBQueryError(#[from] tokio_postgres::Error),
    #[error("error joining spawned tasks: {0}")]
    JoinTaskError(#[from] JoinError),
    #[error("unknown error occurred")]
    DBGenericError(),
}
