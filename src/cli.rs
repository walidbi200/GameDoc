use clap::{Parser, ValueEnum};

/// Deterministic, read-only Linux gaming diagnostics from `GameDoc`.
#[derive(Debug, Parser)]
#[command(
    name = "gamedoc",
    version,
    about,
<<<<<<< HEAD
    long_about = "GameDoc inspects local Linux gaming components and prints a deterministic report.\n\nGameDoc never modifies the system, contacts the network, or sends telemetry.",
    after_help = "EXAMPLES:\n  gamedoc\n    Run and print a colorful terminal report\n\n  gamedoc --share\n    Run and print a sanitized terminal report, hiding usernames\n\n  gamedoc --format json\n    Run and output structured JSON data for external tools\n\n  gamedoc --format markdown\n    Run and output a Markdown document suitable for GitHub issues"
)]
pub struct Cli {
    /// Select a structured report format.
    #[arg(long, value_enum, conflicts_with = "share")]
=======
    long_about = "GameDoc inspects local Linux gaming components and prints a deterministic report.\n\nGameDoc never modifies the system, contacts the network, or sends telemetry."
)]
pub struct Cli {
    /// Select a structured report format.
    #[arg(long, value_enum)]
>>>>>>> 2109919 (v0.2.0)
    pub format: Option<OutputFormat>,

    /// Print a privacy-sanitized report suitable for sharing.
    #[arg(long)]
    pub share: bool,

<<<<<<< HEAD
    /// Print execution traces and durations for all collected evidence.
    #[arg(short, long)]
    pub debug: bool,
=======
    /// Disable colored output even when connected to a terminal.
    #[arg(long)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
>>>>>>> 2109919 (v0.2.0)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Markdown,
    Json,
}

<<<<<<< HEAD
=======
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

>>>>>>> 2109919 (v0.2.0)
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
<<<<<<< HEAD
        assert!(!cli.debug);
=======
>>>>>>> 2109919 (v0.2.0)
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
<<<<<<< HEAD
    fn rejects_format_with_share_mode() {
        let result = Cli::try_parse_from(["gamedoc", "--format", "json", "--share"]);

        assert!(result.is_err());
=======
    fn allows_format_with_share_mode() {
        let result = Cli::try_parse_from(["gamedoc", "--format", "json", "--share"]);

        assert!(result.is_ok());
        let cli = result.unwrap();
        assert_eq!(cli.format, Some(OutputFormat::Json));
        assert!(cli.share);
>>>>>>> 2109919 (v0.2.0)
    }
}
