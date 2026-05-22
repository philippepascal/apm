use anyhow::Result;

const PREAMBLE: &str = "apm \u{2014} Agent Project Manager\n\
Run `apm <command> --help` for full flag details on any command.\n";

pub fn run(cli_cmd: clap::Command) -> Result<()> {
    print!("{}", render(cli_cmd));
    Ok(())
}

fn render_compact_commands(cli_cmd: &clap::Command) -> String {
    let mut cmds: Vec<&clap::Command> = cli_cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .collect();
    cmds.sort_by_key(|c| c.get_name());

    // Compute column width: len("apm ") + longest name
    let max_name = cmds.iter().map(|c| c.get_name().len()).max().unwrap_or(0);
    let col_width = 4 + max_name; // "apm " prefix

    let mut out = String::new();
    for cmd in &cmds {
        let label = format!("apm {}", cmd.get_name());
        let about = cmd.get_about().map(|a| a.to_string()).unwrap_or_default();
        out.push_str(&format!("  {:<col_width$}  {}\n", label, about));
    }
    out
}

fn render(cli_cmd: clap::Command) -> String {
    let mut out = String::from(PREAMBLE);
    out.push('\n');
    out.push_str(&render_compact_commands(&cli_cmd));
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_cmd() -> clap::Command {
        clap::Command::new("testapp")
            .subcommand(
                clap::Command::new("foo")
                    .about("Do foo things")
                    .arg(
                        clap::Arg::new("verbose")
                            .long("verbose")
                            .action(clap::ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                clap::Command::new("bar")
                    .about("Do bar things")
                    .arg(
                        clap::Arg::new("count")
                            .long("count")
                            .value_name("N")
                            .action(clap::ArgAction::Set),
                    ),
            )
            .subcommand(clap::Command::new("_hook").about("Hidden hook").hide(true))
    }

    #[test]
    fn run_returns_ok() {
        let result = run(make_test_cmd());
        assert!(result.is_ok());
    }

    #[test]
    fn render_includes_preamble() {
        let out = render(make_test_cmd());
        assert!(
            out.contains("apm \u{2014} Agent Project Manager"),
            "preamble missing in:\n{out}"
        );
    }

    #[test]
    fn render_includes_command_name() {
        let out = render(make_test_cmd());
        assert!(out.contains("foo"), "command name 'foo' missing in:\n{out}");
    }

    #[test]
    fn render_no_ansi() {
        let out = render(make_test_cmd());
        assert!(!out.contains('\x1b'), "ANSI escape code found in:\n{out}");
    }

    #[test]
    fn render_compact_has_apm_prefix() {
        let out = render(make_test_cmd());
        assert!(out.contains("apm foo"), "apm foo prefix missing in:\n{out}");
        assert!(out.contains("apm bar"), "apm bar prefix missing in:\n{out}");
    }

    #[test]
    fn render_compact_shows_about() {
        let out = render(make_test_cmd());
        assert!(out.contains("Do foo things"), "about for foo missing in:\n{out}");
        assert!(out.contains("Do bar things"), "about for bar missing in:\n{out}");
    }

    #[test]
    fn render_compact_no_flags() {
        let out = render(make_test_cmd());
        assert!(!out.contains("--verbose"), "flag --verbose found in:\n{out}");
        assert!(!out.contains("--count"), "flag --count found in:\n{out}");
    }

    #[test]
    fn render_compact_excludes_hidden() {
        let out = render(make_test_cmd());
        assert!(!out.contains("_hook"), "hidden subcommand _hook found in:\n{out}");
    }
}
