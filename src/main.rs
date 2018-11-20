#![feature(proc_macro_hygiene, decl_macro)]

use dotenv::dotenv;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use self::models::{DataError, Item, NewItem};

mod models;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/item")]
fn items() -> Json<Vec<Item>> {
    let connection = models::establish_connection();
    Json(models::get_item_list(&connection))
}


fn uuid_to_label(uuid: Uuid) -> String {
    uuid.simple().to_string()
}


#[post("/item", format = "application/json", data = "<items>")]
fn new_item(items: Json<Vec<NewItem>>) -> Result<(), DataError> {
    let connection = models::establish_connection();
    let Json(items) = items;
    let transaction = connection.transaction().map_err(DataError::Database)?;
    for item in items {
        item.insert(&transaction)?;
    }
    transaction.commit().map_err(DataError::Database)?;
    Ok(())
}


fn main() {
    dotenv().ok();
    rocket::ignite()
        .mount("/", routes![index, items, new_item])
        .launch();
}

#[cfg(test)]
mod tests {}

