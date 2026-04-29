use anyhow::Result;

static TOPICS: &[(&str, &str)] = &[
    ("commands", "All apm subcommands and their usage"),
    ("config",   "Fields available in .apm/config.toml"),
    ("workflow", "Fields available in .apm/workflow.toml"),
    ("ticket",   "Fields available in .apm/ticket.toml"),
];

// `apm help commands` uses flat word-wrapped lines rather than the
// column-aligned table format used by the config/workflow/ticket topics.
// The divergence is intentional: commands form a hierarchy
// (command → positionals → flags → subcommands) that does not fit a
// key-value table layout; schema topics describe flat key-value fields.
pub fn run(topic: Option<&str>, cli_cmd: clap::Command) -> Result<()> {
    match topic {
        None => {
            print!("{}", render_overview());
            Ok(())
        }
        Some(t) => {
            let content = match t {
                "commands" => render_commands(&cli_cmd),
                "config"   => render_config(),
                "workflow" => render_workflow(),
                "ticket"   => render_ticket(),
                unknown => {
                    let valid: Vec<&str> = TOPICS.iter().map(|(name, _)| *name).collect();
                    anyhow::bail!(
                        "unknown help topic {:?}; valid topics are: {}",
                        unknown,
                        valid.join(", ")
                    );
                }
            };
            print!("{}", content);
            Ok(())
        }
    }
}

fn render_overview() -> String {
    let mut out = String::new();
    out.push_str("apm help — topic reference for Agent Project Manager\n\n");
    out.push_str("Run `apm help <topic>` for details on a specific topic.\n");
    out.push_str("Run `apm <subcommand> --help` for flags on a specific command.\n\n");
    out.push_str("Topics:\n");
    for (name, summary) in TOPICS {
        out.push_str(&format!("  {:<10}  {}\n", name, summary));
    }
    out
}

fn render_commands(root: &clap::Command) -> String {
    let mut cmds: Vec<&clap::Command> = root
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .collect();
    cmds.sort_by_key(|c| c.get_name());

    let mut out = String::from("Commands\n========\n\n");
    let blocks: Vec<String> = cmds.iter().map(|c| render_one(c, "", 100)).collect();
    out.push_str(&blocks.join("\n\n"));
    out.push('\n');
    out
}

