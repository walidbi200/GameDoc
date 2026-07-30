//! GPU diagnostics collector.

use crate::collect::context::CollectorContext;
use crate::collect::{CollectionResult, Collector};
use crate::ids;
use crate::model::{Category, Check, Confidence, Evidence, EvidenceSource, Fact, Status, Trace};

pub struct GpuCollector;

impl Collector for GpuCollector {
    fn id(&self) -> &'static str {
        "gpu"
    }

    fn collect(&self, ctx: &CollectorContext) -> CollectionResult {
        let mut res = CollectionResult::new();
        check_lspci(ctx, &mut res);
        check_glxinfo(ctx, &mut res);
        check_vulkaninfo(ctx, &mut res);
        check_nvidia_smi(ctx, &mut res);
        res
    }
}

fn push_fact(res: &mut CollectionResult, id: &str, value: impl Into<String>, evidence: Evidence) {
    res.facts.push(Fact {
        id: id.to_owned(),
        category: Category::Gpu,
        value: value.into(),
        evidence,
    });
}

fn push_check(
    res: &mut CollectionResult,
    id: &str,
    status: Status,
    confidence: Confidence,
    summary: impl Into<String>,
    details: Vec<String>,
    evidence: Evidence,
) {
    res.checks.push(Check {
        id: id.to_owned(),
        category: Category::Gpu,
        status,
        confidence,
        summary: summary.into(),
        details,
        recommendation: None,
        evidence,
    });
}

fn check_lspci(ctx: &CollectorContext, res: &mut CollectionResult) {
    let output = ctx.command.run("lspci", &[]);
    let trace = output
        .as_ref()
        .map(|o| Trace {
            exit_code: o.exit_code,
            duration_ms: o.duration_ms,
        })
        .ok();

    let evidence = Evidence::new(EvidenceSource::Command, "lspci");
    let evidence = if let Some(t) = trace {
        evidence.with_trace(t)
    } else {
        evidence
    };

    match output {
        Ok(out) if out.success() => {
            let mut found = false;
            for line in out.stdout.lines() {
                if line.contains("VGA compatible controller")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
                {
                    let identity = line.split_once(": ").map_or(line, |x| x.1);
                    push_fact(res, ids::gpu::IDENTITY, identity.trim(), evidence.clone());
                    found = true;
                }
            }
            if !found {
                push_check(
                    res,
                    ids::gpu::IDENTITY,
                    Status::Warning,
                    Confidence::NotDetected,
                    "No GPU detected via lspci",
                    Vec::new(),
                    evidence,
                );
            }
        }
        _ => {
            push_check(
                res,
                ids::gpu::IDENTITY,
                Status::Warning,
                Confidence::Unknown,
                "lspci command failed or not found",
                Vec::new(),
                evidence,
            );
        }
    }
}

fn check_glxinfo(ctx: &CollectorContext, res: &mut CollectionResult) {
    let output = ctx.command.run("glxinfo", &["-B"]);
    let trace = output
        .as_ref()
        .map(|o| Trace {
            exit_code: o.exit_code,
            duration_ms: o.duration_ms,
        })
        .ok();

    let evidence = Evidence::new(EvidenceSource::Command, "glxinfo -B");
    let evidence = if let Some(t) = trace {
        evidence.with_trace(t)
    } else {
        evidence
    };

    match output {
        Ok(out) if out.success() => {
            let mut vendor = None;
            let mut renderer = None;
            let mut opengl = None;

            for line in out.stdout.lines() {
                if let Some(v) = line.strip_prefix("OpenGL vendor string: ") {
                    vendor = Some(v.trim());
                } else if let Some(r) = line.strip_prefix("OpenGL renderer string: ") {
                    renderer = Some(r.trim());
                } else if let Some(c) = line.strip_prefix("OpenGL core profile version string: ") {
                    opengl = Some(c.trim());
                } else if let Some(v) = line.strip_prefix("OpenGL version string: ") {
                    if opengl.is_none() {
                        opengl = Some(v.trim());
                    }
                }
            }

            if let Some(v) = vendor {
                push_fact(res, ids::gpu::VENDOR, v, evidence.clone());
            }
            if let Some(r) = renderer {
                push_fact(res, ids::gpu::RENDERER, r, evidence.clone());
                if r.to_lowercase().contains("mesa") {
                    push_fact(
                        res,
                        ids::gpu::MESA,
                        "Detected via renderer",
                        evidence.clone(),
                    );
                }
            }
            if let Some(o) = opengl {
                push_fact(res, ids::gpu::OPENGL, o, evidence.clone());
            } else {
                push_check(
                    res,
                    ids::gpu::OPENGL,
                    Status::Warning,
                    Confidence::NotDetected,
                    "OpenGL version could not be parsed",
                    Vec::new(),
                    evidence.clone(),
                );
            }
        }
        _ => {
            push_check(
                res,
                ids::gpu::OPENGL,
                Status::Warning,
                Confidence::Unknown,
                "glxinfo command failed or not found",
                Vec::new(),
                evidence,
            );
        }
    }
}

