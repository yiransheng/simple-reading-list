#[derive(Queryable)]
pub struct Post {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub body: String,
    pub tags: Vec<String>,
}
