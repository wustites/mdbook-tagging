use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;
use walkdir::WalkDir;

use crate::frontmatter;

/// A tag entry: (article title, relative path)
type TagEntry = (String, String);

/// Build a map of tag -> list of articles from the source directory.
fn collect_tags(source_dir: &Path) -> Result<BTreeMap<String, Vec<TagEntry>>> {
    let mut tag_map: BTreeMap<String, Vec<TagEntry>> = BTreeMap::new();

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "md") {
            continue;
        }

        // Skip files inside the _tags directory
        if path.components().any(|c| c.as_os_str() == "_tags") {
            continue;
        }

        let Some(fm) = frontmatter::parse_file(path) else {
            continue;
        };

        let title = extract_title(path).unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        let relative = path
            .strip_prefix(source_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        for tag in &fm.tags {
            tag_map
                .entry(tag.clone())
                .or_default()
                .push((title.clone(), relative.clone()));
        }
    }

    Ok(tag_map)
}

/// Extract the first H1 heading as the article title.
fn extract_title(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(title) = line.strip_prefix("# ") {
            return Some(title.to_string());
        }
    }
    None
}

/// Generate markdown content for a tag index page.
fn generate_tag_page(tag: &str, entries: &[TagEntry]) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Tag: {}\n\n", tag));
    for (title, path) in entries {
        md.push_str(&format!("- [{}](../{})\n", title, path));
    }
    md
}

/// Run as mdbook preprocessor: read JSON from stdin, inject tag pages, write to stdout.
pub fn run_preprocess() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut ctx: Value =
        serde_json::from_str(&input).context("Failed to parse mdbook preprocessor input")?;

    // Extract source directory from context
    let root = ctx
        .get("root")
        .and_then(|v| v.as_str())
        .unwrap_or(".");
    let src_dir = Path::new(root).join("src");

    let tag_map = collect_tags(&src_dir)?;

    if tag_map.is_empty() {
        // No tags found, pass through unchanged
        println!("{}", serde_json::to_string(&ctx)?);
        return Ok(());
    }

    // Generate tag index sections
    let mut tag_sections = Vec::new();
    let mut tags_index_content = String::from("# Tags\n\n");
    tags_index_content.push_str("| Tag | 文章数 |\n");
    tags_index_content.push_str("|-----|--------|\n");

    let mut sorted_tags: Vec<_> = tag_map.keys().collect();
    sorted_tags.sort();

    for tag in &sorted_tags {
        let entries = &tag_map[*tag];
        tags_index_content.push_str(&format!("| [{}]({}.md) | {} |\n", tag, tag, entries.len()));

        let page_content = generate_tag_page(tag, entries);
        let tag_section = serde_json::json!({
            "Chapter": {
                "name": format!("Tag: {}", tag),
                "content": page_content,
                "path": format!("_tags/{}.md", tag),
                "parent_names": ["_tags"]
            }
        });
        tag_sections.push(tag_section);
    }

    // Add tags index page
    let tags_index_section = serde_json::json!({
        "Chapter": {
            "name": "Tags",
            "content": tags_index_content,
            "path": "_tags/SUMMARY.md",
            "parent_names": []
        }
    });

    // Insert into sections
    if let Some(sections) = ctx.get_mut("sections").and_then(|s| s.as_array_mut()) {
        sections.push(tags_index_section);
        sections.extend(tag_sections);
    }

    println!("{}", serde_json::to_string(&ctx)?);
    Ok(())
}

/// Generate tag index pages directly to the filesystem.
pub fn generate_tags(book_dir: &str) -> Result<()> {
    let book_path = Path::new(book_dir);
    let src_dir = book_path.join("src");
    let tags_dir = src_dir.join("_tags");

    let tag_map = collect_tags(&src_dir)?;

    if tag_map.is_empty() {
        println!("No tags found in any articles.");
        return Ok(());
    }

    // Create _tags directory
    std::fs::create_dir_all(&tags_dir)?;

    // Generate tags index
    let mut index_content = String::from("# Tags\n\n");
    index_content.push_str("| Tag | 文章数 |\n");
    index_content.push_str("|-----|--------|\n");

    let mut sorted_tags: Vec<_> = tag_map.keys().collect();
    sorted_tags.sort();

    for tag in &sorted_tags {
        let entries = &tag_map[*tag];
        index_content.push_str(&format!("| [{}]({}.md) | {} |\n", tag, tag, entries.len()));

        // Write individual tag page
        let page_content = generate_tag_page(tag, entries);
        let tag_file = tags_dir.join(format!("{}.md", tag));
        std::fs::write(&tag_file, page_content)?;
        println!("  Generated: {}", tag_file.display());
    }

    // Write index
    let index_file = tags_dir.join("SUMMARY.md");
    std::fs::write(&index_file, index_content)?;
    println!("  Generated: {}", index_file.display());

    println!("\nDone! {} tags generated.", sorted_tags.len());
    Ok(())
}
