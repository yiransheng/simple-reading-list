table! {
    bookmarks (id) {
        id -> Int4,
        created -> Timestamp,
        title -> Varchar,
        url -> Varchar,
        body -> Text,
        tags -> Jsonb,
    }
}
