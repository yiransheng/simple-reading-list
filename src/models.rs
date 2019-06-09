use std::collections::HashSet;
use std::fmt;
use std::io::Write;
use std::marker::PhantomData;

use chrono::naive::NaiveDateTime;
use diesel::deserialize::{self, FromSql};
use diesel::not_none;
use diesel::pg::types::sql_types::Jsonb;
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::de::{
    self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor,
};
use serde_derive::*;

use crate::schema::{bookmarks, users};

#[derive(Debug, Clone, Queryable)]
pub struct User {
    pub id: i32,
    pub created: NaiveDateTime,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlimUser {
    pub email: String,
    pub is_admin: bool,
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        SlimUser {
            email: user.email,
            is_admin: user.is_admin,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageData<T> {
    pub data: Vec<T>,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Queryable, Deserialize, Serialize, PartialEq)]
pub struct Bookmark {
    pub id: i32,
    pub created: NaiveDateTime,
    pub title: String,
    pub url: String,
    pub body: String,
    pub tags: TagSet,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[table_name = "bookmarks"]
pub struct NewBookmark {
    pub title: String,
    pub url: String,
    pub body: String,
    pub tags: TagSet,
}

#[derive(Debug, Clone, Queryable, Serialize)]
pub struct BookmarkDoc {
    pub id: i32,
    pub created: NaiveDateTime,
    pub title: String,
    pub url: String,
    pub body: String,
    pub tags: String,
}

impl From<Bookmark> for BookmarkDoc {
    fn from(b: Bookmark) -> Self {
        let Bookmark {
            id,
            created,
            title,
            url,
            body,
            tags,
        } = b;
        BookmarkDoc {
            id,
            created,
            title,
            url,
            body,
            tags: tags.join(" "),
        }
    }
}

impl BookmarkDoc {
    pub fn to_bookmark_lossy(self) -> Bookmark {
        let BookmarkDoc {
            id,
            created,
            title,
            url,
            body,
            ..
        } = self;
        Bookmark {
            id,
            created,
            title,
            url,
            body,
            tags: TagSet::default(),
        }
    }
}

impl<'de> Deserialize<'de> for BookmarkDoc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Id,
            Created,
            Title,
            Url,
            Body,
            Tags,
        }

        struct UnitArray<T>(T);

        impl<'de, T> Deserialize<'de> for UnitArray<T>
        where
            T: Deserialize<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(UnitArrayVisitor::new())
            }
        }

        struct UnitArrayVisitor<T>(PhantomData<T>);

        impl<T> UnitArrayVisitor<T> {
            fn new() -> Self {
                Self(PhantomData)
            }
        }

        impl<'de, T> Visitor<'de> for UnitArrayVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = UnitArray<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Deserialize [a] as a.")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<UnitArray<T>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let value = seq
                    .next_element::<T>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                if seq.next_element::<T>()?.is_some() {
                    return Err(de::Error::invalid_length(1, &self));
                }

                Ok(UnitArray(value))
            }
        }

        struct BookmarkDocVisitor;

        impl<'de> Visitor<'de> for BookmarkDocVisitor {
            type Value = BookmarkDoc;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BookmarkDoc")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id: Option<UnitArray<_>> = None;
                let mut created: Option<UnitArray<_>> = None;
                let mut title: Option<UnitArray<_>> = None;
                let mut url: Option<UnitArray<_>> = None;
                let mut body: Option<UnitArray<_>> = None;
                let mut tags: Option<UnitArray<_>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value::<UnitArray<_>>()?);
                        }
                        Field::Created => {
                            if created.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "created",
                                ));
                            }
                            created = Some(map.next_value::<UnitArray<_>>()?);
                        }
                        Field::Title => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "title",
                                ));
                            }
                            title = Some(map.next_value::<UnitArray<_>>()?);
                        }
                        Field::Url => {
                            if url.is_some() {
                                return Err(de::Error::duplicate_field("url"));
                            }
                            url = Some(map.next_value::<UnitArray<_>>()?);
                        }
                        Field::Body => {
                            if body.is_some() {
                                return Err(de::Error::duplicate_field("body"));
                            }
                            body = Some(map.next_value::<UnitArray<_>>()?);
                        }
                        Field::Tags => {
                            if tags.is_some() {
                                return Err(de::Error::duplicate_field("tags"));
                            }
                            tags = Some(map.next_value::<UnitArray<_>>()?);
                        }
                    }
                }
                let id = id
                    .map(|x| x.0)
                    .ok_or_else(|| de::Error::missing_field("id"))?;
                let created = created
                    .map(|x| x.0)
                    .ok_or_else(|| de::Error::missing_field("created"))?;
                let title = title
                    .map(|x: UnitArray<String>| x.0)
                    .ok_or_else(|| de::Error::missing_field("title"))?;
                let url = url
                    .map(|x| x.0)
                    .ok_or_else(|| de::Error::missing_field("url"))?;
                let body = body
                    .map(|x| x.0)
                    .ok_or_else(|| de::Error::missing_field("body"))?;
                let tags = tags
                    .map(|x| x.0)
                    .ok_or_else(|| de::Error::missing_field("tags"))?;
                Ok(BookmarkDoc {
                    id,
                    created,
                    title,
                    url,
                    body,
                    tags,
                })
            }
        }

        const FIELDS: &'static [&'static str] =
            &["id", "created", "title", "url", "body", "tags"];

        deserializer.deserialize_struct(
            "BookmarkDoc",
            FIELDS,
            BookmarkDocVisitor,
        )
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TagSet(HashSet<String>);

