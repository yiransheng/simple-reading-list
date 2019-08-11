use std::iter::Peekable;

use pulldown_cmark::{CowStr, Event, LinkType, Parser, Tag};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde_derive::*;

#[derive(Debug)]
pub struct JsonML<'a> {
    elements: Vec<Element<'a>>,
}

pub struct JsonMLBuilder<'a> {
    elements: Vec<Element<'a>>,
}
impl<'a> JsonMLBuilder<'a> {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }
    pub fn build(self) -> JsonML<'a> {
        JsonML {
            elements: self.elements,
        }
    }
    pub fn append_text_node<T>(mut self, contents: T) -> Self
    where
        T: Into<CowStr<'a>>,
    {
        let el = Element::Text(contents.into());
        self.elements.push(el);
        self
    }
    pub fn append_element<F>(mut self, tag: &'static str, f: F) -> Self
    where
        F: FnOnce(JsonMLBuilder<'a>) -> JsonML<'a>,
    {
        let builder = JsonMLBuilder::new();
        let children = f(builder).elements;
        let element = Element::Tree(tag, None, children);
        self.elements.push(element);
        self
    }
    pub fn append_element_with_attrs<F>(
        mut self,
        tag: &'static str,
        attrs: Attrs<'a>,
        f: F,
    ) -> Self
    where
        F: FnOnce(JsonMLBuilder<'a>) -> JsonML<'a>,
    {
        let builder = JsonMLBuilder::new();
        let children = f(builder).elements;
        let element = Element::Tree(tag, Some(attrs), children);
        self.elements.push(element);
        self
    }
}

#[derive(Debug)]
pub enum Element<'a> {
    Text(CowStr<'a>),
    Tree(&'static str, Option<Attrs<'a>>, Vec<Element<'a>>),
    SelfClosing(&'static str, Option<Attrs<'a>>),
}

#[derive(Debug)]
pub struct Attrs<'a> {
    attrs: Vec<Attr<'a>>,
}

pub struct AttrsBuilder<'a> {
    attrs: Vec<Attr<'a>>,
}
impl<'a> AttrsBuilder<'a> {
    pub fn new() -> Self {
        Self { attrs: vec![] }
    }
    pub fn build(self) -> Attrs<'a> {
        Attrs { attrs: self.attrs }
    }
    pub fn attr<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<CowStr<'a>>,
        V: Into<CowStr<'a>>,
    {
        let key = key.into();
        let value = value.into();
        self.attrs.push(Attr { key, value });
        self
    }
}

#[derive(Debug)]
struct Attr<'a> {
    key: CowStr<'a>,
    value: CowStr<'a>,
}

#[derive(Debug, Serialize)]
pub enum ParseError {
    EOF,
    ComplexTag,
    Expects(&'static str),
    Unexpected(&'static str),
    NotSupported(&'static str),
    Custom(&'static str),
}

pub struct MDParser<I: Iterator> {
    events: Peekable<I>,
}

type IResult<T> = Result<T, ParseError>;

impl<'a> MDParser<Parser<'a>> {
    pub fn new(raw: &'a str) -> Self {
        let parser = Parser::new(raw);
        Self {
            events: parser.peekable(),
        }
    }
    #[inline(always)]
    pub fn jsonml(&mut self) -> IResult<JsonML<'a>> {
        Ok(JsonML {
            elements: self.elements()?,
        })
    }
}

impl<'a, I> MDParser<I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn current(&mut self) -> IResult<&Event<'a>> {
        self.events.peek().ok_or_else(|| ParseError::EOF)
    }
    fn advance(&mut self) -> IResult<Event<'a>> {
        self.events.next().ok_or_else(|| ParseError::EOF)
    }
    fn elements(&mut self) -> IResult<Vec<Element<'a>>> {
        let mut elements = vec![];
        let mut i = 0;
        while let Some(el) = self.maybe_element()? {
            elements.push(el);
            i += 1;
            // just in case
            if i > 10000 {
                panic!("Possible infinite loop! (MDParser)");
            }
        }

        Ok(elements)
    }
    fn maybe_element(&mut self) -> IResult<Option<Element<'a>>> {
        let current = match self.current() {
            Ok(event) => event,
            Err(ParseError::EOF) => return Ok(None),
            Err(_) => unreachable!(),
        };

