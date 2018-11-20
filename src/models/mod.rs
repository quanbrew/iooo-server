use std::env;

use chrono::NaiveDateTime;
use failure::Fail;
use postgres::{Connection, Error as DatabaseError, TlsMode};
use postgres::transaction::Transaction;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

pub fn establish_connection() -> Connection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    Connection::connect(database_url, TlsMode::None)
        .expect(&format!("Error connecting to Database", ))
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: Uuid,
    pub content: String,
    pub parent: Option<Uuid>,
    pub fold: bool,
    pub metadata: JsonValue,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct NewItem {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub previous: Option<Uuid>,
    pub content: String,
    pub metadata: JsonValue,
}


fn uuid_to_label(uuid: Uuid) -> String {
    use std::mem::transmute;

    let bytes: [u8; 16] = uuid.as_bytes().clone();
    let number: u128 = unsafe { transmute::<[u8; 16], u128>(bytes) };
    crate::base62::encode(number)
}


#[derive(Fail, Debug)]
pub enum DataError {
    #[fail(display = "Not found item by id {}", _0)]
    NotFoundByUUID(Uuid),
    #[fail(display = "Not found item by path {}", _0)]
    NotFoundByPath(String),
    #[fail(display = "{}", _0)]
    Database(#[fail(cause)] DatabaseError),
}


impl NewItem {
    pub fn insert(self, create: &Transaction) -> Result<(), DataError> {
        let mut parent_path: String;
        let mut path: String;
        let mut ranking: i32 = 0;

        if let Some(parent) = self.parent {
            let parent_item_row = create
                .query(include_str!("get_parent_path.sql"), &[&parent])
                .map_err(DataError::Database)?;
            parent_path = parent_item_row
                .into_iter()
                .next()
                .ok_or(DataError::NotFoundByUUID(parent))?
                .get(0);
            path = format!("{}.{}", parent_path, uuid_to_label(self.id));
            if let Some(previous) = self.previous {
                let query = include_str!("get_ranking.sql");
                create.query(query, &[&previous])
                    .map_err(DataError::Database)?
                    .into_iter().next()
                    .map(|r| ranking = r.get(0));
            }
        } else {
            path = self.id.simple().to_string();
            parent_path = "".to_string();
        }
        if parent_path.len() > 0 {
            let children_path_query = format!("{}.*{{1}}", parent_path);
            let _ = create
                .execute(
                    include_str!("update_item_ranking.sql"),
                    &[&children_path_query, &ranking],
                )
                .map_err(DataError::Database)?;
        }
        let _ = create
            .execute(
                include_str!("insert_or_update.sql"),
                &[&self.id, &path, &self.content, &ranking],
            ).map_err(DataError::Database);
        Ok(())
    }
}


pub fn get_item_list(connection: &Connection) -> Vec<Item> {
    let query = include_str!("get_item_list.sql");
    connection.query(query, &[])
        .expect("database query failure on get item list")
        .iter()
        .map(|row| Item {
            id: row.get(0),
            parent: row.get(1),
            content: row.get(2),
            fold: row.get(3),
            metadata: row.get(4),
            favorite: row.get(5),
            tags: row.get(6),
            created: row.get(7),
            modified: row.get(8),
        }).collect()
}