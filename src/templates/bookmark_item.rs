use std::io;

use crate::models::{Bookmark, BookmarkDoc};
use horrorshow::{html, RenderOnce, TemplateBuffer};
use pulldown_cmark::{html, Parser};

pub trait IntoBookmark {
    fn into_bookmark(self) -> Bookmark;
}

impl IntoBookmark for Bookmark {
    fn into_bookmark(self) -> Bookmark {
        self
    }
}
impl IntoBookmark for BookmarkDoc {
    fn into_bookmark(self) -> Bookmark {
        self.into_bookmark_lossy()
    }
}

pub struct BookmarkItem {
    data: Bookmark,
}
impl BookmarkItem {
    pub fn new<B: IntoBookmark>(b: B) -> Self {
        Self {
            data: b.into_bookmark(),
        }
    }
}

impl RenderOnce for BookmarkItem {
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        let Bookmark {
            title, url, body, ..
        } = self.data;

        tmpl << html! {
            div(class = "item") {
                h2 {
                    a(href = url, class = "external") {
                      : title
                    }
                }
                div {
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
