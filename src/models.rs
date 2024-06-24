use diesel::prelude::*;
use diesel::{Queryable, Insertable};
use serde::{Serialize, Deserialize};
use crate::schema::users;

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone, AsChangeset, Default)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub telegram_id: i64,
    pub name: String,
    pub interests: Vec<String>,
    pub goals: Vec<String>,
}

impl User {
    pub fn create(conn: &PgConnection, user: User) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(conn)
    }

    pub fn read(conn: &PgConnection, user_id: i32) -> QueryResult<User> {
        users::table.find(user_id).first(conn)
    }

    pub fn update(conn: &PgConnection, user_id: i32, user: User) -> QueryResult<User> {
        diesel::update(users::table.find(user_id))
            .set(&user)
            .get_result(conn)
    }

    pub fn delete(conn: &PgConnection, user_id: i32) -> QueryResult<usize> {
        diesel::delete(users::table.find(user_id))
            .execute(conn)
    }

    pub fn find_or_create_by_telegram_id(conn: &PgConnection, user_telegram_id: i64, user_name: &str) -> QueryResult<User> {
        use crate::schema::users::dsl::*;
    
        users.filter(telegram_id.eq(user_telegram_id))
            .first(conn)
            .or_else(|_| {
                let new_user = User {
                    id: 0, // This will be ignored by Diesel
                    telegram_id: user_telegram_id, // Use the parameter, not the column name
                    name: user_name.to_string(),
                    interests: Vec::new(),
                    goals: Vec::new(),
                };
                diesel::insert_into(users)
                    .values(&new_user)
                    .get_result(conn)
            })
    }
    
}
