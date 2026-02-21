use crate::ast::{self, Block, Inline, inline_text};

#[derive(Debug)]
enum DiffOp<T> {
    Equal(T),
    Remove(T),
    Add(T),
}

/// LCS-based sequence diff.
fn diff_sequences<'a, T: PartialEq>(old: &'a [T], new: &'a [T]) -> Vec<DiffOp<&'a T>> {
    let m = old.len();
    let n = new.len();

    // Build LCS table
    let mut table = vec![vec![0u32; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = table[i - 1][j].max(table[i][j - 1]);
            }
        }
    }

    // Backtrack
    let mut ops = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            ops.push(DiffOp::Equal(&old[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || table[i][j - 1] >= table[i - 1][j]) {
            ops.push(DiffOp::Add(&new[j - 1]));
            j -= 1;
        } else {
            ops.push(DiffOp::Remove(&old[i - 1]));
            i -= 1;
        }
    }
    ops.reverse();
    ops
}

/// Diff two markdown strings and produce markdown with change markers.
/// Removed content wrapped in `~~strikethrough~~`, added in `**bold**`.
pub fn diff_markdown(old: &str, new: &str) -> String {
    let old_blocks = ast::parse(old);
    let new_blocks = ast::parse(new);
    let diff_ops = diff_sequences(&old_blocks, &new_blocks);

    let mut result_blocks: Vec<Block> = Vec::new();
    for op in diff_ops {
        match op {
            DiffOp::Equal(block) => {
                result_blocks.push(block.clone());
            }
            DiffOp::Remove(block) => {
                result_blocks.push(wrap_block_removed(block));
            }
            DiffOp::Add(block) => {
                result_blocks.push(wrap_block_added(block));
            }
        }
    }

    ast::render_to_markdown(&result_blocks)
}

/// Check if two blocks are the same variant (suitable for inline diffing).
fn same_variant(a: &Block, b: &Block) -> bool {
    matches!(
        (a, b),
        (Block::Paragraph(_), Block::Paragraph(_))
            | (Block::Heading { .. }, Block::Heading { .. })
            | (Block::BlockQuote(_), Block::BlockQuote(_))
            | (Block::CodeBlock { .. }, Block::CodeBlock { .. })
    )
}

/// Diff two markdown strings with inline-level diffing for changed blocks.
/// This is the smarter version that tries to match up structurally similar blocks.
pub fn diff_markdown_inline(old: &str, new: &str) -> String {
    let old_blocks = ast::parse(old);
    let new_blocks = ast::parse(new);
    let diff_ops = diff_sequences(&old_blocks, &new_blocks);

    // Collapse consecutive Remove/Add pairs of the same variant into inline diffs
    let mut result_blocks: Vec<Block> = Vec::new();
    let mut pending_removes: Vec<&Block> = Vec::new();

    for op in &diff_ops {
        match op {
            DiffOp::Equal(block) => {
                flush_removes(&mut result_blocks, &mut pending_removes);
                result_blocks.push((*block).clone());
            }
            DiffOp::Remove(block) => {
                pending_removes.push(block);
            }
            DiffOp::Add(block) => {
                // Try to pair with a pending remove of the same variant
                if let Some(idx) = pending_removes.iter().position(|r| same_variant(r, block)) {
                    let removed = pending_removes.remove(idx);
                    result_blocks.push(diff_block_inline(removed, block));
                } else {
                    flush_removes(&mut result_blocks, &mut pending_removes);
                    result_blocks.push(wrap_block_added(block));
                }
            }
        }
    }
    flush_removes(&mut result_blocks, &mut pending_removes);

    ast::render_to_markdown(&result_blocks)
}

fn flush_removes(result: &mut Vec<Block>, pending: &mut Vec<&Block>) {
    for block in pending.drain(..) {
        result.push(wrap_block_removed(block));
    }
}

