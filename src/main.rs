use anyhow::{Context, Ok, Result};
use mdbook::book::{Chapter, Link, SectionNumber, Summary, SummaryItem};
use mdbook::config::Config;
use mdbook::{BookItem, MDBook};
use pulldown_cmark::{Options, Parser, Tag, TagEnd, TextMergeStream};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::iter::Sum;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Deserialize, Debug)]
struct ColeusConfig {
    name: String,
    path: String,
    lang_path: String,
}

fn main() -> Result<()> {
    let mdbook_dir = Path::new("./build/mdbook").to_path_buf();

    // create_book_dir(&mdbook_dir)?;

    let config_toml = fs::read_to_string(Path::new("./coleus.toml"))
        .with_context(|| "Unable to read coleus.toml")?;
    let coleus_config: ColeusConfig = toml::from_str(&config_toml)?;
    let book_name = coleus_config.name.as_str();

    let book_path = Path::new(coleus_config.path.as_str()).to_path_buf();

    copy_book_pages(&mdbook_dir, &book_path, book_name)?;

    let mut book_hierarchy = create_book_hierarchy(&mdbook_dir.join("src"))?;

    let summary: Summary = create_summary(&mut book_hierarchy)?;

    let mut cfg = Config::default();
    cfg.book.title = Some("My Book".to_string());
    cfg.build.build_dir = Path::new("../book").to_path_buf();

    let md = MDBook::load_with_config_and_summary(mdbook_dir, cfg, summary)?;


    md.build()?;

    Ok(())
}

// 0. create book directory
// 1. move specific lavender book to mdbook
// 2. create the summary
// 3. pre-proccess the book
// 4. build the book

fn create_book_dir(mdbook_dir: &PathBuf) -> Result<()> {
    let src_dir = mdbook_dir.join(Path::new("src"));

    if src_dir.exists() {
        fs::remove_dir_all(&mdbook_dir).with_context(|| "Unable to delete delete mdbook dir")?;
    }
    fs::create_dir_all(&src_dir)?;

    Ok(())
}

fn copy_book_pages(mdbook_dir: &PathBuf, book_dir: &PathBuf, book_name: &str) -> Result<()> {
    let src_dir = mdbook_dir.join(Path::new("src"));

    // let categories_dir = src_dir.join(Path::new("categories"));
    // fs::create_dir(&src_dir)?;

    let categories_dir = book_dir.join(Path::new(&format!("categories/{book_name}")));
    dircpy::copy_dir(categories_dir, src_dir.join("categories"))?;

    let entries_dir = book_dir.join(Path::new(&format!("entries/{book_name}")));
    dircpy::copy_dir(entries_dir, src_dir.join("entries"))?;

    Ok(())
}

struct BookHierarchy {
    pub categorized: HashMap<String, BookCategoryEntry>,
    pub uncategorized: Vec<(Link, u32)>,
}

struct BookCategoryEntry {
    pub title: String,
    pub category_link: Link,
    pub links: Vec<(Link, u32)>,
}

impl BookHierarchy {
    pub fn new() -> BookHierarchy {
        return BookHierarchy {
            categorized: HashMap::new(),
            uncategorized: Vec::new(),
        };
    }
}

fn create_book_hierarchy(src_dir: &PathBuf) -> Result<BookHierarchy> {
    let mut hierarchy = BookHierarchy::new();

    let categories_dir = src_dir.join(Path::new(&format!("categories")));
    for file in fs::read_dir(&categories_dir)? {
        let file = file?;
        let file_name = file.file_name();

        let file_path = categories_dir.join(&file_name);
        let link_path = Path::new("categories").join(&file_name);

        let metadata: CategoryMetadata = get_page_metadata(&fs::read_to_string(&file_path)?)?;

        let category = file_name
            .into_string()
            .expect("file name could not be read")
            .strip_suffix(".md")
            .unwrap()
            .to_string();

        hierarchy.categorized.insert(
            category,
            BookCategoryEntry {
                title: metadata.title.clone(),
                category_link: Link::new(metadata.title, &link_path),
                links: vec![],
            },
        );
    }

    add_entries(
        src_dir,
        &src_dir.join(Path::new(&format!("entries"))),
        &mut hierarchy,
    )?;

    Ok(hierarchy)
}

