# Xcode vs Linux Toolchain Compatibility Matrix

This document provides a feature-by-feature comparison between official Xcode tooling (on macOS) and experimental Linux alternative toolchains (such as `xcross`, `osxcross`, and `libimobiledevice`).

Every entry is classified under one of the five mandatory statuses:
- **`VERIFIED`**: Proven and reproducibly verified in a documented test environment.
- **`EXPERIMENTAL`**: Functional under strict constraints; subject to upstream changes or platform risks.
- **`UNSUPPORTED`**: Cannot execute on Linux without proprietary macOS frameworks.
- **`BLOCKED`**: Hard technical blockers (e.g., host-only binaries like macOS `gen_snapshot`).
- **`NOT_TESTED`**: Requires physical iOS hardware not currently connected or verified.

---

## Compatibility Matrix

| # | Xcode Function | Official macOS Status | Linux Equivalent / Approach | Status on Linux | Evidence & Limitations |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **1** | **Project Discovery** | `VERIFIED` | Inspect `ios/Runner.xcodeproj` & `pubspec.yaml` via Rust parser | `VERIFIED` | Project files, bundle ID, and deployment targets are standard text/plist files parsed natively. |
| **2** | **Build Configuration** | `VERIFIED` | Parse `ios/Flutter/Debug.xcconfig` and `Release.xcconfig` | `VERIFIED` | Key-value xcconfig files parsed natively on Linux. |
| **3** | **Dependency Resolution** | `VERIFIED` | `flutter pub get` + CocoaPods / SwiftPM inspection | `EXPERIMENTAL` | `flutter pub get` works; CocoaPods (`pod install`) requires Ruby CocoaPods + macOS environment for native pod hooks. |
| **4** | **Dart Compilation (Debug / JIT)** | `VERIFIED` | Dart VM kernel compiler (`frontend_server.dart.snapshot`) | `EXPERIMENTAL` | JIT compilation to kernel bytecode works cross-platform; used by xcross debug runner. |
| **5** | **Dart Compilation (Release / AOT)** | `VERIFIED` | Flutter `gen_snapshot` targeting iOS arm64 | `BLOCKED` | Google only ships `gen_snapshot` targeting Darwin arm64 for macOS hosts. Blocked on Linux. |
| **6** | **Native Compilation** | `VERIFIED` | Clang + Apple iOS SDK | `EXPERIMENTAL` | Requires cross-clang + extracted Apple SDK (osxcross). Cannot bundle SDKs legally. |
| **7** | **Linking** | `VERIFIED` | Apple `ld64` or LLVM `lld` with Mach-O arm64 target | `EXPERIMENTAL` | Supported by LLVM `lld` targeting `arm64-apple-ios` with proper sysroot. |
| **8** | **App Packaging** | `VERIFIED` | Directory layout (`Payload/AppName.app`) | `VERIFIED` | ZIP creation and directory structure verified natively in Rust. |
| **9** | **IPA Packaging** | `VERIFIED` | ZIP archive with `Payload/` folder | `VERIFIED` | Zip archive creation compliant with IPA layout; requires valid internal binaries. |
| **10** | **Code Signing (Ad-Hoc)** | `VERIFIED` | `ldid` or `rcodesign` | `EXPERIMENTAL` | Ad-hoc signatures can be produced using `rcodesign` or `ldid`, but rejected by non-jailbroken production devices. |
| **11** | **Code Signing (Developer / Distribution)** | `VERIFIED` | Apple `codesign` + Security framework | `EXPERIMENTAL` | `xcross` uses reverse-engineered ADI signing; raises security and Terms-of-Service risks. |
| **12** | **Provisioning Profiles** | `VERIFIED` | Apple Developer Portal / MobileProvision parser | `VERIFIED` | ASN.1 parser can read `.mobileprovision` profiles natively in Rust. |
| **13** | **Device Installation** | `VERIFIED` | `libimobiledevice` (`ideviceinstaller`) / xcross CoreDevice | `EXPERIMENTAL` | Works on USB-connected devices with paired trust record. |
| **14** | **Debugging / Hot Reload** | `VERIFIED` | Flutter VM service over WebSocket | `EXPERIMENTAL` | xcross implements iOS 17+ CoreDevice tunnel for hot reload. |
| **15** | **Archive / App Store Export** | `VERIFIED` | `xcodebuild -exportArchive` | `UNSUPPORTED` | App Store Connect distribution requires Apple validation notary service requiring macOS. |

---

## Conclusion & Design Directives

1. Linux desktop IDEs can legitimately inspect, parse, validate, and package mock or ad-hoc test bundles.
2. Production App Store release builds for iOS strictly require an Apple macOS host.
3. Ridu IDE will explicitly guide users to connect an authorized Mac worker or cloud macOS CI when store-ready distribution is selected.
