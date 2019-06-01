table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        url -> Varchar,
        body -> Text,
        tags -> Jsonb,
    }
}
