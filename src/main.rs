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
use gamedoc::facts::{Category, Fact, FactBuilder, FactSource, ObservationMethod};
use gamedoc::ids::{collector, fact};
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
    let (report, duration_ms) = if cli.demo {
        (mock_demo_report(), 183)
    } else {
        let start_time = std::time::Instant::now();
        let system = LocalSystemProbe::new();
        let report = engine::build_report(&system).public_safe(cli.share);
        let duration = start_time
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);
        (report, duration)
    };

    let output = match cli.format {
        Some(OutputFormat::Markdown) => render::markdown::render_report(&report),
        Some(OutputFormat::Json) => render::json::render_report(&report)?,
        None => render::terminal::render_report(
            &report,
            &render::terminal::TerminalConfig {
                summary: cli.summary,
                verbose: cli.verbose,
                ascii: cli.ascii,
                duration_ms,
            },
        ),
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

fn mock_demo_report() -> gamedoc::model::Report {
    let mut builder = FactBuilder::new();
    let src = FactSource {
        collector: collector::DISTRO,
        method: ObservationMethod::Command("demo".to_owned()),
    };

    builder.append([
        Fact::observed(
            fact::DISTRO_PRETTY_NAME,
            Category::Distro,
            "Fedora Linux 44",
            src.clone(),
        ),
        Fact::observed(
            fact::KERNEL_VERSION,
            Category::System,
            "6.12.0",
            src.clone(),
        ),
        Fact::observed(fact::DESKTOP, Category::System, "GNOME", src.clone()),
        Fact::observed(fact::SESSION_TYPE, Category::System, "Wayland", src.clone()),
        Fact::observed(
            fact::CPU_MODEL,
            Category::Cpu,
            "AMD Ryzen 7 7800X3D",
            src.clone(),
        ),
        Fact::observed(
            fact::GPU_DEVICE,
            Category::Gpu,
            "NVIDIA RTX 4070 SUPER",
            src.clone(),
        ),
        Fact::observed(
            fact::OPENGL_RENDERER,
            Category::Gpu,
            "NVIDIA RTX 4070 SUPER/PCIe/SSE2",
            src.clone(),
        ),
        Fact::observed(
            fact::OPENGL_VERSION,
            Category::Gpu,
            "4.6.0 NVIDIA 575.xx",
            src.clone(),
        ),
        Fact::observed(fact::VULKAN_AVAILABLE, Category::Gpu, true, src.clone()),
        Fact::observed(
            fact::STEAM_INSTALLATION,
            Category::Steam,
            "native",
            src.clone(),
        ),
        Fact::observed(
            fact::GAMEMODE_DAEMON_RUNNING,
            Category::GameMode,
            true,
            src.clone(),
        ),
        Fact::observed(
            fact::MANGOHUD_INSTALLATION,
            Category::MangoHud,
            "executable",
            src.clone(),
        ),
        Fact::observed(
            fact::GAMESCOPE_INSTALLATION,
            Category::System,
            true,
            src.clone(),
        ),
    ]);

    gamedoc::model::Report::new(builder.build(), vec![], vec![])
}
