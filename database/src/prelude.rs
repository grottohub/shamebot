use mobc::{Connection, Pool};
use mobc_postgres::{PgConnectionManager, tokio_postgres};
use mobc_postgres::tokio_postgres::{Error as PgError, NoTls, Row};
use postgres_types::{ToSql, FromSql};
use thiserror::Error;
use uuid::Uuid;

pub use crate::client::Client;

#[derive(Debug, Clone)]
pub struct Guild {
    pub id:     i64,
    pub name:   String,
    pub icon:   Option<String>,
    pub send_to: Option<i64>,
}

impl Guild {
    pub async fn new(
        db_client: &mut Client,
        id: i64,
        name: String,
        icon: Option<String>,
        send_to: Option<i64>,
    ) -> Result<Self, DatabaseError> {
        let guild = Guild::insert(db_client, id, name, icon, send_to)
            .await?;
        
        Ok(Guild::from_row(&guild))
    }

    pub async fn get(
        db_client: &mut Client,
        id: i64,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM guilds WHERE id = $1";
        let guild = db_client
            .query_one(query, &[&id])
            .await?;
        
        Ok(Guild::from_row(&guild))
    }

    pub async fn delete(
        db_client: &mut Client,
        id: i64,
    ) -> Result<(), DatabaseError> {
        let query = "DELETE FROM guilds WHERE id = $1";
        db_client
            .query_opt(query, &[&id])
            .await?;
        
        Ok(())
    }