fn check_vulkaninfo(ctx: &CollectorContext, res: &mut CollectionResult) {
    let output = ctx.command.run("vulkaninfo", &["--summary"]);
    let trace = output
        .as_ref()
        .map(|o| Trace {
            exit_code: o.exit_code,
            duration_ms: o.duration_ms,
        })
        .ok();

    let evidence = Evidence::new(EvidenceSource::Command, "vulkaninfo --summary");
    let evidence = if let Some(t) = trace {
        evidence.with_trace(t)
    } else {
        evidence
    };

    match output {
        Ok(out) if out.success() => {
            let mut vulkan_ver = None;
            for line in out.stdout.lines() {
                if let Some(v) = line.strip_prefix("Vulkan Instance Version: ") {
                    vulkan_ver = Some(v.trim());
                    break;
                }
            }
            if let Some(v) = vulkan_ver {
                push_fact(res, ids::gpu::VULKAN, v, evidence);
            } else {
                push_check(
                    res,
                    ids::gpu::VULKAN,
                    Status::Warning,
                    Confidence::Unknown,
                    "Vulkan detected but version could not be parsed",
                    Vec::new(),
                    evidence,
                );
            }
        }
        _ => {
            push_check(
                res,
                ids::gpu::VULKAN,
                Status::Warning,
                Confidence::Unknown,
                "vulkaninfo command failed or not found",
                Vec::new(),
                evidence,
            );
        }
    }
}

