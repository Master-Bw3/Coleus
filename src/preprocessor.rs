use std::collections::HashMap;

use anyhow::{Context, Error, Result};
use mdbook::{
    book::{Book, Chapter},
    preprocess::{self, Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::Regex;
use serde::Deserialize;

use crate::config::ColeusConfig;

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
        // get json metadata
        let json_regex = Regex::new(r"``` *json\n?((\n|.)*)```").unwrap();

        let json_str = json_regex
            .captures_iter(&chapter.content)
            .next()
            .expect("no match found")
            .get(1)
            .expect("no match found")
            .as_str();

        let metadata: PageMetadata = serde_json::from_str(json_str).unwrap();

        // remove json metadata

        chapter.content = json_regex.replace_all(&chapter.content, "").to_string();

        // add title to page

        chapter.content = format!("# {}\n{}", metadata.title, chapter.content);

        // replace page breaks with page anchors
        let mut page_index = 0;

        chapter.content = chapter
            .content
            .split(";;;;;")
            .map(|slice| {
                page_index += 1;
                format!("<a id=\"{page_index}\"></a>\n{slice}")
            })
            .collect::<Vec<String>>()
            .join("");

        // fix linked pages
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

            let path = page_map.get(&page.to_string());
            let path = path
                .map(|x| x.display().to_string())
                .unwrap_or(String::new());

            println!("{:?}", anchor);

            chapter.content = chapter.content.replace(
                &captures[0],
                &format!("[{name}](/{path}{})", anchor.unwrap_or("")),
            )
        }

        //chapter.content = link_regex.replace_all(&chapter.content, "[${name}](/entries/${page}.md)").to_string();
        // remap templates

        // remap owo-ui xml
    }
}

#[derive(Deserialize)]
struct PageMetadata {
    title: String,
}
