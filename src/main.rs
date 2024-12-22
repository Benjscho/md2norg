use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing markdown files
    #[arg(short, long)]
    input: String,

    /// Output directory for converted files (optional)
    #[arg(short, long)]
    output: Option<String>,

    /// Process subdirectories recursively
    #[arg(short, long)]
    recursive: bool,

    /// Replace original files (requires confirmation unless --force is used)
    #[arg(long)]
    replace: bool,

    /// Force replacement without confirmation
    #[arg(short, long)]
    force: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_dir = Path::new(&args.input);
    let output_dir = args.output.as_ref().map(Path::new);

    if args.replace && !args.force {
        println!("Warning: This will replace the original markdown files.");
        println!("Are you sure you want to continue? (y/N)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

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

            if args.replace {
                fs::remove_file(path)?;
            }

            println!("Converted: {} -> {}", path.display(), output_path.display());
        }
    }

    Ok(())
}

fn convert_markdown_to_neorg(content: &str) -> Result<String> {
    let mut result = String::new();

    // Convert headings
    let heading_regex = Regex::new(r"^(#+)\s+(.*)$").unwrap();

    // Convert Obsidian links
    let obsidian_link_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    let content = obsidian_link_regex.replace_all(content, |caps: &regex::Captures| {
        let link_text = &caps[1];
        // Keep the original casing and spaces
        format!("{{:{}.norg:}}", link_text)
    });

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
}
