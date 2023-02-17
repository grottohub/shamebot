use database::prelude::Client;

#[macro_use]
extern crate rocket;

mod routes;

#[launch]
async fn rocket() -> _ {
    let db_client = Client::new().await;
    rocket::build()
        .manage(db_client)
        .mount("/", routes![routes::health])
        .mount(
            "/guild",
            routes![
                routes::guild::create_guild,
                routes::guild::get_guild,
                routes::guild::get_guild_users,
                routes::guild::delete_guild
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
        .register("/", rocket::catchers![routes::not_found])
}