    pub fn from_row(row: &Row) -> Self {
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

    async fn insert(
        db_client: &mut Client,
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

#[derive(Debug, Clone)]
pub struct User {
    pub id:             i64,
    pub username:       String,
    pub discriminator:  String,
    pub avatar_hash:    String,
}

impl User {
    pub async fn new(
        db_client: &mut Client,
        id: i64,
        username: String,
        discriminator: String,
        avatar_hash: String,
    ) -> Result<Self, DatabaseError> {
        let user = User::insert(db_client, id, username, discriminator, avatar_hash)
            .await?;
        
        Ok(User::from_row(&user))
    }

    pub async fn associate(
        &self,
        db_client: &mut Client,
        guild: Guild
    ) -> Result<(), DatabaseError> {
        let query = "INSERT INTO user_guild (user_id, guild_id) VALUES ($1, $2)";
        db_client
            .query_opt(query, &[&self.id, &guild.id])
            .await?;
        
        Ok(())
    }

    pub async fn get(
        db_client: &mut Client,
        id: i64,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM users WHERE id = $1";
        let user = db_client
            .query_one(query, &[&id])
            .await?;
        
        Ok(User::from_row(&user))
    }

    pub async fn delete(
        db_client: &mut Client,
        id: i64,
    ) -> Result<(), DatabaseError> {
        let query = "DELETE FROM users WHERE id = $1";
        db_client
            .query_opt(query, &[&id])
            .await?;
        
        Ok(())
    }

    pub fn from_row(row: &Row) -> Self {
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

    async fn insert(
        db_client: &mut Client,
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

#[derive(Debug, Clone)]
pub struct List {
    pub id:         Uuid,
    pub title:      String,
    pub user_id:    i64,
}

impl List {
    pub async fn new(
        db_client: &mut Client,
        title: String,
        user_id: i64,
    ) -> Result<Self, DatabaseError> {
        let list = List::insert(db_client, title, user_id)
            .await?;
        
        Ok(List::from_row(&list))
    }

    pub async fn get(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM lists WHERE id = $1";
        let list = db_client
            .query_one(query, &[&id])
            .await?;
        
        Ok(List::from_row(&list))
    }

    pub async fn delete(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<(), DatabaseError> {
        let query = "DELETE FROM lists WHERE id = $1";
        db_client
            .query_opt(query, &[&id])
            .await?;
        
        Ok(())
    }

    pub async fn get_tasks(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<Vec<Task>, DatabaseError> {
        let query = "SELECT * FROM tasks WHERE list_id = $1";
        let mut tasks: Vec<Task> = Vec::new();
        let result = db_client
            .query(query, &[&id])
            .await?;
        
        for row in result {
            tasks.push(Task::from_row(&row))
        }

        Ok(tasks)
    }

    pub fn from_row(row: &Row) -> Self {
        let id = row.get("id");
        let title = row.get("title");
        let user_id = row.get("user_id");

        List {
            id,
            title,
            user_id,
        }
    }

    async fn insert(
        db_client: &mut Client,
        title: String,
        user_id: i64,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            lists (title, user_id)
            VALUES ($1, $2)
            RETURNING *";
        db_client
            .query_one(query, &[&title, &user_id])
            .await
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id:         Uuid,
    pub list_id:    Uuid,
    pub user_id:    i64,
    pub title:      String,
    pub content:    Option<String>,
    pub checked:    bool,
    pub pester:     Option<i16>,
    pub due_at:     Option<i64>,
    pub proof_id:   Option<Uuid>,
}

impl Task {
    pub async fn new(
        db_client: &mut Client,
        list_id: Uuid,
        user_id: i64,
        title: String,
        content: Option<String>,
        pester: Option<i16>,
        due_at: Option<i64>,
    ) -> Result<Self, DatabaseError> {
        let task = Task::insert(db_client, list_id, user_id, title, content, pester, due_at)
            .await?;

        Ok(Task::from_row(&task))
    }

    pub async fn get(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM tasks WHERE id = $1";
        let task = db_client
            .query_one(query, &[&id])
            .await?;
        
        Ok(Task::from_row(&task))
    }

    pub async fn delete(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<(), DatabaseError> {
        let query = "DELETE FROM tasks WHERE id = $1";
        db_client
            .query_opt(query, &[&id])
            .await?;
        
        Ok(())
    }

    pub fn from_row(row: &Row) -> Self {
        let id = row.get("id");
        let list_id = row.get("list_id");
        let user_id = row.get("user_id");
        let title = row.get("title");
        let content = row.get("content");
        let checked = row.get("checked");
        let pester = row.get("pester");
        let due_at = row.get("due_at");
        let proof_id = row.get("proof_id");

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
        }
    }

    async fn insert(
        db_client: &mut Client,
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
            .query_one(query, &[&list_id, &user_id, &title, &content, &pester, &due_at])
            .await
    }
}

#[derive(Debug, Clone)]
pub struct Proof {
    pub id:         Uuid,
    pub content:    Option<String>,
    pub image:      Option<String>,
    pub approved:   bool,
}

impl Proof {
    pub async fn new(
        db_client: &mut Client,
        content: Option<String>,
        image: Option<String>,
    ) -> Result<Self, DatabaseError> {
        let proof = Proof::insert(db_client, content, image)
            .await?;
        
        Ok(Proof::from_row(&proof))
    }

    pub async fn get(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM proof WHERE id = $1";
        let proof = db_client
            .query_one(query, &[&id])
            .await?;
        
        Ok(Proof::from_row(&proof))
    }

    pub async fn approve(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<(), DatabaseError> {
        let query = "UPDATE proof SET approved = true WHERE id = $1";
        db_client
            .query_opt(query, &[&id])
            .await?;
        
        Ok(())
    }

    pub async fn delete(
        db_client: &mut Client,
        id: Uuid,
    ) -> Result<(), DatabaseError> {
        let query = "DELETE FROM proof WHERE id = $1";
        db_client
            .query_opt(query, &[&id])
            .await?;
        
        Ok(())
    }

    pub fn from_row(row: &Row) -> Self {
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

    async fn insert(
        db_client: &mut Client,
        content: Option<String>,
        image: Option<String>,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            proof (content, image)
            VALUES ($1, $2)
            RETURNING *";
        db_client
            .query_one(query, &[&content, &image])
            .await
    }
}

#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name="accepted")]
pub enum RequestStatus {
    #[postgres(name="accepted")]
    Accepted,
    #[postgres(name="pending")]
    Pending,
    #[postgres(name="rejected")]
    Rejected,
}

// impl RequestStatus {
//     pub fn from_str(status: &str) -> Self {
//         match status {
//             "accepted" => Self::Accepted,
//             "pending" => Self::Pending,
//             "rejected" => Self::Rejected,
//             _ => Self::Unknown,
//         }
//     }

//     pub fn to_str(&self) -> &str {
//         match self {
//             Self::Accepted => "accepted",
//             Self::Pending => "pending",
//             Self::Rejected => "rejected",
//             Self::Unknown => "unknown",
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct AccountabilityRequest {
    pub requesting_user:    i64,
    pub requested_user:     i64,
    pub task_id:            Uuid,
    pub status:             RequestStatus,
}

impl AccountabilityRequest {
    pub async fn new(
        db_client: &mut Client,
        requesting_user: i64,
        requested_user: i64,
        task_id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let result = AccountabilityRequest::insert(db_client, requesting_user, requested_user, task_id)
            .await?;
        
        Ok(AccountabilityRequest::from_row(&result))
    }

    pub async fn get(
        db_client: &mut Client,
        task_id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM accountability_requests WHERE task_id = $1";
        let result = db_client
            .query_one(query, &[&task_id])
            .await?;
        
        Ok(AccountabilityRequest::from_row(&result))
    }

    pub async fn update_status(
        db_client: &mut Client,
        task_id: Uuid,
        status: RequestStatus,
    ) -> Result<(), DatabaseError> {
        let query = "UPDATE accountability_requests SET status = $1 WHERE task_id = $2";
        db_client
            .query_opt(query, &[&status, &task_id])
            .await?;
        
        Ok(())
    }

    pub async fn delete(
        db_client: &mut Client,
        task_id: Uuid,
    ) -> Result<(), DatabaseError> {
        let query = "DELETE FROM accountability_requests WHERE task_id = $1";
        db_client
            .query_opt(query, &[&task_id])
            .await?;
        
        Ok(())
    }

    pub fn from_row(row: &Row) -> Self {
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

    async fn insert(
        db_client: &mut Client,
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

pub type DatabaseConnection = Connection<PgConnectionManager<NoTls>>;
pub type DatabasePool = Pool<PgConnectionManager<NoTls>>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("error getting connection from DB pool: {0}")]
    DBPoolError(mobc::Error<tokio_postgres::Error>),
    #[error("error executing DB query: {0}")]
    DBQueryError(#[from] PgError),
    #[error("unknown error occurred")]
    DBGenericError(),
}