/// Render a single command (and recursively its subcommands) into a text block.
///
/// `prefix` is prepended to the usage line (e.g. "epic " for subcommands).
/// `max_width` is the line-length limit for wrapping; callers reduce it by 2
/// for each level of indentation that will be applied to the output.
fn render_one(cmd: &clap::Command, prefix: &str, max_width: usize) -> String {
    let name = cmd.get_name();
    let mut out = String::new();

    // Usage line: {prefix}{name} [<POS1> [POS2] ...]
    let positionals: Vec<String> = cmd
        .get_arguments()
        .filter(|a| {
            a.is_positional()
                && !a.is_hide_set()
                && a.get_id().as_str() != "help"
                && a.get_id().as_str() != "version"
        })
        .map(|a| {
            let vname = a
                .get_value_names()
                .and_then(|names| names.first())
                .map(|s| s.to_string())
                .unwrap_or_else(|| a.get_id().to_string().to_uppercase());
            if a.is_required_set() {
                format!("<{}>", vname)
            } else {
                format!("[{}]", vname)
            }
        })
        .collect();

    let usage = if positionals.is_empty() {
        format!("{}{}", prefix, name)
    } else {
        format!("{}{} {}", prefix, name, positionals.join(" "))
    };
    out.push_str(&usage);
    out.push('\n');

    // About text (one-liner from get_about())
    if let Some(about) = cmd.get_about() {
        let about_str = about.to_string();
        if !about_str.is_empty() {
            let wrapped = wrap_with_indent("  ", &about_str, max_width);
            out.push_str(&wrapped);
            out.push('\n');
        }
    }

    // Flags and options (non-positional, non-hidden, not auto-generated)
    for arg in cmd.get_arguments() {
        if arg.is_hide_set() {
            continue;
        }
        let id = arg.get_id().as_str();
        if id == "help" || id == "version" {
            continue;
        }
        if arg.is_positional() {
            continue;
        }
        let long = match arg.get_long() {
            Some(l) => l,
            None => continue,
        };

        // "  -s, --flag <VALUE>" or "  --flag <VALUE>" or "  --flag"
        let short_part = arg
            .get_short()
            .map(|s| format!("-{}, ", s))
            .unwrap_or_default();
        // Boolean flags (SetTrue / SetFalse / Count) take no value — omit the
        // <VALUE> placeholder. Other actions (Set, Append, …) display it.
        let takes_value = !matches!(
            arg.get_action(),
            clap::ArgAction::SetTrue | clap::ArgAction::SetFalse | clap::ArgAction::Count
        );
        let val_part = if takes_value {
            arg.get_value_names()
                .and_then(|names| names.first())
                .map(|v| format!(" <{}>", v))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let flag_head = format!("  {}--{}{}", short_part, long, val_part);

        // Help text, optionally followed by "(default: X)"
        let help_str = arg
            .get_help()
            .map(|h| h.to_string())
            .unwrap_or_default();
        let defaults: Vec<String> = arg
            .get_default_values()
            .iter()
            .map(|d| d.to_string_lossy().into_owned())
            .collect();
        // Append a "(default: X)" annotation only when the help text does not
        // already contain one (some commands embed the default in their doc comment).
        let full_help = if !defaults.is_empty() && !help_str.contains("(default:") {
            let def = defaults.join(", ");
            if help_str.is_empty() {
                format!("(default: {})", def)
            } else {
                format!("{} (default: {})", help_str, def)
            }
        } else {
            help_str
        };

        let line = if full_help.is_empty() {
            flag_head
        } else {
            // Two-space separator between flag definition and help text
            let first_prefix = format!("{}  ", flag_head);
            wrap_with_indent(&first_prefix, &full_help, max_width)
        };
        out.push_str(&line);
        out.push('\n');
    }

    // Subcommands (recursive, not re-sorted — declaration order preserved)
    let subcmds: Vec<&clap::Command> = cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .collect();
    if !subcmds.is_empty() {
        out.push('\n');
        let sub_prefix = format!("{}{} ", prefix, name);
        // Reduce the wrap limit by 2 to compensate for the 2-space indent
        // applied to each subcommand block below.
        let sub_max = max_width.saturating_sub(2);
        for sub in &subcmds {
            let block = render_one(sub, &sub_prefix, sub_max);
            for line in block.lines() {
                out.push_str("  ");
                out.push_str(line);
                out.push('\n');
            }
            out.push('\n');
        }
        // Drop the trailing blank line added after the last subcommand
        while out.ends_with("\n\n") {
            out.pop();
        }
    }

    out.trim_end().to_string()
}

/// Word-wrap `text` into lines of at most `max_width` characters.
///
/// The first line is prefixed with `first_prefix`. Continuation lines are
/// indented with the same number of spaces as `first_prefix` has characters,
/// so the text column stays aligned across wrapped lines.
fn wrap_with_indent(first_prefix: &str, text: &str, max_width: usize) -> String {
    if text.trim().is_empty() {
        return first_prefix.trim_end().to_string();
    }

    let cont_indent: String = " ".repeat(first_prefix.len());
    let mut result: Vec<String> = Vec::new();
    let mut current = first_prefix.to_string();

    for word in text.split_whitespace() {
        // current.trim().is_empty() is true when the line contains only spaces
        // (the initial prefix or a continuation indent) — no real text yet.
        if current.trim().is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            result.push(current);
            current = format!("{}{}", cont_indent, word);
        }
    }
    result.push(current);
    result.join("\n")
}

fn render_config() -> String {
    "apm help config — config.toml schema reference\n\nContent not yet implemented. See ticket d486d183.\n".to_string()
}

fn render_workflow() -> String {
    "apm help workflow — workflow.toml schema reference\n\nContent not yet implemented. See ticket 7ba021e8.\n".to_string()
}

