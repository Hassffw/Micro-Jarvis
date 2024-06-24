table! {
    users (id) {
        id -> Int4,
        telegram_id -> Int8,
        name -> Varchar,
        interests -> Array<Text>,
        goals -> Array<Text>,
    }
}