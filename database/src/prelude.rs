use std::str::FromStr;

use mobc::{Connection, Pool};
use mobc_postgres::{PgConnectionManager, tokio_postgres};
use mobc_postgres::tokio_postgres::{Error as PgError, NoTls, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::client::Client;

#[derive(Debug)]
pub struct Guild {
    pub id:     i64,
    pub name:   String,
    pub icon:   Option<String>,
}

impl Guild {
    pub async fn new(
        db_client: Client,
        id: i64,
        name: String,
        icon: Option<String>,
    ) -> Result<Self, DatabaseError> {
        let guild = Guild::insert(db_client, id, name, icon)
            .await?;
        
        Ok(Guild::from_row(&guild))
    }

    pub async fn get(
        mut db_client: Client,
        id: i64,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM guilds WHERE id = $1";
        let guild = db_client
            .query_one(query, &[&id])
            .await?;
        
        Ok(Guild::from_row(&guild))
    }

    fn from_row(row: &Row) -> Self {
        let id = row.get("id");
        let name = row.get("name");
        let icon = row.get("icon");

        Guild {
            id,
            name,
            icon,
        }
    }

    async fn insert(
        mut db_client: Client,
        id: i64,
        name: String,
        icon: Option<String>,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            guilds (id, name, icon)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET
                name = EXCLUDED.name,
                icon = EXCLUDED.icon
            RETURNING *";
        db_client
            .query_one(query, &[&id, &name, &icon])
            .await
    }
}

#[derive(Debug)]
pub struct User {
    pub id:             i64,
    pub username:       String,
    pub discriminator:  String,
    pub avatar_hash:    String,
}

impl User {
    pub async fn new(
        db_client: Client,
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
        mut db_client: Client,
        guild: Guild
    ) -> bool {
        let query = "INSERT INTO user_guild (user_id, guild_id) VALUES ($1, $2)";
        db_client
            .query_one(query, &[&self.id, &guild.id])
            .await
            .map_err(|e| println!("{:?}", e))
            .is_ok()
    }

    pub async fn get(
        mut db_client: Client,
        id: i64,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM users WHERE id = $1";
        let user = db_client
            .query_one(query, &[&id])
            .await?;
        
        Ok(User::from_row(&user))
    }

    fn from_row(row: &Row) -> Self {
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
        mut db_client: Client,
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
                username = EXCLUDED.name,
                discriminator = EXCLUDED.discriminator,
                avatar_hash = EXCLUDED.avatar_hash
            RETURNING *";
        db_client
            .query_one(query, &[&id, &username, &discriminator, &avatar_hash])
            .await
    }
}

#[derive(Debug)]
pub struct List {
    pub id:         Uuid,
    pub title:      String,
    pub user_id:    i64,
}

impl List {
    pub async fn new(
        db_client: Client,
        title: String,
        user_id: i64,
    ) -> Result<Self, DatabaseError> {
        let list = List::insert(db_client, title, user_id)
            .await?;
        
        Ok(List::from_row(&list))
    }

    pub async fn get(
        mut db_client: Client,
        id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM lists WHERE id = $1";
        let list = db_client
            .query_one(query, &[&id.to_string()])
            .await?;
        
        Ok(List::from_row(&list))
    }

    fn from_row(row: &Row) -> Self {
        let id: &str = row.get("id");
        let title = row.get("title");
        let user_id = row.get("user_id");

        List {
            id: Uuid::from_str(id).unwrap(),
            title,
            user_id,
        }
    }

    async fn insert(
        mut db_client: Client,
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

#[derive(Debug)]
pub struct Task {
    pub id:         Uuid,
    pub list_id:    Uuid,
    pub user_id:    i64,
    pub title:      String,
    pub content:    Option<String>,
    pub checked:    bool,
    pub pester:     i16,
    pub due_at:     Option<i64>,
    pub proof_id:   Option<Uuid>,
}

impl Task {
    pub async fn new(
        db_client: Client,
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
        mut db_client: Client,
        id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM tasks WHERE id = $1";
        let task = db_client
            .query_one(query, &[&id.to_string()])
            .await?;
        
        Ok(Task::from_row(&task))
    }

    fn from_row(row: &Row) -> Self {
        let id: &str = row.get("id");
        let list_id: &str = row.get("list_id");
        let user_id = row.get("user_id");
        let title = row.get("title");
        let content = row.get("content");
        let checked = row.get("checked");
        let pester = row.get("pester");
        let due_at = row.get("due_at");
        let proof_id: &str = row.get("proof_id");

        Task {
            id: Uuid::from_str(id).unwrap(),
            list_id: Uuid::from_str(list_id).unwrap(),
            user_id,
            title,
            content,
            checked,
            pester,
            due_at,
            proof_id: Uuid::from_str(proof_id).ok(),
        }
    }

    async fn insert(
        mut db_client: Client,
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
            .query_one(query, &[&list_id.to_string(), &user_id, &title, &content, &pester, &due_at])
            .await
    }
}

#[derive(Debug)]
pub struct Proof {
    pub id:         Uuid,
    pub content:    Option<String>,
    pub image:      Option<String>,
    pub approved:   bool,
}

impl Proof {
    pub async fn new(
        db_client: Client,
        content: Option<String>,
        image: Option<String>,
    ) -> Result<Self, DatabaseError> {
        let proof = Proof::insert(db_client, content, image)
            .await?;
        
        Ok(Proof::from_row(&proof))
    }

    pub async fn get(
        mut db_client: Client,
        id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM proof WHERE id = $1";
        let proof = db_client
            .query_one(query, &[&id.to_string()])
            .await?;
        
        Ok(Proof::from_row(&proof))
    }

    pub async fn approve(
        mut db_client: Client,
        id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "UPDATE proof SET approved = true WHERE id = $1";
        let proof = db_client
            .query_one(query, &[&id.to_string()])
            .await?;
        
        Ok(Proof::from_row(&proof))
    }

    fn from_row(row: &Row) -> Self {
        let id: &str = row.get("id");
        let content = row.get("content");
        let image = row.get("image");
        let approved = row.get("approved");

        Proof {
            id: Uuid::from_str(id).unwrap(),
            content,
            image,
            approved,
        }
    }

    async fn insert(
        mut db_client: Client,
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

#[derive(Debug)]
pub enum RequestStatus {
    Accepted,
    Pending,
    Rejected,
    Unknown,
}

impl RequestStatus {
    pub fn from_str(status: &str) -> Self {
        match status {
            "accepted" => Self::Accepted,
            "pending" => Self::Pending,
            "rejected" => Self::Rejected,
            _ => Self::Unknown,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Accepted => "accepted",
            Self::Pending => "pending",
            Self::Rejected => "rejected",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
pub struct AccountabilityRequest {
    pub requesting_user:    i64,
    pub requested_user:     i64,
    pub task_id:            Uuid,
    pub status:             RequestStatus,
}

impl AccountabilityRequest {
    pub async fn new(
        db_client: Client,
        requesting_user: i64,
        requested_user: i64,
        task_id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let result = AccountabilityRequest::insert(db_client, requesting_user, requested_user, task_id)
            .await?;
        
        Ok(AccountabilityRequest::from_row(&result))
    }

    pub async fn get(
        mut db_client: Client,
        task_id: Uuid,
    ) -> Result<Self, DatabaseError> {
        let query = "SELECT * FROM accountability_requests WHERE task_id = $1";
        let result = db_client
            .query_one(query, &[&task_id.to_string()])
            .await?;
        
        Ok(AccountabilityRequest::from_row(&result))
    }

    pub async fn update_status(
        mut db_client: Client,
        task_id: Uuid,
        status: RequestStatus,
    ) -> Result<Self, DatabaseError> {
        let query = "UPDATE accountability_requests SET status = $1 WHERE task_id = $2";
        let result = db_client
            .query_one(query, &[&status.to_str(), &task_id.to_string()])
            .await?;
        
        Ok(AccountabilityRequest::from_row(&result))
    }

    fn from_row(row: &Row) -> Self {
        let requesting_user = row.get("requesting_user");
        let requested_user = row.get("requested_user");
        let task_id: &str = row.get("task_id");
        let status: &str = row.get("status");

        AccountabilityRequest {
            requesting_user,
            requested_user,
            task_id: Uuid::from_str(task_id).unwrap(),
            status: RequestStatus::from_str(status),
        }
    }

    async fn insert(
        mut db_client: Client,
        requesting_user: i64,
        requested_user: i64,
        task_id: Uuid,
    ) -> Result<Row, DatabaseError> {
        let query = "INSERT INTO 
            accountability_requests (requesting_user, requested_user, task_id)
            VALUES ($1, $2, $3)
            RETURNING *";
        db_client
            .query_one(query, &[&requesting_user, &requested_user, &task_id.to_string()])
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
