use database::prelude::Client;
use discord::bot::Bot;
use rocket::{fairing::{Fairing, Info, Kind}, Request, Response, http::Header};
use utils::logging;

#[macro_use]
extern crate rocket;

mod environment;
mod routes;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, PUT, DELETE, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
async fn rocket() -> _ {
    logging::configure(vec![
        String::from("api"),
        String::from("database"),
        String::from("discord"),
    ]);
    let db_client = Client::new().await;
    let discord_bot = Bot::new().await;
    let env = environment::Env::new();
    rocket::build()
        .manage(db_client)
        .manage(discord_bot)
        .manage(env)
        .attach(CORS)
        .mount("/", routes![routes::health])
        .mount(
            "/guild",
            routes![
                routes::guild::create_guild,
                routes::guild::get_guild,
                routes::guild::get_guild_users,
                routes::guild::update_guild,
                routes::guild::delete_guild,
            ],
        )
        .mount(
            "/user",
            routes![routes::user::create_user, routes::user::get_user],
        )
        .mount(
            "/users",
            routes![routes::users::create_users, routes::users::associate_users],
        )
        .mount(
            "/list",
            routes![
                routes::list::create_list,
                routes::list::get_list,
                routes::list::delete_list,
                routes::list::task::create_task,
                routes::list::task::get_task,
                routes::list::task::get_tasks,
                routes::list::task::delete_task,
                routes::list::task::update_task,
            ],
        )
        .mount(
            "/proof",
            routes![
                routes::proof::create_proof,
                routes::proof::get_proof,
                routes::proof::approve,
                routes::proof::delete_proof,
            ],
        )
        .mount(
            "/accountability",
            routes![
                routes::accountability::create_request,
                routes::accountability::get_request,
                routes::accountability::update_status,
                routes::accountability::delete_request,
            ],
        )
        .mount(
            "/discord",
            routes![
                routes::discord::get_guild_members,
                routes::discord::get_guild_channels,
                routes::discord::authorize,
                routes::discord::refresh_token,
            ],
        )
        .register("/", rocket::catchers![routes::not_found])
}
