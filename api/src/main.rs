use database::prelude::Client;

#[macro_use] extern crate rocket;

mod routes;

#[launch]
async fn rocket() -> _ {
    let db_client = Client::new().await;
    rocket::build()
        .manage(db_client)
        .mount("/", routes![routes::health])
        .mount("/guild", routes![routes::guild::create_guild, routes::guild::get_guild])
        .mount("/user", routes![routes::user::create_user, routes::user::get_user])
        .mount("/users", routes![routes::users::create_users])
}
