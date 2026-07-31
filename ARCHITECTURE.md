# GameDoc Architecture

This document is GameDoc's architectural constitution. It defines the
boundaries that keep collection safe, evaluation deterministic, and output
reproducible as new diagnostic domains are added.

Changes to these rules require a concrete implementation need, an explicit
design decision, and an update to this document. Convenience alone is not a
reason to bypass a boundary.

## Goals

- Diagnose Linux gaming systems without changing them.
- Preserve incomplete, missing, and failed observations as reportable evidence.
- Keep platform I/O replaceable with deterministic fixtures.
- Separate observation, interpretation, problem synthesis, and presentation.
- Produce stable terminal, Markdown, and versioned JSON reports.
- Make new collectors and rules local additions rather than architectural work.

## Non-goals

- Optimizing, repairing, or configuring the user's machine.
- Installing packages or starting, stopping, or enabling services.
- Running shell expressions, pipelines, or `sh -c`.
- Accessing the network or sending telemetry.
- Treating unavailable evidence as proof that a feature is absent.
- Implementing collector caching or automated fixes in version 1.

## Invariants

These rules are non-negotiable:

1. **Collectors only observe.** They do not interpret overall health, emit
   problems, recommend fixes, or mutate the machine.
2. **`SystemProbe` is the only layer allowed to perform platform I/O.**
   Collectors do not use `std::fs`, `std::process`, environment APIs, or other
   external resources directly.
3. **Facts are immutable after collection completes.** Collectors append to a
   `FactBuilder`; rules receive only the `Facts` produced by `build()`.
4. **Rules are pure functions over facts.** Rule evaluation performs no I/O,
   changes no facts, and produces the same outcome for the same facts.
5. **Problem synthesis consumes rule outcomes only.** It cannot inspect the
   host, invoke collectors, or reinterpret raw probe output.
6. **Renderers consume finalized reports only.** Rendering cannot collect,
   evaluate, synthesize, or mutate report data.
7. **No layer may bypass the layer below it.** Data flows through the pipeline
   defined here; shortcuts across boundaries are architectural defects.
8. **The JSON schema is versioned and backward-compatible within a major
   version.** Existing fields, meanings, enum values, and stable IDs are not
   removed or changed incompatibly within that major version.

## Data Flow

```text
                    SystemProbe
               (read-only platform API)
                        |
                        v
                  Collectors
                        |
                        v
                Mutable FactBuilder
                        |
                     build()
                        |
                        v
                 Immutable Facts
                        |
                        v
                  Rule Engine
               (pure evaluation)
                        |
                        v
                 Rule Outcomes
                        |
                        v
              Problem Synthesizer
                        |
                        v
                  Final Report
                        |
          +-------------+-------------+
          |             |             |
          v             v             v
      Terminal       Markdown        JSON
```

Each arrow is a one-way ownership and dependency boundary. Information may be
normalized as it moves down the pipeline, but later layers cannot reach back
into earlier layers.

## System Probe

`SystemProbe` is the complete read-only platform API available to collectors.
Its implementations may provide capabilities such as:

```rust,ignore
pub trait SystemProbe {
    fn command(&self, program: &str, args: &[&str]) -> ProbeResult<CommandOutput>;
    fn read(&self, path: &Path) -> ProbeResult<String>;
    fn exists(&self, path: &Path) -> bool;
    fn which(&self, program: &str) -> Option<PathBuf>;
    fn env(&self, name: &str) -> Option<OsString>;
    fn metadata(&self, path: &Path) -> ProbeResult<Metadata>;
}
```

The exact signatures may evolve with implementation needs. The capability
boundary may not expand to include mutation. In particular, `SystemProbe` has
no filesystem write, directory creation, shell execution, package operation,
service control, or network method.

Commands are always an explicit executable and argument list. Implementations
must not invoke a shell, must enforce a timeout, and must capture standard
output and standard error separately.

Real probes operate on the local system. Fixture probes return canned command,
filesystem, environment, and metadata observations. Collectors must behave the
same way with either implementation.

## Collection

