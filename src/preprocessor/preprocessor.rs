use std::{collections::HashMap, path::Path};

use anyhow::{Context, Error, Result};
use mdbook::{
    book::{Book, Chapter},
    preprocess::{self, Preprocessor, PreprocessorContext},
    BookItem,
};
use path_slash::PathBufExt;
use pathdiff::diff_paths;
use pulldown_cmark::{Event, Parser};
use pulldown_cmark_to_cmark::cmark;
use regex::Regex;
use serde::Deserialize;

use crate::config::ColeusConfig;

use super::parser::{strip_metadata, PageMetadata};

pub struct Coleus {
    config: ColeusConfig,
}

impl Preprocessor for Coleus {
    fn name(&self) -> &str {
        "coleus preprocessor"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let sections = book.sections.clone();

        book.for_each_mut(|item| match item {
            mdbook::BookItem::Chapter(chapter) => self.preprocess_chapter(&sections, chapter),
            mdbook::BookItem::Separator => (),
            mdbook::BookItem::PartTitle(_) => (),
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

impl Coleus {
    pub fn new(config: ColeusConfig) -> Coleus {
        Coleus { config }
    }

    fn preprocess_chapter(&self, sections: &Vec<BookItem>, chapter: &mut mdbook::book::Chapter) {
        // get json metadata and remove it
        let (metadata, content) = strip_metadata(&chapter.content).unwrap();
        chapter.content = content;

        // add title to page
        chapter.content = format!("# {}\n{}", metadata.title, chapter.content);

        // replace page breaks with page anchors

        let parser = Parser::new(&chapter.content);

        let mut page_index = 0;
        let mapped = parser.map(|event| match event {
            Event::Text(text) if text.to_string() == ";;;;;" => {
                page_index += 1;
                Event::InlineHtml(format!("<a id=\"{page_index}\"></a>").into())
            }
            _ => event,
        });

        let mut buf = String::new();

        chapter.content = cmark(mapped, &mut buf).map(|_| buf).unwrap();

        // fix linked pagest
        let link_regex = Regex::new(
            &(r"\[(?<name>[^\]]*)\]\(\^".to_owned()
                + &self.config.id
                + r":(?<page>[^#)]*)(?<anchor>#\d+)?\)"),
        )
        .unwrap();

        let page_map: HashMap<String, std::path::PathBuf> =
            HashMap::from_iter(sections.iter().filter_map(|section| {
                match section {
                    BookItem::Chapter(Chapter {
                        name,
                        content,
                        number,
                        sub_items,
                        path: Some(path),
                        source_path,
                        parent_names,
                    }) => Some((
                        path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                            .strip_suffix(".md")
                            .unwrap()
                            .to_string(),
                        path.clone(),
                    )),
                    _ => None,
                }
            }));

        for captures in link_regex.captures_iter(&chapter.content.clone()) {
            let name = &captures["name"];
            let page = &captures["page"].split("/").last().unwrap();
            let anchor = &captures.name("anchor").map(|x| x.as_str());

            let path = diff_paths(
                page_map.get(&page.to_string()).unwrap(),
                chapter.path.clone().unwrap().parent().unwrap(),
            );

            let path = path
                .map(|x| x.to_slash().unwrap().to_string())
                .unwrap_or(String::new());

            chapter.content = chapter.content.replace(
                &captures[0],
                &format!("[{name}]({path}{})", anchor.unwrap_or("")),
            )
        }

        //chapter.content = link_regex.replace_all(&chapter.content, "[${name}](/entries/${page}.md)").to_string();
        // remap templates

        // remap owo-ui xml
    }
}
