use crate::schema::users;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::Queryable;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub schema:i32,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub schema:i32,
}