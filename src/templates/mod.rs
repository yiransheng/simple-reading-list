use horrorshow::helper::doctype;
use horrorshow::{html, Raw, RenderOnce, TemplateBuffer};

mod bookmark_item;
mod bookmark_jsonml;

pub use bookmark_item::*;
pub use bookmark_jsonml::bookmark_jsonml;

pub struct PageTemplate<I> {
    next_page: Option<i64>,
    query_str: Option<String>,
    items: I,
}

impl<I> PageTemplate<I> {
    pub fn new(items: I) -> Self {
        Self {
            next_page: None,
            query_str: None,
            items,
        }
    }
    pub fn new_with_query(items: I, q: String) -> Self {
        Self {
            next_page: None,
            query_str: Some(q),
            items,
        }
    }
    pub fn new_with_next_page(next_page: Option<i64>, items: I) -> Self {
        Self {
            next_page,
            items,
            query_str: None,
        }
    }
}

impl<R, I> RenderOnce for PageTemplate<I>
where
    I: Iterator<Item = R>,
    R: RenderOnce,
{
    fn render_once(self, tmpl: &mut TemplateBuffer) {
        let Self {
            items,
            query_str,
            next_page,
        } = self;
        let q = match query_str {
            Some(ref q) => q,
            None => "",
        };

        let mut count: i32 = 0;

        tmpl << html! {
            : doctype::HTML;
            html {
                head {
                    title: "Insightful Reads";
                    link(rel = "stylesheet",
                         type = "text/css",
                         href = "/static/site.css");
                }
                body {
                    header {
                        h1 {
                            a(href = "/"): "Insightful Reads"
                        }
                        form(action = "/search", method = "GET") {
                            div {
                                : Raw(ICON);
                                input(type = "text", name = "q", value = q);
                                input(type = "submit", value = "検索");
                            }
                        }
                    }
                    div(class = "main") {
                        @ for t in items {
                            |tmpl| {
                                count += 1;
                                tmpl << t
                            }
                        }
                        @ if let Some(next_page) = next_page {
                            div(class = "item") {
                                a(href = "#", data-next-page = next_page) {
                                    : "More >>"
                                }
                            }
                        }
                        @ if count == 0 {
                            div(class = "item empty") {
                                p {
                                   : "Empty results."
                                }
                            }
                        }
                    }
                    script(src = "/static/js/js-enhance.umd.js") {}
                }
            }
        };
    }
}

const ICON: &str = {
    r#"
    <svg aria-hidden="true" version="1.1" xmlns="http://www.w3.org/2000/svg" style="display: none;">
       <defs>
	  <symbol id="icon-search" viewBox="0 0 32 32">
	     <title>search</title>
	     <path d="M31.008 27.231l-7.58-6.447c-0.784-0.705-1.622-1.029-2.299-0.998 1.789-2.096 2.87-4.815 2.87-7.787 0-6.627-5.373-12-12-12s-12 5.373-12 12 5.373 12 12 12c2.972 0 5.691-1.081 7.787-2.87-0.031 0.677 0.293 1.515 0.998 2.299l6.447 7.58c1.104 1.226 2.907 1.33 4.007 0.23s0.997-2.903-0.23-4.007zM12 20c-4.418 0-8-3.582-8-8s3.582-8 8-8 8 3.582 8 8-3.582 8-8 8z"></path>
	  </symbol>
       </defs>
    </svg>
    <svg class="icon icon-search" aria-hidden="true">
       <use xlink:href=" #icon-search"></use>
    </svg>
    "#
};
