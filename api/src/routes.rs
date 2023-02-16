use database::prelude::Client;
use rocket::{http::Status, State};

#[get("/health")]
pub async fn health(db_client: &State<Client>) -> Status {
    let healthy = db_client.healthy().await;

    if healthy {
        Status::Ok
    } else {
        Status::ServiceUnavailable
    }
}

pub mod guild {
    use database::prelude::{Client, Guild};
    use log::error;
    use rocket::{http::Status, State};
    use rocket::serde::json::Json;

    #[post("/", format = "json", data = "<guild>")]
    pub async fn create_guild(db_client: &State<Client>, guild: Json<Guild>) -> Status {
        let new_guild = Guild::new(&db_client, guild.id, guild.name.clone(), guild.icon.clone(), guild.send_to)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let Some(_) = new_guild {
            Status::Created
        } else {
            Status::InternalServerError
        }
    }

    #[get("/<id>")]
    pub async fn get_guild(db_client: &State<Client>, id: i64) -> Option<Json<Guild>> {
        let guild = Guild::get(&db_client, id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();
        
        if let Some(guild) = guild {
            Some(Json(guild))
        } else {
            None
        }
    }
}

pub mod user {
    use database::prelude::{Client, User};
    use log::error;
    use rocket::{http::Status, State};
    use rocket::serde::json::Json;

    #[post("/", format = "json", data = "<user>")]
    pub async fn create_user(db_client: &State<Client>, user: Json<User>) -> Status {
        let new_user = User::new(&db_client, user.id, user.username.clone(), user.discriminator.clone(), user.avatar_hash.clone())
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();
        
        if let Some(_) = new_user {
            Status::Created
        } else {
            Status::InternalServerError
        }
    }

    #[get("/<id>")]
    pub async fn get_user(db_client: &State<Client>, id: i64) -> Option<Json<User>> {
        let user = User::get(&db_client, id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();
        
        if let Some(user) = user {
            Some(Json(user))
        } else {
            None
        }
    }
}

pub mod users {
    use database::prelude::{Client, User};
    use log::error;
    use rocket::{http::Status, State};
    use rocket::serde::json::Json;

    #[post("/", format = "json", data = "<users>")]
    pub async fn create_users(db_client: &State<Client>, users: Json<Vec<User>>) -> Status {
        let new_users = User::new_batch(&db_client, users.to_vec())
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();
        
        if let Some(_) = new_users {
            Status::Created
        } else {
            Status::InternalServerError
        }
    }
}
