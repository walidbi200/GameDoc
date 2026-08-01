# GameDoc Report

**GameDoc version:** 0.3.0  
**Schema:** `gamedoc.v2`

## Facts

| Category | Fact | Value | Confidence |
| --- | --- | --- | --- |
| System | `system.architecture` | x86_64 | detected |
| System | `system.cpu_governor` | performance | detected |
| System | `system.kernel_version` | 6.9.0-test | detected |
| System | `system.session_type` | wayland | detected |
| CPU | `cpu.architecture` | x86_64 | detected |
| CPU | `cpu.cores_logical` | 0 | detected |
| GPU | `gpu.vulkan_available` | true | detected |
| Steam | `steam.compatibility_tool` | {"installation":"native","name":"GE-Proton10-1"} | detected |
| Steam | `steam.data_readable` | {"installation":"native","readable":true} | detected |
| Steam | `steam.executable` | true | detected |
| Steam | `steam.installation` | native | detected |
| Steam | `steam.library` | {"index":1,"installation":"native"} | detected |
| Steam | `steam.library` | {"index":2,"installation":"native"} | detected |
| Steam | `steam.library_readable` | {"index":1,"installation":"native","readable":true} | detected |
| Steam | `steam.library_readable` | {"index":2,"installation":"native","readable":true} | detected |
| Steam | `steam.proton_scan_complete` | {"complete":true,"installation":"native"} | detected |
| Steam | `steam.proton_scan_complete` | {"complete":true,"installation":"native"} | detected |
| Steam | `steam.proton_version` | {"installation":"native","name":"Proton Experimental"} | detected |
| Steam | `steam.proton_version` | {"installation":"native","name":"Proton Hotfix"} | detected |
| GameMode | `gamemode.daemon_available` | true | detected |
| GameMode | `gamemode.executable` | true | detected |
| GameMode | `gamemode.library` | 64-bit | detected |

## Problems

_No problems detected from available evidence._

## Incomplete Observations

- **cpu** (`read /proc/cpuinfo`): failed to read /proc/cpuinfo: entity not found
- **distro** (`read /etc/os-release`): failed to read /etc/os-release: entity not found
