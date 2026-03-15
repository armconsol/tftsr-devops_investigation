use std::path::Path;

pub fn export_markdown(content_md: &str, output_path: &str) -> anyhow::Result<()> {
    let path = Path::new(output_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content_md)?;
    Ok(())
}

pub fn export_pdf(content_md: &str, title: &str, output_path: &str) -> anyhow::Result<()> {
    use printpdf::*;

    let path = Path::new(output_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let (doc, page1, layer1) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    // Convert markdown to plain text lines
    let lines = markdown_to_lines(content_md);

    let margin_left = Mm(20.0);
    let margin_top = Mm(280.0);
    let line_height = Mm(5.0);
    let max_lines_per_page = 50;

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y_pos = margin_top;
    let mut line_count = 0;

    for line_info in &lines {
        if line_count >= max_lines_per_page {
            let (new_page, new_layer) =
                doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            current_layer = doc.get_page(new_page).get_layer(new_layer);
            y_pos = margin_top;
            line_count = 0;
        }

        let (font_ref, font_size) = match line_info.style {
            LineStyle::Title => (&font_bold, 16.0),
            LineStyle::Heading => (&font_bold, 12.0),
            LineStyle::Normal => (&font, 10.0),
        };

        current_layer.use_text(&line_info.text, font_size, margin_left, y_pos, font_ref);
        y_pos = y_pos - line_height;
        line_count += 1;
    }

    doc.save(&mut std::io::BufWriter::new(std::fs::File::create(path)?))?;
    Ok(())
}

#[derive(Debug)]
enum LineStyle {
    Title,
    Heading,
    Normal,
}

#[derive(Debug)]
struct LineInfo {
    text: String,
    style: LineStyle,
}

fn markdown_to_lines(md: &str) -> Vec<LineInfo> {
    let mut lines = Vec::new();

    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.push(LineInfo {
                text: String::new(),
                style: LineStyle::Normal,
            });
            continue;
        }

        if trimmed.starts_with("# ") {
            lines.push(LineInfo {
                text: trimmed.trim_start_matches('#').trim().to_string(),
                style: LineStyle::Title,
            });
        } else if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            lines.push(LineInfo {
                text: trimmed.trim_start_matches('#').trim().to_string(),
                style: LineStyle::Heading,
            });
        } else if trimmed.starts_with("---") {
            lines.push(LineInfo {
                text: "________________________________________".to_string(),
                style: LineStyle::Normal,
            });
        } else if trimmed.starts_with('|') {
            // Table row: strip pipes and clean up
            let cells: Vec<&str> = trimmed
                .split('|')
                .filter(|s| !s.trim().is_empty() && !s.chars().all(|c| c == '-' || c == '|'))
                .map(|s| s.trim())
                .collect();
            if !cells.is_empty() {
                lines.push(LineInfo {
                    text: cells.join("  |  "),
                    style: LineStyle::Normal,
                });
            }
        } else {
            // Strip basic markdown formatting
            let text = trimmed
                .replace("**", "")
                .replace("__", "")
                .replace('*', "")
                .replace('_', " ");
            // Word wrap at ~90 chars
            for wrapped in word_wrap(&text, 90) {
                lines.push(LineInfo {
                    text: wrapped,
                    style: LineStyle::Normal,
                });
            }
        }
    }

    lines
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    if text.len() <= max_width {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        result.push(current_line);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_wrap_short_text() {
        let result = word_wrap("Hello world", 90);
        assert_eq!(result, vec!["Hello world"]);
    }

    #[test]
    fn test_word_wrap_long_text() {
        let long = "word ".repeat(30);
        let result = word_wrap(long.trim(), 20);
        assert!(result.len() > 1);
        for line in &result {
            assert!(line.len() <= 24); // allow slight overflow from last word
        }
    }

    #[test]
    fn test_word_wrap_single_long_word() {
        let result = word_wrap("superlongwordthatexceedswidth", 10);
        assert_eq!(result, vec!["superlongwordthatexceedswidth"]);
    }

    #[test]
    fn test_markdown_to_lines_title() {
        let lines = markdown_to_lines("# My Title\n\nSome text");
        assert!(lines.iter().any(|l| l.text == "My Title" && matches!(l.style, LineStyle::Title)));
    }

    #[test]
    fn test_markdown_to_lines_heading() {
        let lines = markdown_to_lines("## Section\n### Subsection");
        let headings: Vec<_> = lines.iter().filter(|l| matches!(l.style, LineStyle::Heading)).collect();
        assert_eq!(headings.len(), 2);
    }

    #[test]
    fn test_markdown_to_lines_horizontal_rule() {
        let lines = markdown_to_lines("---");
        assert!(lines.iter().any(|l| l.text.contains("_____")));
    }

    #[test]
    fn test_markdown_to_lines_table_row() {
        let lines = markdown_to_lines("| Col1 | Col2 |\n|------|------|\n| A | B |");
        assert!(lines.iter().any(|l| l.text.contains("Col1") && l.text.contains("Col2")));
    }

    #[test]
    fn test_export_markdown_writes_file() {
        let dir = std::env::temp_dir().join("tftsr_test_export");
        let path = dir.join("test.md");
        let _ = std::fs::remove_file(&path);
        export_markdown("# Test\n\nContent", path.to_str().unwrap()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Test"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
