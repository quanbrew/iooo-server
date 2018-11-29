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
    pub expand: bool,
    pub metadata: JsonValue,
    pub tags: Vec<String>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateItem {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub previous: Option<Uuid>,
    pub content: String,
    pub metadata: JsonValue,
    pub expand: bool,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteItem {
    pub id: Uuid,
}


fn uuid_to_label(uuid: Uuid) -> String {
    use std::mem::transmute;

    let bytes: [u8; 16] = uuid.as_bytes().clone();
    let number: u128 = unsafe { transmute::<[u8; 16], u128>(bytes) };
    crate::base62::encode(number)
}


#[derive(Fail, Debug)]
pub enum DataError {
    #[fail(display = "Cannot parse: {}", _0)]
    CanNotParse(String),
    #[fail(display = "Not found item by id {}", _0)]
    NotFoundByUUID(Uuid),
    #[fail(display = "Not found item by path {}", _0)]
    NotFoundByPath(String),
    #[fail(display = "{}", _0)]
    Database(#[fail(cause)] DatabaseError),
}


impl UpdateItem {
    fn parent_path(transaction: &Transaction, id: Uuid) -> Result<String, DataError> {
        transaction
            .query(include_str!("get_parent_path.sql"), &[&id]) // do sql query
            .map_err(DataError::Database)? // handle query error
            .into_iter().next() // get first row
            .ok_or(DataError::NotFoundByUUID(id)) // ok, or no such row
            .map(|r| r.get(0)) // get first column
    }

    fn get_ranking(transaction: &Transaction, id: Uuid) -> Result<i32, DataError> {
        transaction.query(include_str!("get_ranking.sql"), &[&id])
            .map_err(DataError::Database)?
            .into_iter().next()
            .map(|r| r.get(0))
            .ok_or(DataError::NotFoundByUUID(id))
    }

    pub fn insert(self, transaction: &Transaction) -> Result<(), DataError> {
        let mut parent_path: String;
        let mut path: String;
        let mut ranking: i32 = 0;

        if let Some(parent) = self.parent {
            parent_path = UpdateItem::parent_path(transaction, parent)?;
            path = format!("{}.{}", parent_path, uuid_to_label(self.id));
            if let Some(previous) = self.previous {
                ranking = UpdateItem::get_ranking(transaction, previous)?;
            }
        } else {
            path = self.id.simple().to_string();
            parent_path = "".to_string();
        }
        if parent_path.len() > 0 {
            let children_path_query = format!("{}.*{{1}}", parent_path);
            let _ = transaction
                .execute(
                    include_str!("update_item_ranking.sql"),
                    &[&children_path_query, &ranking],
                )
                .map_err(DataError::Database)?;
        }
        let _ = transaction
            .execute(
                include_str!("insert_or_update.sql"),
                &[&self.id, &path, &self.content, &ranking, &self.expand],
            ).map_err(DataError::Database);
        Ok(())
    }
}


pub fn delete_item(connection: &Connection, id: Uuid) -> Result<(), DataError> {
    connection.execute(include_str!("delete_item.sql"), &[&id])
        .map(|_| ())
        .map_err(DataError::Database)
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
            expand: row.get(3),
            metadata: row.get(4),
            tags: row.get(5),
            created: row.get(6),
            modified: row.get(7),
        }).collect()
}