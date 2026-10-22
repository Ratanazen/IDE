# iOS Toolchain Research & Feasibility on Linux

**Author**: Senior Systems Architect & Linux Desktop Developer  
**Status**: EVIDENCE-DRIVEN RESEARCH REPORT  
**Date**: 2026-10-22  

---

## 1. Executive Summary

Developing and building iOS applications on Linux without proprietary macOS hardware is subject to strict technical and legal constraints imposed by Apple's software architecture, proprietary SDK licenses, code-signing infrastructure, and Mach-O AOT compilation toolchains.

This report documents reproducible findings regarding the five primary tools in this space:
1. **`arxdeus/xcross`**
2. **Flutter iOS Build Pipeline (`flutter build ios`)**
3. **`libimobiledevice/libimobiledevice`**
4. **`ios-control/ios-deploy`**
5. **`tpoechtrager/osxcross`**

---

## 2. Tool Evaluation & Status Matrix

### 2.1 `arxdeus/xcross`
- **Repository**: [https://github.com/arxdeus/xcross](https://github.com/arxdeus/xcross)
- **Documentation**: [https://xcross.sh/docs/introduction/](https://xcross.sh/docs/introduction/)
- **License**: AGPL-3.0 (with parts under MIT/Apache-2.0).
- **Maintenance**: Active development targeting iOS 17+.
- **Capabilities**:
  - Implements an alternative build, sign, install, and hot-reload pipeline for Flutter on Linux.
  - Communicates directly with iOS 17+ devices using Apple's CoreDevice protocol implemented in Dart.
- **Limitations & Terms-of-Service Risks**:
  - **Gen_snapshot Constraint**: Flutter's official iOS AOT compiler (`gen_snapshot` targeting Darwin arm64) is provided by Google only as a macOS host executable. On Linux x86_64 hosts, generating release AOT Mach-O binaries for iOS cannot execute natively without a Darwin emulation/translation layer.
  - **Signing Risk**: xcross signing relies on an Apple Developer login and downloads proprietary ADI (Apple Device Identification) helper binaries during initial setup. This poses severe Terms-of-Service and licensing compliance issues.
  - **Launch Requirement**: Launch and debug capabilities require a physical iOS 17+ device connected over USB/network. No iOS simulator can execute on Linux.
- **Assigned Status**: `EXPERIMENTAL`

### 2.2 Flutter Official iOS Pipeline
- **Official Documentation**: [https://docs.flutter.dev/deployment/ios](https://docs.flutter.dev/deployment/ios)
- **Host Requirement**: macOS with Xcode installed.
- **Linux Status**: On Linux, invoking `flutter build ios` immediately aborts:
  ```
  Building for iOS is only supported on macOS.
  ```
- **Assigned Status**: `UNSUPPORTED_ON_LINUX`

### 2.3 `libimobiledevice`
- **Repository**: [https://github.com/libimobiledevice/libimobiledevice](https://github.com/libimobiledevice/libimobiledevice)
- **License**: LGPL-2.1
- **Status**: Stable, widely packaged on Arch Linux (`pacman -S libimobiledevice`).
- **Capabilities**: Communicates with iOS devices over USB (lockdownd, afc, syslog, crash reports).
- **Limitations**: Cannot compile source code or generate Xcode archives.
- **Assigned Status**: `VERIFIED` (Device Communication Only)

### 2.4 `ios-deploy`
- **Repository**: [https://github.com/ios-control/ios-deploy](https://github.com/ios-control/ios-deploy)
- **Host Requirement**: macOS only (relies on private `MobileDevice.framework`).
- **Linux Status**: Cannot build or run on Linux.
- **Assigned Status**: `UNSUPPORTED_ON_LINUX`

### 2.5 `osxcross`
- **Repository**: [https://github.com/tpoechtrager/osxcross](https://github.com/tpoechtrager/osxcross)
- **License**: GPL-2.0
- **Status**: Clang-based cross-compilation toolchain for macOS/Darwin binaries from Linux.
- **Prerequisites**: Requires extracting a proprietary macOS SDK from an authorized Xcode image.
- **Redistribution**: Apple SDKs cannot be legally bundled or redistributed.
- **Assigned Status**: `EXPERIMENTAL` (requires user-provided SDK)

---

## 3. Honesty & Ethical Guardrails

1. **No Fake IPA Generation**: A generic ZIP file with a `.ipa` extension or a synthetic `Payload/` directory will **never** be presented as a valid iOS application.
2. **No Proprietary SDK Bundling**: Ridu IDE / Ratana Flutter Studio will **never** bundle Apple SDKs, Apple certificates, private keys, or provisioning profiles.
3. **Explicit Capability Status**: Every stage of the iOS build pipeline reports its real status: `VERIFIED`, `EXPERIMENTAL`, `UNSUPPORTED`, `BLOCKED`, or `NOT_TESTED`.
