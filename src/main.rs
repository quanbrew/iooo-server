#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{delete, get, post, routes};
use rocket_contrib::database;
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};

use self::models::{DataError, DeleteItem, Item, UpdateItem};

mod models;
mod base62;


#[database("iooo")]
struct Database(postgres::Connection);


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/item")]
fn items(connection: Database) -> Json<Vec<Item>> {
    let Database(ref connection) = connection;
    Json(models::get_item_list(connection))
}


#[post("/item", format = "application/json", data = "<items>")]
fn update_item(connection: Database, items: Json<Vec<UpdateItem>>) -> Result<(), DataError> {
    let Database(ref connection) = connection;
    let Json(items) = items;
    let transaction = connection.transaction().map_err(DataError::Database)?;
    for item in items {
        item.insert(&transaction)?;
    }
    transaction.commit().map_err(DataError::Database)?;
    Ok(())
}


#[delete("/item", format = "application/json", data = "<data>")]
fn remove_item(connection: Database, data: Json<DeleteItem>) -> Result<(), DataError> {
    let Database(ref connection) = connection;
    models::delete_item(&connection, data.0.id)
}


fn main() {
    rocket::ignite()
        .attach(Database::fairing())
        .mount("/", routes![index, items, new_item, remove_item])
        .launch();
}

#[cfg(test)]
mod tests {}

