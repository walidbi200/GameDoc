# Roadmap

<<<<<<< HEAD
GameDoc is developed iteratively with a strict adherence to its core architectural constitution.

## Current Status (v0.1.0)
- ✅ Core Architecture & Immutable Facts Pipeline
- ✅ System Collector (OS, Memory, Display Server)
- ✅ GPU Collector (Vulkan, OpenGL, Vendor detection)
- ✅ Steam Collector (Native, Flatpak, Snap, Proton)
- ✅ Output Formats: Terminal, Markdown, JSON (Schema v2)

## Future Work (v0.2.0 and beyond)
- [ ] **GameMode Detection**: Detect if Feral GameMode is installed and active.
- [ ] **MangoHud Detection**: Check MangoHud configuration and globally enabled status.
- [ ] **Lutris / Heroic Support**: Extend collection to other major Linux gaming launchers.
- [ ] **Anti-Cheat Validation**: Verify kernel support for common anti-cheat enablers.
- [ ] **Wayland Protocols**: Deeper diagnostics for tearing updates and fractional scaling.
=======
GameDoc v0.1.0 establishes the public architecture and JSON schema. Future work
must remain read-only and additive within `gamedoc.v1`.

## Near Term

- Validate additional Steam `libraryfolders.vdf` variants.
- Add renderer escaping tests for more hostile local text.
- Add distribution packaging after repository ownership is finalized.
- Expand fixture coverage across more distributions and hybrid-GPU setups.

## Later

- Heroic, Lutris, and Bottles collectors.
- More structured GPU and driver facts.
- Optional debug traces behind an explicit privacy policy.
- Collector caching that preserves uncached semantics.

## Not Planned for v1

- Automatic optimization or repair
- Network services or telemetry
- Root-only inspection
- Package installation or service control
- Automatic game launching
>>>>>>> 2109919 (v0.2.0)
