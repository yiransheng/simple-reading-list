use horrorshow::{html, RenderOnce, Template, TemplateBuffer};

use crate::models::{Bookmark, BookmarkDoc};

pub trait IntoBookmarkData: Into<BmData> {}

impl<T: Into<BmData>> IntoBookmarkData for T {}

enum BmData {
    Bookmark(Bookmark),
    Doc(BookmarkDoc),
}
impl BmData {
    fn into_bookmark(self) -> Bookmark {
        match self {
            BmData::Bookmark(b) => b,
            BmData::Doc(b) => b.to_bookmark_lossy(),
        }
    }
}

impl From<Bookmark> for BmData {
    fn from(b: Bookmark) -> Self {
        BmData::Bookmark(b)
    }
}

impl From<BookmarkDoc> for BmData {
    fn from(b: BookmarkDoc) -> Self {
        BmData::Doc(b)
    }
}

pub struct BookmarkItem {
    data: BmData,
}
impl BookmarkItem {
    pub fn new<B: Into<BmData>>(b: B) -> Self {
        Self { data: b.into() }
    }
}

impl RenderOnce for BookmarkItem {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        let Bookmark {
            title,
            url,
            body,
            created,
            ..
        } = self.data.into_bookmark();
        tmpl << html! {
            div(class = "link-item") {
                h2 {
                    a(href = url) {
                      : title
                    }
                }
                p : body
            }
        };
    }
}
