<<<<<<< HEAD
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
=======
use std::io::Write as _;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use gamedoc::cli::{Cli, Commands, OutputFormat};
use gamedoc::probe::LocalSystemProbe;
use gamedoc::{engine, render};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
>>>>>>> 2109919 (v0.2.0)
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

<<<<<<< HEAD
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

=======
    if let Some(command) = cli.command {
        return run_command(&command);
    }

    if cli.no_color {
        gamedoc::render::color::disable_color();
    } else {
        gamedoc::render::color::respect_no_color_env();
    }
    let system = LocalSystemProbe::new();
    let report = engine::build_report(&system).public_safe(cli.share);

    let output = match cli.format {
        Some(OutputFormat::Markdown) => render::markdown::render_report(&report),
        Some(OutputFormat::Json) => render::json::render_report(&report)?,
        None => render::terminal::render_report(&report),
    };
    std::io::stdout().lock().write_all(output.as_bytes())?;
    Ok(())
}

fn run_command(command: &Commands) -> Result<()> {
    let mut cmd = Cli::command();
    match command {
        Commands::Completions { shell } => {
            clap_complete::generate(*shell, &mut cmd, "gamedoc", &mut std::io::stdout());
        }
        Commands::Man => {
            let man = clap_mangen::Man::new(cmd);
            man.render(&mut std::io::stdout())?;
        }
    }
>>>>>>> 2109919 (v0.2.0)
    Ok(())
}
