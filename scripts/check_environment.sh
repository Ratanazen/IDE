#!/usr/bin/env bash
# ==============================================================================
# Ratana Flutter Studio - Read-Only Environment Checker for Arch Linux
# This script does NOT modify system configuration and does NOT use sudo.
# ==============================================================================

set -u

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

MISSING_PKGS=()
MISSING_TOOLS=()
WARNINGS=()

print_header() {
    echo -e "${CYAN}${BOLD}=== Ratana Flutter Studio Environment Checker ===${NC}"
    echo -e "Date: $(date)"
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo -e "Host OS: ${BOLD}${PRETTY_NAME:-$NAME}${NC} (Kernel $(uname -r))"
    else
        echo -e "Host OS: Linux (Kernel $(uname -r))"
    fi
    echo ""
}

check_cmd() {
    local cmd_name="$1"
    local friendly_name="$2"
    local version_arg="${3:---version}"

    if command -v "$cmd_name" >/dev/null 2>&1; then
        local ver
        ver=$("$cmd_name" $version_arg 2>&1 | head -n 1)
        echo -e "  [${GREEN}✓${NC}] ${friendly_name}: ${ver}"
        return 0
    else
        echo -e "  [${RED}✗${NC}] ${friendly_name}: ${RED}Not found on PATH${NC}"
        MISSING_TOOLS+=("$cmd_name")
        return 1
    fi
}

check_pkg() {
    local pkg_name="$1"
    if pacman -Q "$pkg_name" >/dev/null 2>&1; then
        local ver
        ver=$(pacman -Q "$pkg_name" 2>/dev/null)
        echo -e "  [${GREEN}✓${NC}] Package ${pkg_name}: ${ver}"
        return 0
    else
        echo -e "  [${YELLOW}!${NC}] Package ${pkg_name}: ${YELLOW}Not installed via pacman${NC}"
        MISSING_PKGS+=("$pkg_name")
        return 1
    fi
}

check_pkg_config() {
    local lib_name="$1"
    if pkg-config --exists "$lib_name" >/dev/null 2>&1; then
        local ver
        ver=$(pkg-config --modversion "$lib_name" 2>/dev/null)
        echo -e "  [${GREEN}✓${NC}] Library ${lib_name}: ${ver}"
        return 0
    else
        echo -e "  [${RED}✗${NC}] Library ${lib_name}: ${RED}Missing or not in PKG_CONFIG_PATH${NC}"
        return 1
    fi
}

echo ""
print_header

echo -e "${BOLD}1. Checking Core Toolchains & Runtimes:${NC}"
check_cmd "rustc" "Rust Compiler"
check_cmd "cargo" "Cargo Package Manager"
check_cmd "node" "Node.js Runtime"
check_cmd "pnpm" "pnpm Package Manager"
check_cmd "flutter" "Flutter SDK"
check_cmd "dart" "Dart SDK"
check_cmd "java" "Java Runtime" "-version"

echo ""
echo -e "${BOLD}2. Checking Linux Desktop Build Tools:${NC}"
check_cmd "clang" "Clang C/C++ Compiler"
check_cmd "cmake" "CMake"
check_cmd "ninja" "Ninja"
check_cmd "pkg-config" "pkg-config"

echo ""
echo -e "${BOLD}3. Checking Tauri Desktop C-Libraries (pkg-config):${NC}"
check_pkg_config "webkit2gtk-4.1"
check_pkg_config "gtk+-3.0"
if ! pkg-config --exists "ayatana-appindicator3-0.1" >/dev/null 2>&1 && ! pkg-config --exists "libappindicator3-0.1" >/dev/null 2>&1; then
    echo -e "  [${YELLOW}!${NC}] AppIndicator library: ${YELLOW}Neither ayatana-appindicator3-0.1 nor libappindicator3-0.1 found${NC}"
    WARNINGS+=("AppIndicator library recommended: sudo pacman -S --needed libayatana-appindicator")
