use clap::{Parser, ValueEnum};

/// Deterministic, read-only Linux gaming diagnostics from `GameDoc`.
#[derive(Debug, Parser)]
#[command(
    name = "gamedoc",
    version,
    about,
    long_about = "GameDoc inspects local Linux gaming components and prints a deterministic report.\n\nGameDoc never modifies the system, contacts the network, or sends telemetry.",
    after_help = "EXAMPLES:\n  gamedoc\n    Run and print a colorful terminal report\n\n  gamedoc --share\n    Run and print a sanitized terminal report, hiding usernames\n\n  gamedoc --format json\n    Run and output structured JSON data for external tools\n\n  gamedoc --format markdown\n    Run and output a Markdown document suitable for GitHub issues"
)]
pub struct Cli {
    /// Select a structured report format.
    #[arg(long, value_enum, conflicts_with = "share")]
    pub format: Option<OutputFormat>,

    /// Print a privacy-sanitized report suitable for sharing.
    #[arg(long)]
    pub share: bool,

    /// Print execution traces and durations for all collected evidence.
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Markdown,
    Json,
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
        assert!(!cli.debug);
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
    fn rejects_format_with_share_mode() {
        let result = Cli::try_parse_from(["gamedoc", "--format", "json", "--share"]);

        assert!(result.is_err());
    }
}
