use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;
use serde::Deserialize;
use serde_json::Error;

#[derive(Deserialize, Debug)]
pub struct PageMetadata {
    pub title: String,
}

#[derive(PartialEq)]
enum ParseState {
    BeforeMetadata,
    DuringMetadata,
    AfterMetadata,
}

pub fn strip_metadata(chapter: &str) -> Result<(PageMetadata, String), Error> {
    let parser = Parser::new(chapter);

   
    let mut code_text: String = String::new();
    let mut new_parser: Vec<Event> = Vec::new();

    let mut parse_state = ParseState::BeforeMetadata;

    for ref event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language))) => match parse_state {
                ParseState::BeforeMetadata if language.to_string() == "json" => {
                    parse_state = ParseState::DuringMetadata
                }

                ParseState::BeforeMetadata => new_parser.push(event.clone()),

                ParseState::AfterMetadata => new_parser.push(event.clone()),

                _ => (),
            },

            Event::Text(text) => match parse_state {
                ParseState::DuringMetadata => code_text = code_text + text,
                _ => new_parser.push(event.clone()),
            },

            Event::End(TagEnd::CodeBlock) => match parse_state {
                ParseState::DuringMetadata => parse_state = ParseState::AfterMetadata,

                _ => new_parser.push(event.clone()),
            },

            _ => match parse_state {
                ParseState::DuringMetadata => (),
                _ => new_parser.push(event.clone())
            },
        }
    }


    let mut buf = String::with_capacity(chapter.len());

    serde_json::from_str(&code_text).map(|x| 
        (x, cmark(new_parser.iter(), &mut buf).map(|_| buf).unwrap()))
}