        let el = match current {
            Event::Text(_) => self.text()?,
            Event::Start(_) => self.tag()?,
            Event::SoftBreak => {
                self.advance()?;
                Element::Text("\n".into())
            }
            Event::HardBreak => {
                self.advance()?;
                Element::SelfClosing("hr", None)
            }
            Event::Code(_) => {
                if let Event::Code(code) = self.advance()? {
                    Element::Tree("code", None, vec![Element::Text(code)])
                } else {
                    unreachable!()
                }
            }
            Event::End(_) => return Ok(None),
            Event::FootnoteReference(_)
            | Event::Html(_)
            | Event::InlineHtml(_) => {
                self.advance()?;
                Element::Text("".into())
            }
            Event::TaskListMarker(true) => {
                self.advance()?;
                let attrs = Attrs {
                    attrs: vec![
                        Attr {
                            key: "type".into(),
                            value: "checkbox".into(),
                        },
                        Attr {
                            key: "checked".into(),
                            value: "".into(),
                        },
                    ],
                };
                Element::SelfClosing("input", Some(attrs))
            }
            Event::TaskListMarker(false) => {
                self.advance()?;
                let attrs = Attrs {
                    attrs: vec![Attr {
                        key: "type".into(),
                        value: "checkbox".into(),
                    }],
                };
                Element::SelfClosing("input", Some(attrs))
            }
        };

