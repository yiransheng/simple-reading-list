use std::io;

use crate::models::{Bookmark, BookmarkDoc};
use horrorshow::{html, RenderOnce, Template, TemplateBuffer};
use pulldown_cmark::{html, Parser};

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
                p {
                  |buffer| {
                      let parser = Parser::new(&body);
                      let writer = WriteTemplateDirectly { buffer };
                      let _ = html::write_html(writer, parser);
                  }
                }
            }
        };
    }
}

struct WriteTemplateDirectly<'a, 't> {
    buffer: &'a mut TemplateBuffer<'t>,
}

// putting a lot of trust in pushdown-cmark and horrowshow to handle errors
// and escaping correctly
impl<'a, 't> io::Write for WriteTemplateDirectly<'a, 't> {
    // horroshow swallows error in write_raw, and deals with it
    // later..
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let raw_str = std::str::from_utf8(buf)
            .expect("expect utf8 string input to template");
        self.buffer.write_raw(raw_str);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