fn create_summary(hierarchy: &mut BookHierarchy) -> Result<Summary> {
    let mut summary = Summary::default();

    let mut sorted_links = hierarchy.uncategorized.clone();
    sorted_links.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (link, _) in sorted_links {
        summary
            .prefix_chapters
            .push(SummaryItem::Link(link.clone()))
    }

    for (category_index, BookCategoryEntry { title, category_link, links }) in hierarchy.categorized.values().enumerate() {
        // add category title
        summary
            .numbered_chapters
            .push(SummaryItem::PartTitle(title.clone()));
        
        // add main category page
        let mut category_link = category_link.clone();
        category_link.number = Some(SectionNumber(vec![(category_index + 1).try_into().unwrap()]));

        summary
            .numbered_chapters
            .push(SummaryItem::Link(category_link));

        // add other pages in category
        let mut sorted_links = links.clone();
        sorted_links.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (chapter_index, (mut link, _)) in sorted_links.into_iter().enumerate() {
            link.number = Some(SectionNumber(vec![(category_index + 1).try_into().unwrap(), (chapter_index + 1).try_into().unwrap()]));

            summary
                .numbered_chapters
                .push(SummaryItem::Link(link.clone()))
        }
    }

    Ok(summary)
}

fn add_entries(
    src_dir: &PathBuf,
    entries_dir: &PathBuf,
    hierarchy: &mut BookHierarchy,
) -> Result<(), anyhow::Error> {
    for dir_entry in fs::read_dir(entries_dir).unwrap() {
        let dir_entry = dir_entry.unwrap();

        if dir_entry.metadata().unwrap().is_dir() {
            add_entries(src_dir, &dir_entry.path(), hierarchy).unwrap()
        } else if dir_entry
            .file_name()
            .into_string()
            .expect("could not read file name")
            .ends_with(".md")
        {
            let file = dir_entry;

            let file_path = file.path();
            let link_path = file_path.strip_prefix(src_dir).unwrap();

            let metadata: EntryMetadata =
                get_page_metadata(&fs::read_to_string(&file_path).unwrap()).unwrap();

            if metadata.category.is_some() {
                let entries = hierarchy
                    .categorized
                    .get_mut(metadata.category.unwrap().split(":").collect::<Vec<_>>()[1]);

                if entries.is_some() {
                    entries.unwrap().links.push((
                        Link::new(metadata.title, link_path),
                        metadata.ordinal.unwrap_or(u32::max_value()),
                    ))
                }
            } else {
                hierarchy
                    .uncategorized
                    .push((Link::new(metadata.title, link_path), metadata.ordinal.unwrap_or(u32::max_value())))
            }
        }
    }

    Ok(())
}

fn get_page_metadata<'a, T: Deserialize<'a>>(page: &'a str) -> Result<T> {
    //regex to get the json in a code block
    let regex = Regex::new(r"``` *json\n?((\n|.)*)```").unwrap();

    let json_str = regex
        .captures_iter(page)
        .next()
        .expect("no match found")
        .get(1)
        .expect("no match found")
        .as_str();

    return serde_json::from_str(json_str).with_context(|| "could not decode json");
}

#[derive(Deserialize)]
struct CategoryMetadata {
    title: String,
    icon: Option<String>,
    ordinal: Option<u32>,
    parent: Option<String>,
}

#[derive(Deserialize)]
struct EntryMetadata {
    title: String,
    icon: Option<String>,
    ordinal: Option<u32>,
    category: Option<String>,
}