fn diff_block_inline(old: &Block, new: &Block) -> Block {
    match (old, new) {
        (Block::Paragraph(old_inlines), Block::Paragraph(new_inlines)) => {
            Block::Paragraph(diff_inlines(old_inlines, new_inlines))
        }
        (
            Block::Heading {
                level,
                content: old_content,
            },
            Block::Heading {
                content: new_content,
                ..
            },
        ) => Block::Heading {
            level: *level,
            content: diff_inlines(old_content, new_content),
        },
        (Block::BlockQuote(old_inner), Block::BlockQuote(new_inner)) => {
            let inner_old_md = ast::render_to_markdown(old_inner);
            let inner_new_md = ast::render_to_markdown(new_inner);
            let diffed = diff_markdown_inline(&inner_old_md, &inner_new_md);
            Block::BlockQuote(ast::parse(&diffed))
        }
        (
            Block::CodeBlock {
                language: old_lang,
                code: old_code,
            },
            Block::CodeBlock {
                language: new_lang,
                code: new_code,
            },
        ) => {
            if old_code == new_code && old_lang == new_lang {
                new.clone()
            } else {
                // Show as word-level diff of the code content.
                // We can't embed literal ``` in inline text (it would be
                // re-parsed as a code fence), so diff the code words directly.
                let old_words: Vec<&str> = old_code.split_whitespace().collect();
                let new_words: Vec<&str> = new_code.split_whitespace().collect();
                let ops = diff_sequences(&old_words, &new_words);
                let mut inlines = Vec::new();
                let mut removed_run: Vec<String> = Vec::new();
                let mut added_run: Vec<String> = Vec::new();

                let flush = |result: &mut Vec<Inline>,
                             removed: &mut Vec<String>,
                             added: &mut Vec<String>| {
                    if !removed.is_empty() {
                        if !result.is_empty() {
                            result.push(Inline::Text(" ".to_string()));
                        }
                        result.push(Inline::Strikethrough(vec![Inline::Code(
                            std::mem::take(removed).join(" "),
                        )]));
                    }
                    if !added.is_empty() {
                        if !result.is_empty() {
                            result.push(Inline::Text(" ".to_string()));
                        }
                        result.push(Inline::Strong(vec![Inline::Code(
                            std::mem::take(added).join(" "),
                        )]));
                    }
                };

                for op in ops {
                    match op {
                        DiffOp::Equal(word) => {
                            flush(&mut inlines, &mut removed_run, &mut added_run);
                            if !inlines.is_empty() {
                                inlines.push(Inline::Text(" ".to_string()));
                            }
                            inlines.push(Inline::Code((*word).to_string()));
                        }
                        DiffOp::Remove(word) => {
                            removed_run.push((*word).to_string());
                        }
                        DiffOp::Add(word) => {
                            added_run.push((*word).to_string());
                        }
                    }
                }
                flush(&mut inlines, &mut removed_run, &mut added_run);
                Block::Paragraph(inlines)
            }
        }
        _ => {
            // Fallback: show removed then added
            wrap_block_removed(old)
        }
    }
}

fn diff_inlines(old: &[Inline], new: &[Inline]) -> Vec<Inline> {
    // Flatten to words for word-level diffing
    let old_text = inline_text(old);
    let new_text = inline_text(new);

    let old_words: Vec<&str> = old_text.split_whitespace().collect();
    let new_words: Vec<&str> = new_text.split_whitespace().collect();

    let ops = diff_sequences(&old_words, &new_words);

    let mut result = Vec::new();
    let mut removed_run = Vec::new();
    let mut added_run = Vec::new();

    let flush_runs =
        |result: &mut Vec<Inline>, removed: &mut Vec<String>, added: &mut Vec<String>| {
            if !removed.is_empty() {
                if !result.is_empty() {
                    result.push(Inline::Text(" ".to_string()));
                }
                result.push(Inline::Strikethrough(vec![Inline::Text(
                    std::mem::take(removed).join(" "),
                )]));
            }
            if !added.is_empty() {
                if !result.is_empty() {
                    result.push(Inline::Text(" ".to_string()));
                }
                result.push(Inline::Strong(vec![Inline::Text(
                    std::mem::take(added).join(" "),
                )]));
            }
        };

    for op in ops {
        match op {
            DiffOp::Equal(word) => {
                flush_runs(&mut result, &mut removed_run, &mut added_run);
                if !result.is_empty() {
                    result.push(Inline::Text(" ".to_string()));
                }
                result.push(Inline::Text((*word).to_string()));
            }
            DiffOp::Remove(word) => {
                removed_run.push((*word).to_string());
            }
            DiffOp::Add(word) => {
                added_run.push((*word).to_string());
            }
        }
    }
    flush_runs(&mut result, &mut removed_run, &mut added_run);

    result
}

