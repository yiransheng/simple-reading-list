use std::collections::HashSet;
use std::io::Write;

use chrono::naive::NaiveDateTime;
use diesel::deserialize::{self, FromSql};
use diesel::not_none;
use diesel::pg::types::sql_types::Jsonb;
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
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

#[derive(Debug, Clone, Queryable, Serialize)]
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
}
