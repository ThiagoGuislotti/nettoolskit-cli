use nettoolskit_ui::render_markdown;

fn strip_ansi(input: &str) -> String {
    let mut stripped = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == b'\x1b' {
            index += 1;
            if index < bytes.len() && bytes[index] == b'[' {
                index += 1;
                while index < bytes.len() {
                    let byte = bytes[index];
                    if byte.is_ascii_alphabetic() {
                        index += 1;
                        break;
                    }
                    index += 1;
                }
                continue;
            }
        }

        stripped.push(bytes[index] as char);
        index += 1;
    }

    stripped
}

#[test]
fn render_markdown_renders_headings_and_lists() {
    let input = "# NetToolsKit\n\n## Commands\n\n- /help\n- /quit";
    let output = strip_ansi(&render_markdown(input));

    assert!(output.contains("NetToolsKit"));
    assert!(output.contains("Commands"));
    assert!(output.contains("- /help"));
    assert!(output.contains("- /quit"));
}

#[test]
fn render_markdown_renders_links_and_inline_code() {
    let input = "Read [docs](https://example.com) and run `ntk /help`.";
    let output = strip_ansi(&render_markdown(input));

    assert!(output.contains("docs"));
    assert!(output.contains("https://example.com"));
    assert!(output.contains("`ntk /help`"));
}

#[test]
fn render_markdown_renders_fenced_code_blocks() {
    let input = "```rust\nlet ok = true;\nprintln!(\"{ok}\");\n```";
    let output = strip_ansi(&render_markdown(input));

    assert!(output.contains("let ok = true;"));
    assert!(output.contains("println!(\"{ok}\");"));
}

#[test]
fn render_markdown_handles_task_list_markers() {
    let input = "- [x] done\n- [ ] pending";
    let output = strip_ansi(&render_markdown(input));

    assert!(output.contains("[x] done"));
    assert!(output.contains("[ ] pending"));
}
