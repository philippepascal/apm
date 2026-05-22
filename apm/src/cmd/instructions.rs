use anyhow::Result;

const PREAMBLE: &str = "apm \u{2014} Agent Project Manager\n\
Run `apm <command> --help` for full flag details on any command.\n";

pub fn run(cli_cmd: clap::Command) -> Result<()> {
    print!("{}", render(cli_cmd));
    Ok(())
}

fn render(cli_cmd: clap::Command) -> String {
    let mut out = String::from(PREAMBLE);
    out.push('\n');
    out.push_str(&super::help::render_commands(&cli_cmd));
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_cmd() -> clap::Command {
        clap::Command::new("testapp")
            .subcommand(clap::Command::new("foo").about("Do foo things"))
            .subcommand(clap::Command::new("bar").about("Do bar things"))
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
}
