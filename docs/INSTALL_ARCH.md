# Arch Linux Environment Setup Guide

This guide details the complete prerequisites and installation steps for running and developing **Ratana Flutter Studio** on Arch Linux, Archcraft, and related distributions (e.g., Garuda Linux, Manjaro, EndeavourOS).

---

## 1. System Upgrade & Base Packages

> [!WARNING]
> Always perform a full system upgrade before installing packages on Arch Linux. Never perform partial upgrades (`pacman -Sy <pkg>`).

```bash
sudo pacman -Syu
```

Install the required core development packages, desktop build tools, and Tauri dependencies:

```bash
sudo pacman -S --needed \
  base-devel git curl wget unzip zip file openssl pkgconf \
  nodejs npm pnpm \
  webkit2gtk-4.1 librsvg libappindicator-gtk3 xdotool \
  clang cmake ninja gtk3 \
  jdk17-openjdk
```

### Package Renames and Distro Notes:
- **Rust Toolchain**: On Arch Linux, Rust can be installed via `rustup` from official repositories (`sudo pacman -S rustup && rustup default stable`) or via the official standalone script from `https://rustup.rs` (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`). If installed via standalone rustup, `pacman -Q rustup` will not report it, but `rustc` and `cargo` will reside in `~/.cargo/bin`.
- **AppIndicator**: Arch Linux provides `libappindicator-gtk3` (or `libayatana-appindicator`). Tauri 2 links against `webkit2gtk-4.1` and `ayatana-appindicator3-0.1`.
- **JDK Compatibility**: Android Gradle Plugin (AGP) 8.x requires Java 17. The system may have multiple JDKs installed under `/usr/lib/jvm/`. You can switch the active system Java or set `JAVA_HOME` explicitly:
  ```bash
  sudo archlinux-java set java-17-openjdk
  # Or export per build session:
  export JAVA_HOME=/usr/lib/jvm/java-17-openjdk
  ```

---

## 2. Flutter SDK Installation

Per the official Linux instructions, install the stable Flutter channel:

```bash
# Clone Flutter SDK to a user-writable directory (e.g., ~/.cache/flutter_sdk or ~/development/flutter)
git clone https://github.com/flutter/flutter.git -b stable ~/.cache/flutter_sdk

# Add to PATH in ~/.bashrc or ~/.zshrc:
export PATH="$PATH:$HOME/.cache/flutter_sdk/bin"
```

Verify installation:
```bash
flutter --version
flutter doctor -v
```

---

## 3. Android SDK & Command-Line Tools

Android command-line tools can be installed via AUR or directly downloaded from Google:

### Option A: Via AUR (Recommended on Arch)
```bash
yay -S android-sdk-cmdline-tools-latest
```

### Option B: Manual Download from Google
1. Download commandline-tools from Google Developer:
   `https://developer.android.com/studio#command-line-tools-only`
2. Extract into `$ANDROID_HOME/cmdline-tools/latest/`:
   ```bash
   mkdir -p ~/Android/Sdk/cmdline-tools/latest
   unzip commandlinetools-linux-*_latest.zip -d /tmp/cmdline-tools
   mv /tmp/cmdline-tools/cmdline-tools/* ~/Android/Sdk/cmdline-tools/latest/
   ```
3. Set environment variables in your shell profile:
   ```bash
   export ANDROID_HOME="$HOME/Android/Sdk"
   export PATH="$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools"
   ```

### Android License Acceptance (Interactive)
> [!IMPORTANT]
> The Android SDK licenses require personal review and acceptance. Never auto-accept licenses without user consent.
> Run in an interactive terminal:
```bash
flutter doctor --android-licenses
```

---

## 4. Environment Verification Script

A non-modifying, read-only verification script is provided in the repository at:
`scripts/check_environment.sh`

Run it at any time to verify system dependencies without altering system state:
```bash
./scripts/check_environment.sh
```
