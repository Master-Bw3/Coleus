use anyhow::{Context, Error, Result};
use mdbook::{book::Book, preprocess::{self, Preprocessor, PreprocessorContext}};
use regex::Regex;
use serde::Deserialize;

pub struct Coleus;

impl Coleus {
    pub fn new() -> Coleus {
        Coleus
    }
}

impl Preprocessor for Coleus {
    fn name(&self) -> &str {
        "coleus preprocessor"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(|item| match item  {
            mdbook::BookItem::Chapter(chapter) => preprocess_chapter(chapter),
            mdbook::BookItem::Separator => (),
            mdbook::BookItem::PartTitle(_) => (),
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn preprocess_chapter(chapter: &mut mdbook::book::Chapter) {
    let page = chapter.content.clone();

    // get json metadata
    let regex = Regex::new(r"``` *json\n?((\n|.)*)```").unwrap();

    let json_str = regex
        .captures_iter(&page)
        .next()
        .expect("no match found")
        .get(1)
        .expect("no match found")
        .as_str();

    let metadata: PageMetadata = serde_json::from_str(json_str).unwrap();


    // remove json metadata
    
    chapter.content = regex.replace_all(&page, "").to_string();
    
    // add title to page

    chapter.content = format!("# {}\n{}", metadata.title, chapter.content)

    // remap templates

    // remap owo-ui xml
}

#[derive(Deserialize)]
struct PageMetadata {
    title: String,
}