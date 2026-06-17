# Molestus

Molestus is a playful and highly annoying desktop pet written in Rust. Using Rapier2D physics, a transparent click-through interface via Slint, and computer vision / AI analysis, Molestus dynamically calculates where you are focusing on the screen and bounces its soft-body blob self directly into your line of sight to annoy you.

When the blob gets close to its target, it triggers a "splat" effect, clinging momentarily to the area before bouncing away again!

## Features
- **Soft-Body Physics**: Implemented with Rapier2D using an Extended Position-Based Dynamics (XBPD) style spring-node simulation.
- **Click-Through Interface**: Bypasses input so you can click right through the blob without disrupting your workflow (it just visually blocks you).
- **AI/Vision Target Tracking**: Analyzes the screen (or uses Qwen3-VL heuristics) to determine the most "interesting" or active area of your screen and attacks it.

## Installation

Molestus uses a convenient PowerShell script to install. This script will download the pre-compiled binary and the necessary AI model weights to your system.

To install Molestus, open PowerShell as an Administrator (or normal user) and run the following command:

```powershell
irm https://raw.githubusercontent.com/turtle170/Molestus/main/install.ps1 | iex
```

### What the script does:
1. Creates the application directory at `C:\ProgramData\Molestus`.
2. Creates the model weights directory at `D:\Molestus\models`.
3. Downloads the latest `Molestus.exe` release from GitHub.
4. Downloads the `Qwen3-VL-2B-Instruct` AI model files from HuggingFace to your `D:` drive.
5. Places a handy shortcut on your Desktop!

## Building from Source

If you wish to compile Molestus yourself:

1. Ensure you have the latest stable Rust toolchain installed.
2. Clone this repository:
   ```bash
   git clone https://github.com/turtle170/Molestus.git
   cd Molestus
   ```
3. Run the build command:
   ```bash
   cargo build --release
   ```
   *Note: This will build the binary but you will still need to manually place the `Qwen3` GGUF model files in `D:\Molestus\models` for the AI functionality to work properly.*

## License

This project is licensed under the Apache 2.0 License. See the [LICENSE](LICENSE) file for details.