fn wrap_block_removed(block: &Block) -> Block {
    match block {
        Block::Paragraph(inlines) => Block::Paragraph(vec![Inline::Strikethrough(inlines.clone())]),
        Block::Heading { level, content } => Block::Heading {
            level: *level,
            content: vec![Inline::Strikethrough(content.clone())],
        },
        Block::CodeBlock { code, .. } => {
            Block::Paragraph(vec![Inline::Strikethrough(vec![Inline::Code(
                code.trim_end_matches('\n').to_string(),
            )])])
        }
        Block::BlockQuote(inner) => {
            let wrapped: Vec<Block> = inner.iter().map(wrap_block_removed).collect();
            Block::BlockQuote(wrapped)
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            let wrapped_items: Vec<Vec<Block>> = items
                .iter()
                .map(|item| item.iter().map(wrap_block_removed).collect())
                .collect();
            Block::List {
                ordered: *ordered,
                start: *start,
                items: wrapped_items,
            }
        }
        Block::ThematicBreak => Block::Paragraph(vec![Inline::Strikethrough(vec![Inline::Text(
            "---".to_string(),
        )])]),
        Block::Table {
            alignments,
            header,
            rows,
        } => {
            let wrap_cells = |cells: &[Vec<Inline>]| -> Vec<Vec<Inline>> {
                cells
                    .iter()
                    .map(|cell| vec![Inline::Strikethrough(cell.clone())])
                    .collect()
            };
            Block::Table {
                alignments: alignments.clone(),
                header: wrap_cells(header),
                rows: rows.iter().map(|row| wrap_cells(row)).collect(),
            }
        }
        Block::HtmlBlock(html) => {
            Block::Paragraph(vec![Inline::Strikethrough(vec![Inline::Text(
                html.clone(),
            )])])
        }
    }
}

