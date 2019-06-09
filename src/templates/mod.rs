use horrorshow::helper::doctype;
use horrorshow::{html, RenderOnce, Template, TemplateBuffer};

mod bookmark_item;

pub use bookmark_item::*;

pub struct PageTemplate<I> {
    items: I,
}

impl<I> PageTemplate<I> {
    pub fn new(items: I) -> Self {
        Self { items }
    }
}

impl<R, I> RenderOnce for PageTemplate<I>
where
    I: Iterator<Item = R>,
    R: RenderOnce,
{
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        tmpl << html! {
            : doctype::HTML;
            html {
                head {
                    title: "Reading list";
                    link(rel = "stylesheet",
                         type = "text/css",
                         href = "/static/site.css");
                }
                body {
                    @ for t in self.items {
                        |tmpl| tmpl << t
                    }
                }
            }
        };
    }
}
