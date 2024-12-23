use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use regex::Regex;
use walkdir::WalkDir;

/// md2norg - a markdown to neorg file converter.
///
/// This tool converts notes kept in a markdown format to neorg (.norg). This is
/// primarily handy if you have a bunch of notes in Obsidian that you want to
/// import into a neorg workspace.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing markdown files
    #[arg(short, long)]
    input: String,

    /// Output directory for converted files (optional), otherwise existing
    /// directory is used.
    #[arg(short, long)]
    output: Option<String>,

    /// Process subdirectories recursively
    #[arg(short, long)]
    recursive: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_dir = Path::new(&args.input);
    let output_dir = args.output.as_ref().map(Path::new);

    let walker = if args.recursive {
        WalkDir::new(input_dir)
    } else {
        WalkDir::new(input_dir).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            let output_path = if let Some(out_dir) = output_dir {
                out_dir
                    .join(path.strip_prefix(input_dir)?)
                    .with_extension("norg")
            } else {
                path.with_extension("norg")
            };

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let content = fs::read_to_string(path)?;
            let converted = convert_markdown_to_neorg(&content)?;

            fs::write(&output_path, converted)?;

            println!("Converted: {} -> {}", path.display(), output_path.display());
        }
    }

    Ok(())
}

fn convert_markdown_to_neorg(content: &str) -> Result<String> {
    let mut result = String::new();

    // Convert headings
    let heading_regex = Regex::new(r"^(#+)\s+(.*)$").unwrap();

    let link_conversions = [
        // Image link with title (must come before basic image link)
        (r#"!\[([^\]]*)\]\(([^)]+)\s+"([^"]+)"\)"#, "{image:$2}[$1]"),
        // Basic image link
        (r"!\[([^\]]*)\]\(([^)]+)\)", "{image:$2}[$1]"),
        // Reference-style image link
        (r"!\[([^\]]*)\]\[([^\]]*)\]", "{image:$2}[$1]"),
        // Basic Markdown link
        (r"\[([^\]]+)\]\(([^)]+)\)", "{$2}[$1]"),
        // Reference-style link
        (r"\[([^\]]+)\]\[([^\]]*)\]", "{$2}[$1]"),
        // Obsidian links
        (r"\[\[([^\]]+)\]\]", "{:$1.norg:}"),
        // Reference-style link definition
        (
            r#"(?m)^\[([^\]]+)\]:\s*(\S+)(?:\s+"([^"]+)")?"#,
            "@$1 $2 $3",
        ),
        // Automatic links
        (r"<(https?://[^>]+)>", "{$1}[$1]"),
    ];

    let mut content = content.to_string();
    for (pattern, replacement) in link_conversions.iter() {
        let re = Regex::new(pattern).unwrap();
        content = re.replace_all(&content, *replacement).to_string();
    }

    for line in content.lines() {
        if let Some(caps) = heading_regex.captures(line) {
            let level = caps[1].len();
            let text = &caps[2];
            result.push_str(&format!("{} {}\n", "*".repeat(level), text));
        } else if let Some(caps) = Regex::new(r"^(\s*)- \[ \] (.*)$").unwrap().captures(line) {
            let indent = &caps[1];
            let text = &caps[2];
            result.push_str(&format!("{}-- ( ) {}\n", indent, text));
        } else if let Some(caps) = Regex::new(r"^(\s*)- \[x\] (.*)$").unwrap().captures(line) {
            let indent = &caps[1];
            let text = &caps[2];
            result.push_str(&format!("{}-- (x) {}\n", indent, text));
        } else if let Some(caps) = Regex::new(r"^(\s*)[-*+]\s+(.*)$").unwrap().captures(line) {
            let indent = &caps[1];
            let text = &caps[2];
            result.push_str(&format!("{}-- {}\n", indent, text));
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Convert code blocks
    let code_block_regex = Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap();
    let result = code_block_regex.replace_all(&result, |caps: &regex::Captures| {
        let language = &caps[1];
        let code = &caps[2].trim_end(); // Trim trailing whitespace
        format!("@code {}\n{}\n@end", language, code)
    });

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_headings() -> Result<()> {
        let markdown = "# Heading 1\n## Heading 2\n### Heading 3";
        let expected = "* Heading 1\n** Heading 2\n*** Heading 3\n";
        assert_eq!(convert_markdown_to_neorg(markdown)?, expected);
        Ok(())
    }

    #[test]
    fn test_convert_code_blocks() -> Result<()> {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let expected = "@code rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n@end\n";
        assert_eq!(convert_markdown_to_neorg(markdown)?, expected);
        Ok(())
    }

    #[test]
    fn test_convert_lists() -> Result<()> {
        let markdown = "- Item 1\n- Item 2\n  - Subitem 2.1\n- Item 3";
        let expected = "-- Item 1\n-- Item 2\n  -- Subitem 2.1\n-- Item 3\n";
        assert_eq!(convert_markdown_to_neorg(markdown)?, expected);
        Ok(())
    }

    #[test]
    fn test_convert_todos() -> Result<()> {
        let markdown = "- [ ] Todo item\n- [x] Completed item";
        let expected = "-- ( ) Todo item\n-- (x) Completed item\n";
        assert_eq!(convert_markdown_to_neorg(markdown)?, expected);
        Ok(())
    }

    #[test]
    fn test_convert_mixed_content() -> Result<()> {
        let markdown = "# Main Heading\n\n## Subheading\n\n- List item 1\n- [ ] Todo item\n\n```python\nprint(\"Hello, world!\")\n```";
        let expected = "* Main Heading\n\n** Subheading\n\n-- List item 1\n-- ( ) Todo item\n\n@code python\nprint(\"Hello, world!\")\n@end\n";
        assert_eq!(convert_markdown_to_neorg(markdown)?, expected);
        Ok(())
    }

    #[test]
    fn test_preserve_non_converted_content() -> Result<()> {
        let markdown = "This is regular text.\n\nIt should be preserved as-is.";
        let expected = "This is regular text.\n\nIt should be preserved as-is.\n";
        assert_eq!(convert_markdown_to_neorg(markdown)?, expected);
        Ok(())
    }

    #[test]
    fn test_convert_obsidian_links() -> Result<()> {
        let markdown = "Check out [[My Page]] and [[Another Page With Spaces]]";
        let expected = "Check out {:My Page.norg:} and {:Another Page With Spaces.norg:}\n";
        assert_eq!(convert_markdown_to_neorg(markdown)?, expected);
        Ok(())
    }

    #[test]
    fn test_convert_markdown_links() -> Result<()> {
        let input = r#"
[Basic link](https://example.com)
[Reference link][ref]
[Implicit reference link][]
<https://example.com>
![Image](image.jpg)
![Image with title](image.jpg "Title")
![Reference image][img-ref]

[ref]: https://example.com "Reference Title"
[img-ref]: image.jpg "Image Reference Title"
"#;

        let expected_output = r#"
{https://example.com}[Basic link]
{ref}[Reference link]
{}[Implicit reference link]
{https://example.com}[https://example.com]
{image:image.jpg}[Image]
{image:image.jpg}[Image with title]
{image:img-ref}[Reference image]

@ref https://example.com Reference Title
@img-ref image.jpg Image Reference Title
"#;

        let actual = convert_markdown_to_neorg(input)?;
        println!("{}", &actual);
        assert_eq!(actual, expected_output);
        Ok(())
    }
}