A collector owns observation for one cohesive subsystem. The collector
registry has an explicit order, and the engine invokes collectors in that
order.

Collection does not return `Result`. A missing command, permission denial,
timeout, unreadable file, unsupported subsystem, or non-zero command exit is an
observation about the user's machine, not an engine failure:

```rust,ignore
pub trait Collector {
    fn id(&self) -> CollectorId;
    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult;
}

pub struct CollectionResult {
    pub facts: Vec<Fact>,
    pub diagnostics: Vec<CollectorDiagnostic>,
}
```

Diagnostics preserve unsuccessful collection attempts without forcing a
collector to invent a fact. They are available to report finalization and may
explain why evidence is incomplete. Expected host conditions never panic and
never abort collection of unrelated subsystems.

Fatal termination is reserved for conditions in which GameDoc cannot produce a
trustworthy report, such as invalid internal state, a corrupted fixture, or a
programming defect.

## Facts

Facts are an append-only stream of observations. A fact ID is not a map key,
and adding a fact never overwrites an earlier fact. Multiple GPUs, Steam
libraries, Proton versions, and other repeated observations retain their full
cardinality.

Rules query immutable facts using operations such as `first`, `all`, `count`,
and `exists`. Where `first` has meaning, it refers to deterministic fact order,
not incidental hash-map or filesystem order.

Domain identifiers use distinct newtypes:

```rust,ignore
pub struct FactId(&'static str);
pub struct CollectorId(&'static str);
pub struct RuleId(&'static str);
pub struct ProblemId(&'static str);
```

These types prevent accidental interchange even when their serialized forms
are strings. Stable IDs follow the established `<category>.<name>` convention.

Facts become immutable through an ownership transition:

```rust,ignore
let mut builder = FactBuilder::new();
builder.append(collection_result.facts);
let facts: Facts = builder.build();
```

`Facts` exposes no mutation API. `FactBuilder` is unavailable to the rule
engine, problem synthesizer, and renderers.

## Provenance and Debug Traces

Every fact always carries provenance identifying where the observation came
from. Provenance includes the collector and the observation method, such as a
command, file, path lookup, environment variable, or metadata query.

Debug traces are separate from facts and provenance. A trace may include the
exact command and arguments, duration, exit code, standard output, standard
error, or a probe error. These details can be verbose, sensitive, and specific
to an implementation attempt; they are not the fact itself.

Finalization applies privacy policy before traces reach a report. Renderers
only decide how already-approved report traces are displayed. They do not gain
access to raw traces or perform redaction themselves.

## Rules

Rules interpret immutable facts and produce rule outcomes. They declare their
inputs rather than reimplementing dependency checks:

```rust,ignore
pub trait Rule {
    fn id(&self) -> RuleId;
    fn requires(&self) -> &'static [FactId];
    fn optional(&self) -> &'static [FactId];
    fn evaluate(&self, facts: &Facts) -> RuleOutcome;
}
```

The engine evaluates rules in `RuleId` order. If any required fact is absent,
the engine emits a skipped outcome without calling `evaluate`. Missing optional
facts do not skip a rule; the rule decides how they affect confidence or
detail. An empty `requires` set is valid.

A rule outcome records what the rule concluded and why. It is not yet a
terminal paragraph or a JSON object tailored to one renderer.

## Problem Synthesis

The problem synthesizer consumes the complete set of rule outcomes and creates
the report's user-facing problems. This is the only layer that may combine
related outcomes, suppress redundant symptoms, assign final severity, and
attach recommendations.

The synthesizer cannot query facts directly. If synthesis needs information,
the relevant rule outcome must carry it explicitly. This preserves the rule
engine as the sole interpreter of facts.

Future fixes may consume finalized problems, but they belong to a separate,
explicitly invoked subsystem. A future `gamedoc fix` must not add mutation to
collectors, `SystemProbe`, rules, synthesis, or report rendering.

## Reports and Rendering

A final report is a renderer-independent data model. It contains the schema
version, GameDoc version, finalized problems and outcomes intended for output,
approved evidence, collector diagnostics, and any privacy-filtered debug
information selected by report policy.

