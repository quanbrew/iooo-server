#![feature(proc_macro_hygiene, decl_macro)]
use std::env;
use dotenv::dotenv;
use postgres::{Connection, TlsMode};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use uuid::Uuid;
use serde_json::Value as JsonValue;
use serde_derive::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/item")]
fn items() -> Json<Vec<Item>> {
    let connection = establish_connection();
    let query = "
    SELECT
        L.id, R.id, L.content, L.fold, L.metadata, L.deleted,
        L.favorite, L.tags, L.created, L.modified
    FROM
        items L LEFT JOIN items R
        ON subpath(L.path, 0, -1) = R.path
    ORDER BY (R.id, L.path)";
    let mut items = vec![];
    for row in &connection.query(query, &[]).unwrap() {
        let item = Item {
            id: row.get(0),
            parent: row.get(1),
            content: row.get(2),
            fold: row.get(3),
            metadata: row.get(4),
            deleted: row.get(5),
            favorite: row.get(6),
            tags: row.get(7),
            created: row.get(8),
            modified: row.get(9),
        };
        items.push(item);
    }
    Json(items)
}


#[post("/item/new/<_id>", format = "application/json", data = "<item>")]
fn new_item(_id: String, item: Json<NewItem>) -> () {
    let connection = establish_connection();
    let Json(item) = item;
    let mut path: String;
    if let Some(parent) = item.parent {
        let parent_item_row = connection
            .query("SELECT ltree2text(path) FROM items WHERE id = $1", &[&parent]).unwrap();
        let parent_path: String = parent_item_row.into_iter().next().unwrap().get(0);
        path = parent_path + ".A";
    }
    else {
        path = item.id.simple().to_string();
    }
    let _ = connection.execute("
    INSERT INTO items (id, path, content, created, modified)
    VALUES (
        $1, $2, $3, TIMESTAMP 'now', TIMESTAMP 'now'
    )
    ON CONFLICT (id) DO UPDATE
        SET content = EXCLUDED.content
        SET modified = EXCLUDED.modified
    ", &[&item.id, &path, &item.content]).unwrap();
}



#[derive(Serialize, Deserialize, Debug)]
struct Item {
    id: Uuid,
    content: String,
    parent: Option<Uuid>,
    fold: bool,
    metadata: JsonValue,
    deleted: bool,
    favorite: bool,
    tags: Vec<String>,
    created: NaiveDateTime,
    modified: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
struct NewItem {
    id: Uuid,
    parent: Option<Uuid>,
    previous: Option<Uuid>,
    content: String,
    metadata: JsonValue,
}


pub fn establish_connection() -> Connection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    Connection::connect(database_url, TlsMode::None)
        .expect(&format!("Error connecting to Databse",))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, items])
        .launch();
}