fn render_ticket() -> String {
    "apm help ticket — ticket.toml schema reference\n\nContent not yet implemented. See ticket 14214305.\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_cmd() -> clap::Command {
        clap::Command::new("testapp")
            .subcommand(
                clap::Command::new("foo")
                    .about("Do foo things")
                    .arg(clap::Arg::new("id").value_name("ID").required(true))
                    .arg(
                        clap::Arg::new("verbose")
                            .long("verbose")
                            .short('v')
                            .action(clap::ArgAction::SetTrue)
                            .help("Enable verbose output"),
                    ),
            )
            .subcommand(
                clap::Command::new("bar")
                    .about("Do bar things")
                    .arg(
                        clap::Arg::new("count")
                            .long("count")
                            .value_name("N")
                            .default_value("1")
                            .help("Number of repetitions"),
                    ),
            )
            .subcommand(
                clap::Command::new("hidden")
                    .about("Should not appear")
                    .hide(true),
            )
            .subcommand(
                clap::Command::new("parent")
                    .about("Has subcommands")
                    .subcommand(
                        clap::Command::new("child")
                            .about("Child command"),
                    ),
            )
    }

    #[test]
    fn render_commands_includes_visible_cmds() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(out.contains("foo"), "missing 'foo' in:\n{out}");
        assert!(out.contains("bar"), "missing 'bar' in:\n{out}");
        assert!(out.contains("parent"), "missing 'parent' in:\n{out}");
    }

    #[test]
    fn render_commands_excludes_hidden() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(!out.contains("hidden"), "hidden cmd appeared in:\n{out}");
    }

    #[test]
    fn render_commands_alphabetical_order() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        let bar_pos = out.find("bar").unwrap();
        let foo_pos = out.find("foo").unwrap();
        let parent_pos = out.find("parent").unwrap();
        assert!(bar_pos < foo_pos, "'bar' should come before 'foo'");
        assert!(foo_pos < parent_pos, "'foo' should come before 'parent'");
    }

    #[test]
    fn render_commands_shows_about() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(out.contains("Do foo things"), "about missing in:\n{out}");
        assert!(out.contains("Do bar things"), "about missing in:\n{out}");
    }

    #[test]
    fn render_commands_shows_flags() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(out.contains("--verbose"), "flag missing in:\n{out}");
        assert!(out.contains("-v,"), "short flag missing in:\n{out}");
        assert!(out.contains("--count"), "flag missing in:\n{out}");
    }

    #[test]
    fn render_commands_shows_default() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(out.contains("(default: 1)"), "default annotation missing in:\n{out}");
    }

    #[test]
    fn render_commands_no_auto_flags() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(!out.contains("--help"), "--help appeared in:\n{out}");
        assert!(!out.contains("--version"), "--version appeared in:\n{out}");
    }

    #[test]
    fn render_commands_shows_subcommands() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(out.contains("parent child"), "subcommand missing in:\n{out}");
        assert!(out.contains("Child command"), "subcommand about missing in:\n{out}");
    }

    #[test]
    fn render_commands_shows_positional_in_usage() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(out.contains("<ID>"), "required positional missing in:\n{out}");
    }

    #[test]
    fn wrap_short_line_unchanged() {
        let result = wrap_with_indent("  ", "hello world", 100);
        assert_eq!(result, "  hello world");
    }

    #[test]
    fn wrap_long_line_breaks_at_word_boundary() {
        // Each word is 5 chars; prefix is 2 chars; max is 20.
        // "  alpha beta gamma delta" = 24 chars → should wrap.
        let result = wrap_with_indent("  ", "alpha beta gamma delta", 20);
        let lines: Vec<&str> = result.lines().collect();
        for line in &lines {
            assert!(
                line.len() <= 20,
                "line exceeds 20 chars: {:?}",
                line
            );
        }
        // All words must appear somewhere in the output
        assert!(result.contains("alpha"));
        assert!(result.contains("delta"));
    }

    #[test]
    fn wrap_continuation_lines_aligned() {
        // prefix = "  --flag  " (10 chars); text wraps; continuation should
        // also be indented 10 chars.
        let result = wrap_with_indent("  --flag  ", "word1 word2 word3 word4 word5 word6 word7 word8", 25);
        let lines: Vec<&str> = result.lines().collect();
        // First line starts with "  --flag  "
        assert!(lines[0].starts_with("  --flag  "), "first line: {:?}", lines[0]);
        // Continuation lines start with 10 spaces
        for line in lines.iter().skip(1) {
            assert!(
                line.starts_with("          "),
                "continuation line not indented: {:?}",
                line
            );
        }
    }

    #[test]
    fn no_ansi_in_output() {
        let root = make_test_cmd();
        let out = render_commands(&root);
        assert!(!out.contains('\x1b'), "ANSI escape code found in output");
    }
}
