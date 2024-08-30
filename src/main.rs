use anyhow::{Context, Ok, Result};
use mdbook::book::{Chapter, Link, Summary, SummaryItem};
use mdbook::config::Config;
use mdbook::{BookItem, MDBook};
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
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


    create_book_dir(&mdbook_dir)?;

    let config_toml = fs::read_to_string(Path::new("./coleus.toml")).with_context(|| "Unable to read coleus.toml")?;
    let coleus_config: ColeusConfig = toml::from_str(&config_toml)?;
    let book_name = coleus_config.name.as_str();

    let book_path = Path::new(coleus_config.path.as_str()).to_path_buf();


    copy_book_pages(&mdbook_dir, &book_path, book_name)?;

    let summary: Summary = create_summary(& mdbook_dir.join(Path::new("src")), book_name)?;

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


fn create_summary(src_dir: &PathBuf, book_name: &str) -> Result<Summary> {
    let mut summary = Summary::default();

    let categories_dir = src_dir.join(Path::new(&format!("categories")));
    for file in fs::read_dir(&categories_dir)? {
        let file = file?;
        let file_name = file.file_name();

        let file_path = categories_dir.join(&file_name);
        let link_path = Path::new("categories").join(&file_name);

        summary.numbered_chapters.push(SummaryItem::Link(Link::new(file_path.to_str().unwrap(), &link_path)))
    };

    let entries_dir = src_dir.join(Path::new(&format!("entries")));

    Ok(summary)
}

// fn create_book_dir() -> Result<()> {
//     let current_dir = env::current_dir().expect("could not get working directory");

//     let mdbook_dir = current_dir.join(Path::new("build/mdbook"));
    
//     let src_dir = mdbook_dir.join(Path::new("src"));

//     if src_dir.exists() {
//         fs::remove_dir_all(&mdbook_dir).with_context(|| "Unable to delete delete mdbook dir")?;
//     }

//     fs::create_dir_all(&src_dir)?;

//     let mut cfg = Config::default();
//     cfg.book.title = Some("My Book".to_string());
//     cfg.build.build_dir = Path::new("../book").to_path_buf();

//     create_test(&src_dir)?;

//     let mut summary = Summary::default();
//     summary.numbered_chapters.push(SummaryItem::Link(Link::new("test", "chapter_2.md")));



//     let md = MDBook::load_with_config_and_summary(mdbook_dir, cfg, summary)?;

//     md.build().with_context(|| "Building failed")?;
//     Ok(())
// }

// fn create_summary_md(src_dir: &PathBuf) -> Result<()> {
//     let summary_path = src_dir.join(Path::new("SUMMARY.md"));

//     if summary_path.exists() {
//         fs::remove_file(&summary_path).with_context(|| "Unable to delete delete summary file")?;
//     }

//     let mut f = File::create(&summary_path).with_context(|| "Unable to create SUMMARY.md")?;
//     writeln!(f, "# Summary")?;
//     writeln!(f)?;
//     writeln!(f, "- [Chapter 2](./chapter_2.md)")?;

//     return Ok(());
// }

// fn create_test(src_dir: &PathBuf) -> Result<()> {
//     let test_path = src_dir.join(Path::new("chapter_2.md"));

//     if test_path.exists() {
//         fs::remove_file(&test_path).with_context(|| "Unable to delete delete summary file")?;
//     }

//     let mut f = File::create(&test_path).with_context(|| "Unable to create SUMMARY.md")?;
//     writeln!(f, "# this is a test")?;

//     return Ok(());
// }