fn check_nvidia_smi(ctx: &CollectorContext, res: &mut CollectionResult) {
    let output = ctx.command.run("nvidia-smi", &[]);
    let trace = output
        .as_ref()
        .map(|o| Trace {
            exit_code: o.exit_code,
            duration_ms: o.duration_ms,
        })
        .ok();

    let evidence = Evidence::new(EvidenceSource::Command, "nvidia-smi");
    let evidence = if let Some(t) = trace {
        evidence.with_trace(t)
    } else {
        evidence
    };

    match output {
        Ok(out) if out.success() => {
            let mut driver = None;
            // E.g., "NVIDIA-SMI 535.113.01   Driver Version: 535.113.01   CUDA Version: 12.2"
            for line in out.stdout.lines() {
                if line.contains("Driver Version: ") {
                    if let Some(part) = line.split("Driver Version: ").nth(1) {
                        if let Some(v) = part.split_whitespace().next() {
                            driver = Some(v);
                        }
                    }
                    break;
                }
            }
            if let Some(d) = driver {
                push_fact(res, ids::gpu::NVIDIA, d, evidence.clone());
                push_fact(
                    res,
                    ids::gpu::DRIVER,
                    format!("nvidia {d}"),
                    evidence.clone(),
                );
            } else {
                push_check(
                    res,
                    ids::gpu::NVIDIA,
                    Status::Warning,
                    Confidence::Detected,
                    "nvidia-smi succeeded but driver version missing",
                    Vec::new(),
                    evidence,
                );
            }
        }
        _ => {
            // It's normal to not have nvidia-smi if the user doesn't have an NVIDIA GPU.
            // We just don't emit anything (or maybe a NotDetected check).
            push_check(
                res,
                ids::gpu::NVIDIA,
                Status::Healthy,
                Confidence::NotDetected,
                "NVIDIA GPU not detected",
                Vec::new(),
                evidence,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::GpuCollector;
    use crate::collect::command::{CommandOutput, CommandRunner};
    use crate::collect::context::CollectorContext;
    use crate::collect::filesystem::StubFilesystem;
    use crate::collect::Collector;
    use crate::errors::CollectError;
    use crate::ids;

    struct StubCommandRunner(HashMap<String, String>);

    impl CommandRunner for StubCommandRunner {
        fn run(&self, program: &str, _args: &[&str]) -> Result<CommandOutput, CollectError> {
            self.0.get(program).map_or_else(
                || {
                    Err(CollectError::CommandNotFound {
                        program: program.to_owned(),
                    })
                },
                |stdout| {
                    Ok(CommandOutput {
                        stdout: stdout.clone(),
                        stderr: String::new(),
                        exit_code: Some(0),
                        duration_ms: 5,
                    })
                },
            )
        }
    }

    fn ctx(cmd: StubCommandRunner) -> CollectorContext {
        CollectorContext::new(Arc::new(cmd), Arc::new(StubFilesystem::new()))
    }

    #[test]
    fn lspci_parses_identity() {
        let mut cmds = HashMap::new();
        cmds.insert("lspci".to_owned(), "03:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Navi 21 [Radeon RX 6800/6800 XT / 6900 XT] (rev c3)".to_owned());
        let res = GpuCollector.collect(&ctx(StubCommandRunner(cmds)));

        let identity = res
            .facts
            .iter()
            .find(|f| f.id == ids::gpu::IDENTITY)
            .unwrap();
        assert!(identity.value.contains("Advanced Micro Devices"));
    }

    #[test]
    fn glxinfo_parses_vendor_and_opengl() {
        let mut cmds = HashMap::new();
        cmds.insert("glxinfo".to_owned(), "OpenGL vendor string: AMD\nOpenGL renderer string: AMD Radeon RX 6800 (Mesa, LLVM)\nOpenGL core profile version string: 4.6 (Core Profile) Mesa 23.1.6".to_owned());
        let res = GpuCollector.collect(&ctx(StubCommandRunner(cmds)));

        let vendor = res.facts.iter().find(|f| f.id == ids::gpu::VENDOR).unwrap();
        assert_eq!(vendor.value, "AMD");

        let renderer = res
            .facts
            .iter()
            .find(|f| f.id == ids::gpu::RENDERER)
            .unwrap();
        assert!(renderer.value.contains("Radeon"));

        let opengl = res.facts.iter().find(|f| f.id == ids::gpu::OPENGL).unwrap();
        assert!(opengl.value.contains("4.6"));

        let mesa = res.facts.iter().find(|f| f.id == ids::gpu::MESA).unwrap();
        assert_eq!(mesa.value, "Detected via renderer");
    }

    #[test]
    fn vulkaninfo_parses_version() {
        let mut cmds = HashMap::new();
        cmds.insert(
            "vulkaninfo".to_owned(),
            "Vulkan Instance Version: 1.3.255".to_owned(),
        );
        let res = GpuCollector.collect(&ctx(StubCommandRunner(cmds)));

        let vulkan = res.facts.iter().find(|f| f.id == ids::gpu::VULKAN).unwrap();
        assert_eq!(vulkan.value, "1.3.255");
    }

    #[test]
    fn nvidia_smi_parses_driver() {
        let mut cmds = HashMap::new();
        cmds.insert(
            "nvidia-smi".to_owned(),
            "NVIDIA-SMI 535.113.01   Driver Version: 535.113.01   CUDA Version: 12.2".to_owned(),
        );
        let res = GpuCollector.collect(&ctx(StubCommandRunner(cmds)));

        let nvidia = res.facts.iter().find(|f| f.id == ids::gpu::NVIDIA).unwrap();
        assert_eq!(nvidia.value, "535.113.01");

        let driver = res.facts.iter().find(|f| f.id == ids::gpu::DRIVER).unwrap();
        assert_eq!(driver.value, "nvidia 535.113.01");
    }
}
