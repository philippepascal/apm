use anyhow::Result;

static TOPICS: &[(&str, &str)] = &[
    ("commands", "All apm subcommands and their usage"),
    ("config",   "Fields available in .apm/config.toml"),
    ("workflow", "Fields available in .apm/workflow.toml"),
    ("ticket",   "Fields available in .apm/ticket.toml"),
];

pub fn run(topic: Option<&str>) -> Result<()> {
    match topic {
        None => {
            print!("{}", render_overview());
            Ok(())
        }
        Some(t) => {
            let content = match t {
                "commands" => render_commands(),
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

fn render_commands() -> String {
    "apm help commands — full command reference\n\nContent not yet implemented. See ticket 3665e017.\n".to_string()
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
