table! {
    bookmarks (id) {
        id -> Int4,
        created -> Timestamp,
        title -> Varchar,
        url -> Varchar,
        body -> Text,
        tags -> Jsonb,
        toshi_index -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Int4,
        created -> Timestamp,
        email -> Varchar,
        password -> Varchar,
        is_admin -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    bookmarks,
    users,
);