Terminal, Markdown, and JSON renderers receive `&Report` only. They may format,
escape, and omit fields according to the documented output mode. They may not
change diagnostic meaning or perform host I/O.

Share-mode sanitization occurs before rendering so every renderer receives the
same safe report. Adding a new renderer must not require changes to collection
or evaluation.

## Determinism

All observable ordering is defined:

| Data | Order |
| --- | --- |
| Collectors | Explicit registry order |
| Facts | Category, fact ID, normalized value |
| Rules and outcomes | Rule ID |
| Problems | Severity, category, problem ID |
| Directory entries | Lexical name order |
| JSON collections | The corresponding canonical order above |

When values are structured, their canonical serialized representation defines
the final comparison. Stable sorting preserves collection order for otherwise
equal facts.

Inputs with no semantic ordering, including filesystem iteration and hash-map
iteration, must be normalized before entering output-visible data. Given the
same fixture and GameDoc version, collection through rendering must produce
byte-for-byte identical JSON.

## JSON Compatibility

The top-level JSON report includes a schema version. Schema compatibility is
independent from the GameDoc binary version.

Within one schema major version:

- Existing fields are not removed or renamed.
- Existing field meanings and types are not changed incompatibly.
- Existing enum values and stable IDs are not renamed or removed.
- New fields are additive and must have backward-compatible absence semantics.
- Ordering remains canonical where arrays are used.

An incompatible change requires a new schema major version, migration notes,
and updated serialization fixtures. JSON shape and ordering are covered by
tests.

## Caching

Collector caching is intentionally deferred. The architecture may later cache
versioned collector results using hashes of relevant probe inputs and
collector implementation versions. Caching must preserve the same facts,
diagnostics, ordering, provenance, and privacy behavior as uncached collection.

No current API may assume every collector executes on every run, but no cache
may bypass report correctness merely because broad machine state appears
unchanged.

## Failure Model

Expected machine-level failures become collector diagnostics and, where
appropriate, facts expressing an unknown or unavailable observation. They do
not become top-level engine errors.

Top-level errors are limited to failures outside normal diagnosis, including
invalid CLI usage, corrupted fixtures, internal invariant violations, and an
inability to serialize or write the requested output. One collector's failure
must not discard successful observations from other collectors.

## Implementation Status

The core pipeline is implemented:

- `SystemProbe` is the collector-facing I/O boundary, with local and fixture
  implementations.
- `Collector` emits `CollectionResult` containing facts and diagnostics.
- `FactBuilder::build` creates the immutable `Facts` input to rules.
- Rules declare required and optional facts and produce `RuleOutcome` values.
- Problem synthesis consumes rule outcomes only.
- Renderers consume only finalized `Report` values.
- JSON reports identify the public schema as `gamedoc.v1`.

The v0.1 collector registry covers system, CPU, distribution, graphics, Steam,
MangoHud, and GameMode observations. Facts are namespaced as `system.*`,
`cpu.*`, `distro.*`, `gpu.*`, `steam.*`, `mangohud.*`, and `gamemode.*`.
The rule catalog covers evidence-backed Vulkan, Steam, and GameMode conditions.
Expanding domain coverage must preserve the implemented boundaries rather than
introduce a parallel path.

The CLI supports `--format`, `--share`, `--no-color`, `completions`, and `man`
subcommands. Shell completion scripts are generated at runtime by `clap_complete`;
man pages are generated at runtime by `clap_mangen`.

All CLI output is privacy-finalized before rendering. Normal output may include
sanitized collector diagnostics; `--share` omits diagnostics. Facts and
provenance must use semantic labels instead of usernames, home paths, account
IDs, environment values, launch arguments, or raw stderr.

## Change Policy

An architectural change should answer all of the following:

1. What concrete implementation problem cannot be solved within these
   boundaries?
2. Which invariant or layer changes?
3. How are safety, determinism, testability, privacy, and JSON compatibility
   preserved?
4. What migration and rollback steps are required?
5. Which tests prove the new boundary?

If those questions do not have specific answers, the architecture remains as
defined here.
