# GameDoc Report

**GameDoc version:** 0.3.0  
**Schema:** `gamedoc.v2`

## Facts

| Category | Fact | Value | Confidence |
| --- | --- | --- | --- |
| System | `system.architecture` | x86_64 | detected |
| System | `system.kernel_version` | 6.9.0-test | detected |
| CPU | `cpu.architecture` | x86_64 | detected |
| CPU | `cpu.cores_logical` | 0 | detected |
| GPU | `gpu.vulkan_available` | false | detected |
| Steam | `steam.data_readable` | {"installation":"native","readable":true} | detected |
| Steam | `steam.executable` | true | detected |
| Steam | `steam.installation` | native | detected |
| Steam | `steam.library` | {"index":1,"installation":"native"} | detected |
| Steam | `steam.library` | {"index":2,"installation":"native"} | detected |
| Steam | `steam.library_readable` | {"index":1,"installation":"native","readable":true} | detected |
| Steam | `steam.library_readable` | {"index":2,"installation":"native","readable":false} | detected |
| Steam | `steam.proton_scan_complete` | {"complete":false,"installation":"native"} | detected |

## Problems

### critical: Vulkan is not available

- **ID:** `gpu.vulkan_unavailable`
- **Category:** GPU
- The local Vulkan probe exited unsuccessfully.

**Recommendation:** GameDoc could not verify Vulkan support because vulkaninfo was not installed or did not report a working Vulkan device.

### medium: A Steam library could not be read

- **ID:** `steam.library_unreadable`
- **Category:** Steam
- 1 Steam library location(s) could not be read.

**Recommendation:** GameDoc could not inspect this library, so installed games and compatibility tools may be missing from the report. Check the mount and grant your user read access.


## Incomplete Observations

- **system** (`read /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor`): failed to read /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor: entity not found
- **cpu** (`read /proc/cpuinfo`): failed to read /proc/cpuinfo: entity not found
- **distro** (`read /etc/os-release`): failed to read /etc/os-release: entity not found
- **graphics** (`vulkaninfo --summary`): command exited with status Some(1)
- **steam** (`native Steam common directory`): Steam common directory could not be read
- **steam** (`native Steam library 2`): Steam library could not be read
