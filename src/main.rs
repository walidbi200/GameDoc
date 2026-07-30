use anyhow::Result;
use clap::Parser;
use gamedoc::checks;
use gamedoc::cli::{Cli, OutputFormat};
use gamedoc::collect::context::CollectorContext;
use gamedoc::model::Report;
use gamedoc::render;

fn main() {
    if let Err(err) = run() {
        eprintln!("\n ✗ Error: {err}");
        let mut source = err.source();
        while let Some(s) = source {
            eprintln!("   ↳ {s}");
            source = s.source();
        }
        eprintln!();
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let ctx = CollectorContext::real();
    let diagnostics = checks::collect(&ctx);
    let report = Report::new(
        env!("CARGO_PKG_VERSION"),
        diagnostics.facts,
        diagnostics.checks,
        diagnostics.problems,
    );

    if cli.share {
        render::terminal::print_report(&report, /*sanitize=*/ true, cli.debug);
    } else {
        match cli.format {
            None => render::terminal::print_report(&report, /*sanitize=*/ false, cli.debug),
            Some(OutputFormat::Markdown) => print!("{}", render::markdown::render_report(&report)),
            Some(OutputFormat::Json) => print!("{}", render::json::render_report(&report)?),
        }
    }

    Ok(())
}