fn wrap_block_added(block: &Block) -> Block {
    match block {
        Block::Paragraph(inlines) => Block::Paragraph(vec![Inline::Strong(inlines.clone())]),
        Block::Heading { level, content } => Block::Heading {
            level: *level,
            content: vec![Inline::Strong(content.clone())],
        },
        Block::CodeBlock { code, .. } => {
            Block::Paragraph(vec![Inline::Strong(vec![Inline::Code(
                code.trim_end_matches('\n').to_string(),
            )])])
        }
        Block::BlockQuote(inner) => {
            let wrapped: Vec<Block> = inner.iter().map(wrap_block_added).collect();
            Block::BlockQuote(wrapped)
        }
        Block::List {
            ordered,
            start,
            items,
        } => {
            let wrapped_items: Vec<Vec<Block>> = items
                .iter()
                .map(|item| item.iter().map(wrap_block_added).collect())
                .collect();
            Block::List {
                ordered: *ordered,
                start: *start,
                items: wrapped_items,
            }
        }
        Block::ThematicBreak => {
            Block::Paragraph(vec![Inline::Strong(vec![Inline::Text("---".to_string())])])
        }
        Block::Table {
            alignments,
            header,
            rows,
        } => {
            let wrap_cells = |cells: &[Vec<Inline>]| -> Vec<Vec<Inline>> {
                cells
                    .iter()
                    .map(|cell| vec![Inline::Strong(cell.clone())])
                    .collect()
            };
            Block::Table {
                alignments: alignments.clone(),
                header: wrap_cells(header),
                rows: rows.iter().map(|row| wrap_cells(row)).collect(),
            }
        }
        Block::HtmlBlock(html) => {
            Block::Paragraph(vec![Inline::Strong(vec![Inline::Text(html.clone())])])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Realistic tracey scenarios — these mirror actual requirement .raw text
    // that gets diffed for hover popups.
    // =========================================================================

    #[test]
    fn tracey_paragraph_req_word_change() {
        // Paragraph requirement: .raw is text after the r[...] marker
        let v1 = "All payloads MUST use Postcard wire format.\n";
        let v2 = "All payloads MUST use MessagePack wire format.\n";
        let result = diff_markdown_inline(v1, v2);
        eprintln!("--- paragraph req word change ---\n{result}---");
        assert!(result.contains("~~Postcard~~"));
        assert!(result.contains("**MessagePack**"));
        assert!(result.contains("MUST"));
        assert!(result.contains("wire"));
    }

    #[test]
    fn tracey_blockquote_req_text_change() {
        // Blockquote requirement: .raw includes `> ` prefixes, marker already stripped
        let v1 = "\
> The server MUST validate all incoming session tokens
> before processing any request.
";
        let v2 = "\
> The server MUST validate all incoming session tokens
> and verify their expiry before processing any request.
";
        let result = diff_markdown_inline(v1, v2);
        eprintln!("--- blockquote req text change ---\n{result}---");
        // Blockquote structure preserved
        assert!(
            result.contains("> "),
            "blockquote prefix preserved: {result}"
        );
        // Word-level diff inside the blockquote — added phrase grouped together
        assert!(
            result.contains("**and verify their expiry**"),
            "added words marked: {result}"
        );
    }

    #[test]
    fn tracey_blockquote_req_with_code_block() {
        // Multi-block blockquote: text paragraph + code example
        let v1 = "\
> Responses MUST conform to the following schema:
>
> ```json
> {\"status\": \"ok\", \"data\": []}
> ```
";
        let v2 = "\
> Responses MUST conform to the following schema:
>
> ```json
> {\"status\": \"ok\", \"data\": [], \"meta\": {}}
> ```
";
        let result = diff_markdown_inline(v1, v2);
        eprintln!("--- blockquote with code block change ---\n{result}---");
        // The text paragraph is unchanged
        assert!(result.contains("Responses MUST conform"));
        // Code block change should show old/new
        assert!(result.contains("~~"), "old code struck: {result}");
        assert!(result.contains("**"), "new code bolded: {result}");
    }

    #[test]
    fn tracey_added_paragraph_to_req() {
        // Requirement grows: v2 adds a clarifying paragraph
        let v1 = "Clients MUST retry failed requests with exponential backoff.\n";
        let v2 = "\
Clients MUST retry failed requests with exponential backoff.

The initial delay SHOULD be 100ms, doubling on each retry up to 10s.
";
        let result = diff_markdown_inline(v1, v2);
        eprintln!("--- added paragraph ---\n{result}---");
        assert!(result.contains("exponential backoff."));
        assert!(
            result.contains("**"),
            "added paragraph should be bold: {result}"
        );
        assert!(
            result.contains("100ms") || result.contains("SHOULD"),
            "new content present: {result}"
        );
    }

    #[test]
    fn tracey_removed_paragraph_from_req() {
        // Requirement shrinks: v2 removes the example
        let v1 = "\
Connections MUST use TLS 1.3 or higher.

Legacy TLS 1.2 connections MAY be accepted during the migration period.
";
        let v2 = "Connections MUST use TLS 1.3 or higher.\n";
        let result = diff_markdown_inline(v1, v2);
        eprintln!("--- removed paragraph ---\n{result}---");
        assert!(result.contains("TLS 1.3"));
        assert!(
            result.contains("~~"),
            "removed paragraph should be struck: {result}"
        );
    }

    #[test]
    fn tracey_unchanged_req() {
        let md = "\
> The implementation MUST handle partial reads by buffering
> incomplete frames until a full message boundary is received.
";
        let result = diff_markdown_inline(md, md);
        eprintln!("--- unchanged ---\n{result}---");
        assert!(!result.contains("~~"), "no strikethrough: {result}");
        assert!(!result.contains("**"), "no bold: {result}");
        assert!(result.contains("MUST handle"));
    }

    // =========================================================================
    // Structural preservation tests — the whole point vs raw word diff
    // =========================================================================

    #[test]
    fn blockquote_markers_not_mangled() {
        // The old word-level diff would collapse "> foo\n> bar" into
        // "> ~~foo~~ > ~~bar~~" — a single line with embedded `>`
        let v1 = "> Line one.\n> Line two.\n";
        let v2 = "> Line one.\n> Line changed.\n";
        let result = diff_markdown_inline(v1, v2);
        eprintln!("--- blockquote not mangled ---\n{result}---");
        // Must start with blockquote
        assert!(
            result.starts_with("> "),
            "output must be a blockquote: {result}"
        );
        // Must not have bare `>` in the middle of a line (mangled)
        for line in result.lines() {
            if line.contains('>') {
                assert!(
                    line.starts_with('>'),
                    "bare > in middle of line — structure mangled: {line}"
                );
            }
        }
    }

    #[test]
    fn paragraph_breaks_preserved() {
        let v1 = "First paragraph.\n\nSecond paragraph.\n";
        let v2 = "First paragraph.\n\nChanged paragraph.\n";
        let result = diff_markdown_inline(v1, v2);
        eprintln!("--- paragraph breaks preserved ---\n{result}---");
        // Should have two separate paragraph blocks, not one merged line
        let non_empty_lines: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
        assert!(
            non_empty_lines.len() >= 2,
            "should be multiple paragraphs, got: {result}"
        );
    }

    // =========================================================================
    // Unit tests
    // =========================================================================

    #[test]
    fn diff_sequences_basic() {
        let old = vec![1, 2, 3, 4, 5];
        let new = vec![1, 3, 4, 6];
        let ops = diff_sequences(&old, &new);
        let equal: Vec<_> = ops
            .iter()
            .filter_map(|op| match op {
                DiffOp::Equal(v) => Some(**v),
                _ => None,
            })
            .collect();
        assert_eq!(equal, vec![1, 3, 4]);
    }

    #[test]
    fn unchanged_text_no_markers() {
        let md = "Hello world.\n";
        let result = diff_markdown(md, md);
        assert!(result.contains("Hello world."));
        assert!(!result.contains("~~"));
        assert!(!result.contains("**"));
    }

    #[test]
    fn heading_preserves_level() {
        let old = "## Old Title\n";
        let new = "## New Title\n";
        let result = diff_markdown_inline(old, new);
        assert!(result.contains("## "), "heading level preserved: {result}");
        assert!(result.contains("~~Old~~"), "old struck: {result}");
        assert!(result.contains("**New**"), "new bolded: {result}");
    }
}