        Ok(Some(el))
    }

    fn text(&mut self) -> IResult<Element<'a>> {
        match self.advance()? {
            Event::Text(contents) => Ok(Element::Text(contents)),
            _ => Err(ParseError::Expects("text")),
        }
    }
    fn tag(&mut self) -> IResult<Element<'a>> {
        match self.peek_basic_tag() {
            Ok(tag) => self.basic_tag(tag),
            Err(ParseError::ComplexTag) => self.complex_tag(),
            Err(err) => Err(err),
        }
    }
    fn peek_basic_tag(&mut self) -> IResult<&'static str> {
        match self.current()? {
            Event::Start(ref tag) => to_basic_tag(tag),
            _ => Err(ParseError::Expects("tag open")),
        }
    }
    fn complex_tag(&mut self) -> IResult<Element<'a>> {
        match self.advance()? {
            Event::Start(tag) => match tag {
                Tag::Rule => Ok(Element::SelfClosing("hr", None)),
                Tag::List(Some(1)) => {
                    let elements = self.elements()?;
                    let el = Element::Tree("ol", None, elements);
                    self.expects_close()?;
                    self.advance()?;
                    Ok(el)
                }
                Tag::List(Some(start)) => {
                    let elements = self.elements()?;
                    let attr = Attr {
                        key: "start".into(),
                        value: start.to_string().into(),
                    };
                    let el = Element::Tree(
                        "ol",
                        Some(Attrs { attrs: vec![attr] }),
                        elements,
                    );
                    self.expects_close()?;
                    self.advance()?;
                    Ok(el)
                }
                Tag::Link(ty, dest, title) => self.link(ty, dest, title),
                Tag::Image(ty, dest, title) => self.image(ty, dest, title),
                Tag::CodeBlock(info) => self.code(info),
                _ => Err(ParseError::Expects("open tag")),
            },
            _ => Err(ParseError::Expects("open tag")),
        }
    }
    fn link(
        &mut self,
        ty: LinkType,
        dest: CowStr<'a>,
        _title: CowStr<'a>,
    ) -> IResult<Element<'a>> {
        if let LinkType::Email = ty {
            return Err(ParseError::NotSupported("mailto"));
        }

        let attrs = Attrs {
            attrs: vec![Attr {
                key: "href".into(),
                value: dest,
            }],
        };
        let elements = self.elements()?;
        self.expects_close()?;
        self.advance()?;

        Ok(Element::Tree("a", Some(attrs), elements))
    }
    fn image(
        &mut self,
        ty: LinkType,
        dest: CowStr<'a>,
        title: CowStr<'a>,
    ) -> IResult<Element<'a>> {
        if let LinkType::Email = ty {
            return Err(ParseError::NotSupported("mailto"));
        }

        let attrs = Attrs {
            attrs: vec![
                Attr {
                    key: "src".into(),
                    value: dest,
                },
                Attr {
                    key: "alt".into(),
                    value: title,
                },
            ],
        };

        Ok(Element::SelfClosing("img", Some(attrs)))
    }
    fn code(&mut self, _info: CowStr<'a>) -> IResult<Element<'a>> {
        let code = self.text()?;
        self.expects_close()?;

        Ok(Element::Tree(
            "pre",
            None,
            vec![Element::Tree("code", None, vec![code])],
        ))
    }

    fn basic_tag(&mut self, open_tag: &'static str) -> IResult<Element<'a>> {
        self.advance()?;
        let elements = self.elements()?;
        match self.current()? {
            Event::End(ref tag) => {
                let close_tag = to_basic_tag(tag)?;
                if close_tag == open_tag {
                    self.advance()?;
                    Ok(Element::Tree(open_tag, None, elements))
                } else {
                    Err(ParseError::Unexpected(close_tag))
                }
            }
            _ => Err(ParseError::Expects("tag close")),
        }
    }
    fn expects_close(&mut self) -> IResult<()> {
        match self.current()? {
            Event::End(_) => Ok(()),
            _ => Err(ParseError::Expects("close tag")),
        }
    }
}

fn to_basic_tag(tag: &Tag) -> IResult<&'static str> {
    let tag = match tag {
        Tag::Paragraph => "p",
        // does not support h1, h2
        Tag::Header(3) => "h3",
        Tag::Header(4) => "h4",
        Tag::Header(5) => "h5",
        Tag::Header(6) => "h6",
        Tag::Header(_) => "h3",
        Tag::BlockQuote => "blockquote",
        Tag::Strong => "strong",
        Tag::Item => "li",
        Tag::List(None) => "ul",
        Tag::Emphasis => "em",
        Tag::Strikethrough => "del",
        // not supported
        Tag::FootnoteDefinition(_) => {
            return Err(ParseError::NotSupported("footnote"))
        }
        Tag::HtmlBlock => return Err(ParseError::NotSupported("html")),
        Tag::Table(_) => return Err(ParseError::NotSupported("table")),
        Tag::TableRow => return Err(ParseError::NotSupported("tr")),
        Tag::TableCell => return Err(ParseError::NotSupported("td")),
        _ => return Err(ParseError::ComplexTag),
    };

    Ok(tag)
}

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
                Element::SelfClosing("input", None),
            ],
        );
        let serialized = serde_json::to_string_pretty(&el).unwrap();
        let js_value: serde_json::Value =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            js_value,
            serde_json::json!([
              "ul",
                ["li", "first"],
                ["li", "second"],
                ["li", ["a", {"href": "https://www.google.com"}, "Google"]],
                ["input"],
            ])
        );
    }

    #[test]
    fn test_from_md_events() {
        let raw = r#"### Header 3
This `inline` is paragraph, with [Link](https://www.google.com).

```
fn test() {}
```

* Foo
* Bar
* Baz
    - Baz 1
    - Baz 2
"#;

        let jsonml = MDParser::new(raw).jsonml().unwrap();
        let serialized = serde_json::to_string_pretty(&jsonml).unwrap();
        eprintln!("{}", serialized);
        let js_value: serde_json::Value =
            serde_json::from_str(&serialized).unwrap();
        let expected = serde_json::json!(
        [
          [
            "h3",
            "Header 3"
          ],
          [
            "p",
            "This ",
            [
              "code",
              "inline"
            ],
            " is paragraph, with ",
            [
              "a",
              {
                "href": "https://www.google.com"
              },
              "Link"
            ],
            "."
          ],
          [
            "pre",
            [
              "code",
              "fn test() {}\n"
            ]
          ]
        ]);

        assert_eq!(expected, js_value);
    }
}