else
    echo -e "  [${GREEN}✓${NC}] AppIndicator library found"
fi

echo ""
echo -e "${BOLD}4. Checking Java & Android SDK Configuration:${NC}"
# JDK 17 check
if [ -d "/usr/lib/jvm/java-17-openjdk" ]; then
    echo -e "  [${GREEN}✓${NC}] JDK 17 found at /usr/lib/jvm/java-17-openjdk (Required for AGP 8.x)"
else
    echo -e "  [${YELLOW}!${NC}] JDK 17 not found at /usr/lib/jvm/java-17-openjdk"
    WARNINGS+=("Install JDK 17: sudo pacman -S --needed jdk17-openjdk")
fi

# Android SDK check
ANDROID_SDK_PATH="${ANDROID_HOME:-$HOME/Android/Sdk}"
if [ -d "$ANDROID_SDK_PATH" ]; then
    echo -e "  [${GREEN}✓${NC}] Android SDK directory found at: $ANDROID_SDK_PATH"
    
    # Check platforms
    if [ -d "$ANDROID_SDK_PATH/platforms" ] && [ "$(ls -A "$ANDROID_SDK_PATH/platforms" 2>/dev/null)" ]; then
        echo -e "  [${GREEN}✓${NC}] Android platforms: $(ls "$ANDROID_SDK_PATH/platforms" | tr '\n' ' ')"
    else
        echo -e "  [${YELLOW}!${NC}] No Android platforms found in $ANDROID_SDK_PATH/platforms"
        WARNINGS+=("Android platform missing. Install via sdkmanager or AUR.")
    fi
    
    # Check build-tools
    if [ -d "$ANDROID_SDK_PATH/build-tools" ] && [ "$(ls -A "$ANDROID_SDK_PATH/build-tools" 2>/dev/null)" ]; then
        echo -e "  [${GREEN}✓${NC}] Android build-tools: $(ls "$ANDROID_SDK_PATH/build-tools" | tr '\n' ' ')"
    else
        echo -e "  [${YELLOW}!${NC}] No Android build-tools found in $ANDROID_SDK_PATH/build-tools"
        WARNINGS+=("Android build-tools missing.")
    fi

    # Check cmdline-tools
    if [ -d "$ANDROID_SDK_PATH/cmdline-tools" ] && [ "$(ls -A "$ANDROID_SDK_PATH/cmdline-tools" 2>/dev/null)" ]; then
        echo -e "  [${GREEN}✓${NC}] Android cmdline-tools found"
    else
        echo -e "  [${YELLOW}!${NC}] Android cmdline-tools component is missing"
        WARNINGS+=("Install cmdline-tools: yay -S android-sdk-cmdline-tools-latest OR download from Google and place in $ANDROID_SDK_PATH/cmdline-tools/latest")
    fi
else
    echo -e "  [${YELLOW}!${NC}] Android SDK directory not found at $ANDROID_SDK_PATH"
    WARNINGS+=("Android SDK not installed. Required for APK and AAB builds.")
fi

echo ""
echo -e "${BOLD}=== Summary & Recommendations ===${NC}"
if [ ${#MISSING_TOOLS[@]} -eq 0 ] && [ ${#WARNINGS[@]} -eq 0 ]; then
    echo -e "${GREEN}${BOLD}All essential toolchains and dependencies are detected and ready!${NC}"
    exit 0
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo -e "${RED}Missing required executable tools:${NC}"
    for tool in "${MISSING_TOOLS[@]}"; do
        echo -e "  - ${RED}${tool}${NC}"
    done
fi

if [ ${#WARNINGS[@]} -gt 0 ]; then
    echo -e "${YELLOW}Recommendations and optional steps:${NC}"
    for warn in "${WARNINGS[@]}"; do
        echo -e "  - ${warn}"
    done
fi

echo ""
echo -e "Refer to ${BOLD}docs/INSTALL_ARCH.md${NC} for full step-by-step setup instructions."
exit 0
