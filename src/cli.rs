use clap::{Parser, ValueEnum};

/// Deterministic, read-only Linux gaming diagnostics from `GameDoc`.
#[derive(Debug, Parser)]
#[command(
    name = "gamedoc",
    version,
    about,
    long_about = "GameDoc inspects local Linux gaming components and prints a deterministic report.\n\nGameDoc never modifies the system, contacts the network, or sends telemetry."
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Select a structured report format.
    #[arg(long, value_enum, conflicts_with_all = ["summary", "verbose"])]
    pub format: Option<OutputFormat>,

    /// Show only high-level health status and problems.
    #[arg(long, conflicts_with_all = ["verbose", "format"])]
    pub summary: bool,

    /// Show detailed raw diagnostic facts below the dashboard.
    #[arg(long, conflicts_with_all = ["summary", "format"])]
    pub verbose: bool,

    /// Use ASCII characters instead of Unicode
    #[arg(long, env = "GAMEDOC_ASCII")]
    pub ascii: bool,

    /// Run in deterministic demo mode (useful for screenshots)
    #[arg(long, env = "GAMEDOC_DEMO", hide = true)]
    pub demo: bool,

    /// Print a privacy-sanitized report suitable for sharing.
    #[arg(long)]
    pub share: bool,

    /// Disable colored output even when connected to a terminal.
    #[arg(long)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Markdown,
    Json,
}

#[derive(Debug, Parser)]
pub enum Commands {
    /// Generate shell completion script for the specified shell.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Generate the man page in roff format.
    Man,
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::{Cli, OutputFormat};

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_default_invocation() {
        let cli = Cli::try_parse_from(["gamedoc"]).expect("default CLI should parse");

        assert_eq!(cli.format, None);
        assert!(!cli.share);
        assert!(!cli.summary);
        assert!(!cli.verbose);
        assert!(!cli.ascii);
    }

    #[test]
    fn parses_supported_formats() {
        let markdown = Cli::try_parse_from(["gamedoc", "--format", "markdown"])
            .expect("markdown format should parse");
        let json =
            Cli::try_parse_from(["gamedoc", "--format", "json"]).expect("JSON format should parse");

        assert_eq!(markdown.format, Some(OutputFormat::Markdown));
        assert_eq!(json.format, Some(OutputFormat::Json));
    }

    #[test]
    fn allows_format_with_share_mode() {
        let result = Cli::try_parse_from(["gamedoc", "--format", "json", "--share"]);

        assert!(result.is_ok());
        let cli = result.unwrap();
        assert_eq!(cli.format, Some(OutputFormat::Json));
        assert!(cli.share);
    }
}
