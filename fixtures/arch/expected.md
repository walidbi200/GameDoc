# GameDoc Report

**GameDoc version:** 0.2.0  
**Schema:** `gamedoc.v2`

## Facts

| Category | Fact | Value | Confidence |
| --- | --- | --- | --- |
| System | `system.architecture` | x86_64 | detected |
| System | `system.cpu_governor` | performance | detected |
| System | `system.desktop` | GNOME | detected |
| System | `system.kernel_version` | 6.9.0-test | detected |
| System | `system.session_type` | wayland | detected |
| CPU | `cpu.architecture` | x86_64 | detected |
| CPU | `cpu.cores_logical` | 0 | detected |
| GPU | `gpu.device` | 00:02.0 "VGA compatible controller" "Example GPU" "Model 1" | detected |
| GPU | `gpu.opengl_renderer` | Example Renderer | detected |
| GPU | `gpu.opengl_version` | 4.6 Mesa 24.1 | detected |
| GPU | `gpu.vulkan_available` | true | detected |

## Problems

_No problems detected from available evidence._

## Incomplete Observations

- **cpu** (`read /proc/cpuinfo`): failed to read /proc/cpuinfo: entity not found
- **distro** (`read /etc/os-release`): failed to read /etc/os-release: entity not found
