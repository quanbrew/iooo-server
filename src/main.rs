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
use failure::{Error, format_err};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/item")]
fn items() -> Result<Json<Vec<Item>>, Error> {
    let connection = establish_connection();
    let query = "
    SELECT
        L.id, R.id, L.content, L.fold, L.metadata, L.deleted,
        L.favorite, L.tags, L.created, L.modified
    FROM
        items L LEFT JOIN items R
        ON subpath(L.path, 0, -1) = R.path
    ORDER BY (R.id, L.path);";
    let mut items = vec![];
    for row in &connection.query(query, &[])? {
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
    Ok(Json(items))
}


fn uuid_to_label(uuid: Uuid) -> String {
    uuid.simple().to_string()
}


#[post("/item/<_id>", format = "application/json", data = "<item>")]
fn new_item(_id: String, item: Json<NewItem>) -> Result<(), Error> {
    let connection = establish_connection();
    let Json(item) = item;
    let mut parent_path: String;
    let mut path: String;
    let mut ranking: i32 = 0;

    let create = connection.transaction()?;
    if let Some(parent) = item.parent {
        let query = "SELECT ltree2text(path) FROM items
        WHERE path = (SELECT path FROM items WHERE id = $1);";
        let parent_item_row = create
            .query(query, &[&parent])?;
        let not_found = format_err!("not found parent");
        parent_path = parent_item_row
            .into_iter()
            .next()
            .ok_or(not_found)?
            .get(0);
        path = format!("{}.{}", parent_path, uuid_to_label(item.id));
        if let Some(previous) = item.previous {
            let query = "SELECT ranking + 1 FROM items WHERE id = $1;";
            if let Some(r) = create.query(query, &[&previous])?.into_iter().next() {
                ranking = r.get(0)
            }
        }
    }
    else {
        path = item.id.simple().to_string();
        parent_path = "".to_string();
    }
    if parent_path.len() > 0 {
        let _ = create.execute(
            "UPDATE items SET ranking = ranking + 1 WHERE path ~ (text($1))::lquery AND ranking >= $2;",
            &[&format!("{}.*{{1}}", parent_path), &ranking]
            )?;
    }
    let _ = create.execute("
    INSERT INTO items (id, path, content, ranking, created, modified)
    VALUES (
        $1, text2ltree($2), $3, $4, TIMESTAMP 'now', TIMESTAMP 'now'
    )
    ON CONFLICT (id) DO UPDATE
        SET content = EXCLUDED.content,
            modified = EXCLUDED.modified,
            path = EXCLUDED.path,
            ranking = EXCLUDED.ranking;
    ", &[&item.id, &path, &item.content, &ranking])?;
    create.commit()?;
    Ok(())
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
        .mount("/", routes![index, items, new_item])
        .launch();
}