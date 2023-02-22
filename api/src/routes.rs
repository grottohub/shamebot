use std::fmt::Display;
use std::str::FromStr;

use database::prelude::{ApiKey, Client};
use log::error;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::{Deserialize, Serialize};
use rocket::Request;
use rocket::{http::Status, State};
use uuid::Uuid;

#[get("/health")]
pub async fn health(db_client: &State<Client>) -> Status {
    let healthy = db_client.healthy().await;

    if healthy {
        Status::Ok
    } else {
        Status::ServiceUnavailable
    }
}

#[catch(404)]
pub fn not_found() -> &'static str {
    "Not Found"
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GenericResponse<T> {
    status: u16,
    data: Vec<T>,
    error: Option<GenericError>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GenericError {
    message: String,
}

impl<T, E> From<Result<Option<T>, E>> for GenericResponse<T>
where
    E: Display,
{
    fn from(value: Result<Option<T>, E>) -> Self {
        match value {
            Ok(v) => {
                if let Some(v) = v {
                    GenericResponse {
                        status: 200,
                        data: vec![v],
                        error: None,
                    }
                } else {
                    GenericResponse {
                        status: 404,
                        data: vec![],
                        error: Some(GenericError {
                            message: String::from("Resource not found."),
                        }),
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
                GenericResponse {
                    status: 500,
                    data: vec![],
                    error: Some(GenericError {
                        message: format!("{}", e),
                    }),
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
    DbError,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ShamebotApiKey {
    api_key: ApiKey,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ShamebotApiKey {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db_client = req.guard::<&State<Client>>().await.succeeded().unwrap();

        match req.headers().get_one("x-api-key") {
            None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) => {
                let valid = ApiKey::is_valid(db_client, Uuid::from_str(key).unwrap()).await;

                match valid {
                    Ok(v) => {
                        if v {
                            let api_key = ApiKey::get(db_client, Uuid::from_str(key).unwrap())
                                .await
                                .ok()
                                .unwrap();
                            Outcome::Success(ShamebotApiKey {
                                api_key: api_key.unwrap(),
                            })
                        } else {
                            Outcome::Failure((Status::Forbidden, ApiKeyError::Invalid))
                        }
                    }
                    Err(_) => Outcome::Failure((Status::InternalServerError, ApiKeyError::DbError)),
                }
            }
        }
    }
}

pub mod guild {
    use database::prelude::{Client, Guild, User};
    use rocket::serde::json::Json;
    use rocket::{http::Status, State};

    use super::GenericResponse;

    #[post("/", format = "json", data = "<guild>")]
    pub async fn create_guild(
        db_client: &State<Client>,
        guild: Json<Guild>,
    ) -> (Status, Json<GenericResponse<Guild>>) {
        let new_guild = Guild::new(
            db_client,
            guild.id,
            guild.name.clone(),
            guild.icon.clone(),
            guild.send_to,
        )
        .await
        .map(Some);
        let resp = GenericResponse::from(new_guild);

        // if successful update status to 201 Created
        let status = if resp.status == 200 { 201 } else { resp.status };

        (Status::from_code(status).unwrap(), Json(resp))
    }

    #[get("/<id>")]
    pub async fn get_guild(
        db_client: &State<Client>,
        id: i64,
    ) -> (Status, Json<GenericResponse<Guild>>) {
        let guild = Guild::get(db_client, id).await;
        let resp = GenericResponse::from(guild);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[get("/<id>/users")]
    pub async fn get_guild_users(
        db_client: &State<Client>,
        id: i64,
    ) -> (Status, Json<GenericResponse<Vec<User>>>) {
        let users = Guild::get_users(db_client, id).await.map(Some);
        let resp = GenericResponse::from(users);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[put("/<id>", format = "json", data = "<guild>")]
    pub async fn update_guild(
        db_client: &State<Client>,
        id: i64,
        guild: Json<Guild>,
    ) -> (Status, Json<GenericResponse<()>>) {
        let updated = Guild::update_channel(db_client, id, guild.send_to.unwrap()).await;
        let resp = GenericResponse::from(updated);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[delete("/<id>")]
    pub async fn delete_guild(
        db_client: &State<Client>,
        id: i64,
    ) -> (Status, Json<GenericResponse<()>>) {
        let deleted = Guild::delete(db_client, id).await.map(Some);
        let resp = GenericResponse::from(deleted);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }
}

pub mod user {
    use database::prelude::{Client, User};
    use rocket::serde::json::Json;
    use rocket::{http::Status, State};

    use crate::routes::GenericResponse;

    #[post("/", format = "json", data = "<user>")]
    pub async fn create_user(
        db_client: &State<Client>,
        user: Json<User>,
    ) -> (Status, Json<GenericResponse<User>>) {
        let new_user = User::new(
            db_client,
            user.id,
            user.username.clone(),
            user.discriminator.clone(),
            user.avatar_hash.clone(),
        )
        .await
        .map(Some);
        let resp = GenericResponse::from(new_user);

        // if successful update status to 201 Created
        let status = if resp.status == 200 { 201 } else { resp.status };

        (Status::from_code(status).unwrap(), Json(resp))
    }

    #[get("/<id>")]
    pub async fn get_user(
        db_client: &State<Client>,
        id: i64,
    ) -> (Status, Json<GenericResponse<User>>) {
        let user = User::get(db_client, id).await;
        let resp = GenericResponse::from(user);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }
}

pub mod users {
    use database::prelude::{Client, User};
    use rocket::serde::json::Json;
    use rocket::{http::Status, State};

    use super::GenericResponse;

    #[post("/", format = "json", data = "<users>")]
    pub async fn create_users(
        db_client: &State<Client>,
        users: Json<Vec<User>>,
    ) -> (Status, Json<GenericResponse<Vec<User>>>) {
        let new_users = User::new_batch(db_client, users.to_vec()).await.map(Some);
        let resp = GenericResponse::from(new_users);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[post("/associate/<guild_id>", format = "json", data = "<users>")]
    pub async fn associate_users(
        db_client: &State<Client>,
        users: Json<Vec<i64>>,
        guild_id: i64,
    ) -> (Status, Json<GenericResponse<Vec<()>>>) {
        let associated = User::batch_associate(db_client, users.to_vec(), guild_id)
            .await
            .map(Some);
        let resp = GenericResponse::from(associated);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }
}

pub mod list {
    use database::prelude::{Client, List};
    use rocket::serde::json::Json;
    use rocket::{http::Status, State};
    use uuid::Uuid;

    use crate::routes::GenericResponse;

    #[post("/", format = "json", data = "<list>")]
    pub async fn create_list(
        db_client: &State<Client>,
        list: Json<List>,
    ) -> (Status, Json<GenericResponse<List>>) {
        let new_list = List::new(db_client, list.title.clone(), list.user_id)
            .await
            .map(Some);
        let resp = GenericResponse::from(new_list);

        // if successful update status to 201 Created
        let status = if resp.status == 200 { 201 } else { resp.status };

        (Status::from_code(status).unwrap(), Json(resp))
    }

    #[get("/<id>")]
    pub async fn get_list(
        db_client: &State<Client>,
        id: Uuid,
    ) -> (Status, Json<GenericResponse<List>>) {
        let list = List::get(db_client, id).await;
        let resp = GenericResponse::from(list);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[delete("/<list_id>")]
    pub async fn delete_list(
        db_client: &State<Client>,
        list_id: Uuid,
    ) -> (Status, Json<GenericResponse<()>>) {
        let deleted = List::delete(db_client, list_id).await.map(Some);
        let resp = GenericResponse::from(deleted);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    pub mod task {
        use database::prelude::{Client, List, Task, TaskJobs};
        use log::debug;
        use rocket::serde::json::Json;
        use rocket::serde::Deserialize;
        use rocket::{http::Status, State};
        use uuid::Uuid;

        use crate::routes::GenericResponse;

        #[derive(Debug, Deserialize)]
        #[serde(crate = "rocket::serde")]
        #[allow(dead_code)]
        pub struct JobError {
            message: String,
        }

        #[derive(Debug, Deserialize)]
        #[serde(crate = "rocket::serde")]
        #[allow(dead_code)]
        pub struct JobsResponse {
            status: u16,
            data: Vec<TaskJobs>,
            error: Option<JobError>,
        }

        async fn register_jobs(task_id: Uuid) -> Result<JobsResponse, reqwest::Error> {
            let client = reqwest::Client::new();
            let resp = client
                .post(format!("http://cron:8080/jobs/{:?}", task_id))
                .send()
                .await;

            match resp {
                Ok(r) => r.json::<JobsResponse>().await,
                Err(e) => Err(e),
            }
        }

        #[post("/<_id>/task", format = "json", data = "<task>")]
        pub async fn create_task(
            db_client: &State<Client>,
            _id: Uuid,
            task: Json<Task>,
        ) -> (Status, Json<GenericResponse<Task>>) {
            let task = Task::new(
                db_client,
                task.list_id,
                task.user_id,
                task.guild_id,
                task.title.clone(),
                task.content.clone(),
                task.pester,
                task.due_at,
            )
            .await
            .map(Some);
            let resp: GenericResponse<Task>;

            if let Some(task) = task.as_ref().ok().unwrap() {
                register_jobs(task.id)
                    .await
                    .map_err(|e| error!("{}", e))
                    .map(|j| debug!("{:?}", j))
                    .ok();

                let updated_task = Task::get(db_client, task.id).await;
                resp = GenericResponse::from(updated_task);
            } else {
                resp = GenericResponse::from(task);
            }

            (Status::from_code(resp.status).unwrap(), Json(resp))
        }

        #[put("/<_list_id>/task/<_task_id>", format = "json", data = "<task>")]
        pub async fn update_task(
            db_client: &State<Client>,
            _list_id: Uuid,
            _task_id: Uuid,
            task: Json<Task>,
        ) -> (Status, Json<GenericResponse<Task>>) {
            let updated = Task::update(db_client, task.into_inner()).await;
            let resp = GenericResponse::from(updated);

            (Status::from_code(resp.status).unwrap(), Json(resp))
        }

        #[get("/<_list_id>/task/<task_id>")]
        pub async fn get_task(
            db_client: &State<Client>,
            _list_id: Uuid,
            task_id: Uuid,
        ) -> (Status, Json<GenericResponse<Task>>) {
            let task = Task::get(db_client, task_id).await;
            let resp = GenericResponse::from(task);

            (Status::from_code(resp.status).unwrap(), Json(resp))
        }

        #[delete("/<_list_id>/task/<task_id>")]
        pub async fn delete_task(
            db_client: &State<Client>,
            _list_id: Uuid,
            task_id: Uuid,
        ) -> (Status, Json<GenericResponse<()>>) {
            let deleted = Task::delete(db_client, task_id).await.map(Some);
            let resp = GenericResponse::from(deleted);

            (Status::from_code(resp.status).unwrap(), Json(resp))
        }

        #[get("/<list_id>/tasks")]
        pub async fn get_tasks(
            db_client: &State<Client>,
            list_id: Uuid,
        ) -> (Status, Json<GenericResponse<Vec<Task>>>) {
            let tasks = List::get_tasks(db_client, list_id).await.map(Some);
            let resp = GenericResponse::from(tasks);

            (Status::from_code(resp.status).unwrap(), Json(resp))
        }
    }
}

pub mod proof {
    use database::prelude::{Client, Proof};
    use rocket::serde::json::Json;
    use rocket::{http::Status, State};
    use uuid::Uuid;

    use crate::routes::GenericResponse;

    #[post("/", format = "json", data = "<proof>")]
    pub async fn create_proof(
        db_client: &State<Client>,
        proof: Json<Proof>,
    ) -> (Status, Json<GenericResponse<Proof>>) {
        let new_proof = Proof::new(db_client, proof.content.clone(), proof.image.clone())
            .await
            .map(Some);
        let resp = GenericResponse::from(new_proof);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[get("/<id>")]
    pub async fn get_proof(
        db_client: &State<Client>,
        id: Uuid,
    ) -> (Status, Json<GenericResponse<Proof>>) {
        let proof = Proof::get(db_client, id).await;
        let resp = GenericResponse::from(proof);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[post("/<id>/approve")]
    pub async fn approve(
        db_client: &State<Client>,
        id: Uuid,
    ) -> (Status, Json<GenericResponse<()>>) {
        let approval = Proof::approve(db_client, id).await.map(Some);
        let resp = GenericResponse::from(approval);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[delete("/<id>")]
    pub async fn delete_proof(
        db_client: &State<Client>,
        id: Uuid,
    ) -> (Status, Json<GenericResponse<()>>) {
        let deleted = Proof::delete(db_client, id).await.map(Some);
        let resp = GenericResponse::from(deleted);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }
}

pub mod accountability {
    use database::prelude::{AccountabilityRequest, Client};
    use discord::bot::Bot;
    use rocket::serde::json::Json;
    use rocket::{http::Status, State};
    use uuid::Uuid;

    use crate::routes::GenericResponse;

    #[post("/", format = "json", data = "<request>")]
    pub async fn create_request(
        db_client: &State<Client>,
        discord_bot: &State<Bot>,
        request: Json<AccountabilityRequest>,
    ) -> (Status, Json<GenericResponse<AccountabilityRequest>>) {
        let new_request = AccountabilityRequest::new(
            db_client,
            request.requesting_user,
            request.requested_user,
            request.task_id,
        )
        .await
        .map(Some);

        if let Ok(r) = new_request.as_ref() {
            discord_bot.send_accountability_request(r).await;
        }

        let resp = GenericResponse::from(new_request);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[get("/<id>")]
    pub async fn get_request(
        db_client: &State<Client>,
        id: Uuid,
    ) -> (Status, Json<GenericResponse<AccountabilityRequest>>) {
        let request = AccountabilityRequest::get(db_client, id).await;
        let resp = GenericResponse::from(request);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[put("/<_id>", format = "json", data = "<request>")]
    pub async fn update_status(
        db_client: &State<Client>,
        _id: Uuid,
        request: Json<AccountabilityRequest>,
    ) -> (Status, Json<GenericResponse<()>>) {
        let approval =
            AccountabilityRequest::update_status(db_client, request.task_id, request.status)
                .await
                .map(Some);
        let resp = GenericResponse::from(approval);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[delete("/<id>")]
    pub async fn delete_request(
        db_client: &State<Client>,
        id: Uuid,
    ) -> (Status, Json<GenericResponse<()>>) {
        let deleted = AccountabilityRequest::delete(db_client, id).await.map(Some);
        let resp = GenericResponse::from(deleted);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }
}

pub mod discord {
    use chrono::Utc;
    use database::prelude::{ApiKey, Client, Token, User};
    use discord::bot::{Bot, GuildChannel, Member, User as DiscordUser};
    use log::{error, info};
    use rocket::serde::json::Json;
    use rocket::serde::{Deserialize, Serialize};
    use rocket::{http::Status, State};
    use uuid::Uuid;

    use crate::environment;
    use crate::routes::GenericResponse;

    use super::ShamebotApiKey;

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "rocket::serde")]
    struct TokenRequest {
        client_id: u64,
        client_secret: String,
        grant_type: String,
        code: String,
        redirect_uri: String,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "rocket::serde")]
    struct TokenResponse {
        access_token: String,
        token_type: String,
        expires_in: i64,
        refresh_token: String,
        scope: String,
    }

    impl Into<Token> for TokenResponse {
        fn into(self) -> Token {
            Token {
                id: Uuid::default(),
                access_token: self.access_token,
                token_type: self.token_type,
                expires_at: Utc::now().timestamp() + self.expires_in,
                refresh_token: self.refresh_token,
                scope: self.scope,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "rocket::serde")]
    struct RefreshRequest {
        client_id: u64,
        client_secret: String,
        grant_type: String,
        refresh_token: String,
    }

    #[get("/guild/<id>/members")]
    pub async fn get_guild_members(
        discord_bot: &State<Bot>,
        id: u64,
    ) -> (Status, Json<GenericResponse<Vec<Member>>>) {
        let members = discord_bot.get_guild_members(id).await;
        let resp = GenericResponse::from(members);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[get("/guild/<id>/channels")]
    pub async fn get_guild_channels(
        discord_bot: &State<Bot>,
        id: u64,
    ) -> (Status, Json<GenericResponse<Vec<GuildChannel>>>) {
        let text_channels = discord_bot.get_text_channels(id).await;
        let resp = GenericResponse::from(text_channels);

        (Status::from_code(resp.status).unwrap(), Json(resp))
    }

    #[get("/authorize?<code>")]
    pub async fn authorize(
        db_client: &State<Client>,
        env: &State<environment::Env>,
        code: String,
    ) -> Json<Option<User>> {
        let req = TokenRequest {
            client_id: env.client_id,
            client_secret: env.client_secret.clone(),
            code,
            grant_type: String::from("authorization_code"),
            redirect_uri: env.redirect_uri.clone(),
        };

        let client = reqwest::Client::new();
        let endpoint = "https://discord.com/api/oauth2/token";

        let resp = client
            .post(endpoint)
            .form(&req)
            .send()
            .await
            .map_err(|e| error!("{}", e));

        if let Ok(resp) = resp {
            let token = resp
                .json::<TokenResponse>()
                .await
                .map_err(|e| error!("{}", e))
                .unwrap();

            let persisted = Token::new(db_client, token.into())
                .await
                .map_err(|e| error!("{}", e))
                .unwrap();

            let user_resp = client
                .get("https://discord.com/api/users/@me")
                .bearer_auth(persisted.access_token.clone())
                .send()
                .await
                .map_err(|e| error!("{}", e))
                .unwrap();

            info!("{:?}", user_resp);

            let user = user_resp
                .json::<DiscordUser>()
                .await
                .map_err(|e| error!("{}", e))
                .unwrap();

            info!("{:?}", user);

            let new_user = User::new(
                db_client,
                *user.id.as_u64() as i64,
                user.name,
                user.discriminator.to_string(),
                user.avatar.unwrap_or_default(),
            )
            .await
            .map_err(|e| error!("{}", e))
            .unwrap();

            ApiKey::new(db_client, new_user.id, persisted.id)
                .await
                .map_err(|e| error!("{}", e))
                .unwrap();

            return Json(Some(new_user));
        }

        Json(None)
    }

    #[post("/refresh")]
    pub async fn refresh_token(
        db_client: &State<Client>,
        env: &State<environment::Env>,
        key: ShamebotApiKey,
    ) -> Status {
        let token = Token::get(db_client, key.api_key.discord_token)
            .await
            .map_err(|e| error!("{}", e));

        if let Some(token) = token.unwrap() {
            let refresh_req = RefreshRequest {
                client_id: env.client_id,
                client_secret: env.client_secret.clone(),
                grant_type: String::from("refresh_token"),
                refresh_token: token.refresh_token,
            };

            let client = reqwest::Client::new();
            let endpoint = "https://discord.com/api/oauth2/token";

            let resp = client
                .post(endpoint)
                .form(&refresh_req)
                .send()
                .await
                .map_err(|e| error!("{}", e));

            if let Ok(resp) = resp {
                let new_token = resp
                    .json::<TokenResponse>()
                    .await
                    .map_err(|e| error!("{}", e))
                    .unwrap();

                let updated_token = Token {
                    id: token.id,
                    access_token: new_token.access_token,
                    token_type: token.token_type,
                    expires_at: Utc::now().timestamp() + new_token.expires_in,
                    refresh_token: new_token.refresh_token,
                    scope: new_token.scope,
                };

                let refreshed = Token::refresh(db_client, updated_token)
                    .await
                    .map_err(|e| error!("{}", e));

                if let Ok(_) = refreshed {
                    return Status::Ok;
                }
            }
        }

        Status::InternalServerError
    }
}
