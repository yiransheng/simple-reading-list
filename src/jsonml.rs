use std::iter::FromIterator;

use pulldown_cmark::{CowStr, Event, Parser, Tag};
use serde::ser::{Serialize, SerializeSeq, Serializer};

pub struct JsonML<'a> {
    elements: Vec<Element<'a>>,
}

pub enum Element<'a> {
    Text(CowStr<'a>),
    Tree(&'static str, Option<Attrs<'a>>, Vec<Element<'a>>),
    SelfClosing(&'static str, Option<Attrs<'a>>),
}

pub struct Attrs<'a> {
    attrs: Vec<Attr<'a>>,
}

struct Attr<'a> {
    key: CowStr<'a>,
    value: CowStr<'a>,
}

pub struct MDError {}

// impl<'a> FromIterator<Event<'a>> for JsonML<'a> {
// fn from_iter<T>(iter: T) -> Self
// where
// T: IntoIterator<Item = Event<'a>>,
// {

// }
// }

// fn text<'a, I>(iter: &mut I) -> Result<Element<'a>, MDError>
// where I: Iterator<Item = Event<'a>>,
// {
// match iter.next() {
// Some(Event::Text(t)) => Ok(Element::Text(t)),
// Some(Event::
// }

// }

// fn convert_tag<'a>(tag: Tag<'a>) -> &'static str {
// match tag {
// Tag::Paragraph => "p",
// Tag::Rule => "hr",
// Tag::Header(1) => "h1",
// Tag::Header(2) => "h2",
// Tag::Header(3) => "h3",
// Tag::Header(4) => "h4",
// Tag::Header(5) => "h5",
// Tag::Header(_) => "h6",
// Tag::BlockQuote => "blockquote",
// Tag::CodeBlock(_) => "code",
// Tag::List(_) => "ul",
// Tag::Item => "li",
// Tag::Emphasis => "em",
// Tag::Strong => "strong",
// }
// }

impl<'a> Serialize for JsonML<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.elements.len()))?;
        for element in self.elements.iter() {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

impl<'a> Serialize for Attrs<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_map(
            self.attrs
                .iter()
                .map(|attr| (attr.key.as_ref(), attr.value.as_ref())),
        )
    }
}

impl<'a> Serialize for Element<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Element::Text(ref text) => serializer.serialize_str(text),
            Element::Tree(ref tag, ref attrs, ref children) => {
                let n = if attrs.is_some() {
                    children.len() + 2
                } else {
                    children.len() + 1
                };
                let mut seq = serializer.serialize_seq(Some(n))?;
                seq.serialize_element(tag)?;
                if let Some(ref attrs) = attrs {
                    seq.serialize_element(attrs)?;
                }
                for child in children {
                    seq.serialize_element(child)?;
                }
                seq.end()
            }
            Element::SelfClosing(ref tag, ref attrs) => {
                let n = if attrs.is_some() { 2 } else { 1 };
                let mut seq = serializer.serialize_seq(Some(n))?;
                seq.serialize_element(tag)?;
                if let Some(ref attrs) = attrs {
                    seq.serialize_element(attrs)?;
                }
                seq.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_serialization() {
        let el = Element::Tree(
            "ul",
            None,
            vec![
                Element::Tree(
                    "li",
                    None,
                    vec![Element::Text(CowStr::Borrowed("first"))],
                ),
                Element::Tree(
                    "li",
                    None,
                    vec![Element::Text(CowStr::Borrowed("second"))],
                ),
                Element::Tree(
                    "li",
                    None,
                    vec![Element::Tree(
                        "a",
                        Some(Attrs {
                            attrs: vec![Attr {
                                key: CowStr::Borrowed("href"),
                                value: CowStr::Borrowed(
                                    "https://www.google.com",
                                ),
                            }],
                        }),
                        vec![Element::Text(CowStr::Borrowed("Google"))],
                    )],
                ),
            ],
        );
        let serialized = serde_json::to_string_pretty(&el).unwrap();
        eprintln!("{}", serialized);
        let js_value: serde_json::Value =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            js_value,
            serde_json::json!([
              "ul",
                ["li", "first"],
                ["li", "second"],
                ["li", ["a", {"href": "https://www.google.com"}, "Google"]],
            ])
        );
    }
}
