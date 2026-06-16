# mdbook-tagger

An mdbook preprocessor that generates tag index pages from frontmatter metadata.

## Features

- Scans markdown files for `tags` in YAML frontmatter
- Generates `_tags/` directory with per-tag index pages
- Works as both **mdbook preprocessor** and **standalone CLI**

## Frontmatter Format

```yaml
---
tags: [rust, wasm, tutorial]
---
```

Or multiline:

```yaml
---
tags:
  - rust
  - wasm
  - tutorial
---
```

## Usage

### Standalone CLI

```bash
# Generate tag pages for a book
mdbook-tagger generate ./my-book
```

### As mdbook Preprocessor

Add to `book.toml`:

```toml
[preprocessor.tagger]
command = "mdbook-tagger preprocess"
```

Then run `mdbook build` as usual.

## Generated Output

For each unique tag, a page is created at `_tags/<tag>.md` listing all articles with that tag.

Example `_tags/rust.md`:

```markdown
# Tag: rust

- [Introduction](../chapters/intro.md)
- [Rust Basics](../chapters/rust-basics.md)
```

A `_tags/SUMMARY.md` index page is also generated:

```markdown
# Tags

| Tag | 文章数 |
|-----|--------|
| [rust](rust.md) | 2 |
| [wasm](wasm.md) | 1 |
```

## Build

```bash
cargo build --release
```

## License

MIT
