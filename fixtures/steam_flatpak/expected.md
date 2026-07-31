# GameDoc Report

**GameDoc version:** 0.2.0  
**Schema:** `gamedoc.v2`

## Facts

| Category | Fact | Value | Confidence |
| --- | --- | --- | --- |
| System | `system.architecture` | x86_64 | detected |
| System | `system.kernel_version` | 6.9.0-test | detected |
| CPU | `cpu.architecture` | x86_64 | detected |
| CPU | `cpu.cores_logical` | 0 | detected |
| GPU | `gpu.vulkan_available` | true | detected |
| Steam | `steam.data_readable` | {"installation":"flatpak","readable":true} | detected |
| Steam | `steam.installation` | flatpak | detected |
| Steam | `steam.library` | {"index":1,"installation":"flatpak"} | detected |
| Steam | `steam.library_readable` | {"index":1,"installation":"flatpak","readable":true} | detected |
| Steam | `steam.proton_scan_complete` | {"complete":true,"installation":"flatpak"} | detected |
| Steam | `steam.proton_version` | {"installation":"flatpak","name":"Proton Experimental"} | detected |

## Problems

_No problems detected from available evidence._

## Incomplete Observations

- **system** (`read /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor`): failed to read /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor: entity not found
- **cpu** (`read /proc/cpuinfo`): failed to read /proc/cpuinfo: entity not found
- **distro** (`read /etc/os-release`): failed to read /etc/os-release: entity not found
