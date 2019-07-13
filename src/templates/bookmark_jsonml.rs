use crate::jsonml::{AttrsBuilder, JsonML, JsonMLBuilder, MDParser};
use crate::models::Bookmark;

pub fn bookmark_jsonml<'a>(bookmark: &'a Bookmark) -> JsonML<'a> {
    let mut parser = MDParser::new(&bookmark.body);
    let body = match parser.jsonml() {
        Ok(jsonml) => jsonml,
        Err(_) => JsonMLBuilder::new()
            .append_text_node(bookmark.body.as_str())
            .build(),
    };
    JsonMLBuilder::new()
        .append_element_with_attrs(
            "div",
            AttrsBuilder::new().attr("class", "item").build(),
            |builder| {
                let link_attrs = AttrsBuilder::new()
                    .attr("class", "external")
                    .attr("href", bookmark.url.as_str())
                    .build();
                builder
                    .append_element_with_attrs("a", link_attrs, |builder| {
                        builder
                            .append_text_node(bookmark.title.as_str())
                            .build()
                    })
                    .append_element("div", move |_| body)
                    .build()
            },
        )
        .build()
}