impl TagSet {
    fn join(&self, sep: &str) -> String {
        itertools::join(self.0.iter(), sep)
    }
}

// traits implements below

#[allow(dead_code)]
mod foreign_derives {
    use super::{Jsonb, TagSet};

    #[derive(FromSqlRow, AsExpression)]
    #[diesel(foreign_derive)]
    #[sql_type = "Jsonb"]
    struct TagSetProxy(TagSet);
}

impl FromSql<Jsonb, Pg> for TagSet {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let bytes = not_none!(bytes.and_then(seek_json_start));

        serde_json::from_slice(bytes).map_err(Into::into)
    }
}

// for some reason, diesel jsonb bytes starts with byte 1, seek subslice starts with "{" | "["
#[inline]
fn seek_json_start(bytes: &[u8]) -> Option<&[u8]> {
    for i in 0..bytes.len() {
        if bytes[i] == b'[' || bytes[i] == b'{' {
            return Some(&bytes[i..]);
        }
    }
    None
}

impl ToSql<Jsonb, Pg> for TagSet {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        // prefix jsonb version num.
        out.write(&[1])?;
        serde_json::to_writer(out, self)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_tags() {
        let json = r#"["foo", "bar"]"#;
        let tag_set: TagSet = serde_json::from_str(json).expect("Parse error");
        let expected = TagSet(
            vec!["foo", "bar"].iter().map(ToString::to_string).collect(),
        );

        assert_eq!(tag_set, expected);
    }

    #[test]
    fn test_from_sql() {
        // first byte in json is jsonb ver number
        let mut json: Vec<u8> = vec![1];
        json.extend(r#"["foo", "bar"]"#.as_bytes());

        let tag_set: TagSet =
            TagSet::from_sql(Some(&json)).expect("Parse error");
        let expected = TagSet(
            vec!["foo", "bar"].iter().map(ToString::to_string).collect(),
        );

        assert_eq!(tag_set, expected);
    }

    #[test]
    fn test_sql_round_trip() {
        // TODO: construct Pg Output
        assert!(true);
    }

    #[test]
    fn test_bookmark_de() {
        let json = r#"{
	  "id": 2,
	  "created": "2019-06-02T10:39:20.840523",
	  "title": "second",
	  "url": "http://ok",
	  "body": "world",
	  "tags": [
	    "bar",
	    "foo"
	  ]
	}"#;
        let bookmark: Result<Bookmark, _> = serde_json::from_str(json);
        assert!(bookmark.is_ok());
    }

    #[test]
    fn test_bookmark_doc_de() {
        let json1 = r#"{
	  "id": 2,
	  "created": "2019-06-02T10:39:20.840523",
	  "title": "second",
	  "url": "http://ok",
	  "body": "world",
	  "tags": []
	}"#;
        let json2 = r#"{
	  "id": [2],
	  "created": ["2019-06-02T10:39:20.840523"],
	  "title": ["second"],
	  "url": ["http://ok"],
	  "body": ["world"],
	  "tags": ["foo bar"]
	}"#;

        let bookmark: Result<Bookmark, _> = serde_json::from_str(json1);
        let bookmark_doc: Result<BookmarkDoc, _> = serde_json::from_str(json2);

        assert!(bookmark_doc.is_ok());
        assert_eq!(
            bookmark.unwrap(),
            bookmark_doc.unwrap().to_bookmark_lossy()
        );
    }
}
