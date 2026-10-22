use regex::Regex;
use ridu_core::models::{CapabilityStatus, Diagnostic, DiagnosticSeverity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LanguageError {
    #[error("LSP server '{0}' not found on host")]
    ServerNotFound(String),

    #[error("Failed to parse diagnostic output: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerInfo {
    pub id: String,
    pub name: String,
    pub path: Option<PathBuf>,
    pub supported_languages: Vec<String>,
    pub status: CapabilityStatus,
    pub command_args: Vec<String>,
}

pub struct LspRegistry;

impl LspRegistry {
    /// Detects all supported Language Server Protocol binaries on the host system.
    pub fn detect_all() -> Vec<LspServerInfo> {
        vec![
            Self::detect_clangd(),
            Self::detect_sourcekit_lsp(),
            Self::detect_dart_lsp(),
        ]
    }

    pub fn detect_clangd() -> LspServerInfo {
        let path = which::which("clangd").ok();
        let status = if path.is_some() {
            CapabilityStatus::Verified
        } else {
            CapabilityStatus::MissingDependency
        };

        LspServerInfo {
            id: "clangd".to_string(),
            name: "clangd (C/C++/Objective-C Language Server)".to_string(),
            path,
            supported_languages: vec![
                "c".to_string(),
                "cpp".to_string(),
                "objective-c".to_string(),
                "objective-cpp".to_string(),
            ],
            status,
            command_args: vec!["--background-index".to_string(), "--clang-tidy".to_string()],
        }
    }

    pub fn detect_sourcekit_lsp() -> LspServerInfo {
        let path = which::which("sourcekit-lsp").ok();
        let status = if path.is_some() {
            CapabilityStatus::Verified
        } else {
            CapabilityStatus::MissingDependency
        };

        LspServerInfo {
            id: "sourcekit-lsp".to_string(),
            name: "SourceKit-LSP (Swift & Apple C-family)".to_string(),
            path,
            supported_languages: vec![
                "swift".to_string(),
                "objective-c".to_string(),
                "c".to_string(),
                "cpp".to_string(),
            ],
            status,
            command_args: Vec::new(),
        }
    }

    pub fn detect_dart_lsp() -> LspServerInfo {
        let dart = which::which("dart").ok();
        let status = if dart.is_some() {
            CapabilityStatus::Verified
        } else {
            CapabilityStatus::MissingDependency
        };

        LspServerInfo {
            id: "dart-lsp".to_string(),
            name: "Dart Analysis Server (LSP Mode)".to_string(),
            path: dart,
            supported_languages: vec!["dart".to_string()],
            status,
            command_args: vec!["language-server".to_string(), "--protocol=lsp".to_string()],
        }
    }

    pub fn find_for_language(lang: &str) -> Option<LspServerInfo> {
        Self::detect_all()
            .into_iter()
            .find(|s| s.status == CapabilityStatus::Verified && s.supported_languages.iter().any(|l| l == lang))
    }
}

pub struct DiagnosticsParser;

impl DiagnosticsParser {
    /// Parses compiler and build tool stderr/stdout into structured Diagnostics.
    /// Handles standard Clang, Swiftc, and GCC outputs:
    /// `<file>:<line>:<col>: <severity>: <message>`
    pub fn parse(output: &str, default_source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Pattern for LLVM/Clang/Swift:
        // /path/to/File.swift:42:10: error: cannot find 'Foo' in scope
        // AppDelegate.m:15:5: warning: incompatible pointer types
        let clang_regex = Regex::new(
            r"(?m)^(?P<file>[^:\n]+):(?P<line>\d+):(?P<col>\d+):\s*(?P<severity>error|fatal error|warning|note|remark):\s*(?P<msg>.+)$"
        ).unwrap();

        // Pattern for Dart analyzer:
        //   error • Undefined name 'x' • lib/main.dart:10:5 • undefined_identifier
        let dart_regex = Regex::new(
            r"(?m)^\s*(?P<severity>error|warning|info)\s*[•-]\s*(?P<msg>.+?)\s*[•-]\s*(?P<file>[^:\n]+):(?P<line>\d+):(?P<col>\d+)(?:\s*[•-]\s*(?P<code>\w+))?"
        ).unwrap();

        for cap in clang_regex.captures_iter(output) {
            let file_str = cap.name("file").map(|m| m.as_str()).unwrap_or("");
            let line: usize = cap.name("line").and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
            let col: usize = cap.name("col").and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
            let sev_str = cap.name("severity").map(|m| m.as_str()).unwrap_or("error");
            let msg = cap.name("msg").map(|m| m.as_str().trim().to_string()).unwrap_or_default();

            let severity = match sev_str {
                "error" | "fatal error" => DiagnosticSeverity::Error,
                "warning" => DiagnosticSeverity::Warning,
                "note" => DiagnosticSeverity::Information,
                "remark" => DiagnosticSeverity::Hint,
                _ => DiagnosticSeverity::Error,
            };

            let id = format!("{}:{}:{}:{}", file_str, line, col, diagnostics.len());

            diagnostics.push(Diagnostic {
                id,
                file_path: PathBuf::from(file_str),
                line,
                column: col,
                end_line: None,
                end_column: None,
                severity,
                message: msg,
                source: default_source.to_string(),
                code: None,
                snippet: None,
            });
        }

        for cap in dart_regex.captures_iter(output) {
            let file_str = cap.name("file").map(|m| m.as_str().trim()).unwrap_or("");
            let line: usize = cap.name("line").and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
            let col: usize = cap.name("col").and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
            let sev_str = cap.name("severity").map(|m| m.as_str().trim()).unwrap_or("error");
            let msg = cap.name("msg").map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            let code = cap.name("code").map(|m| m.as_str().trim().to_string());

            let severity = match sev_str {
                "error" => DiagnosticSeverity::Error,
                "warning" => DiagnosticSeverity::Warning,
                _ => DiagnosticSeverity::Information,
            };

            let id = format!("{}:{}:{}:{}", file_str, line, col, diagnostics.len());

            diagnostics.push(Diagnostic {
                id,
                file_path: PathBuf::from(file_str),
                line,
                column: col,
                end_line: None,
                end_column: None,
                severity,
                message: msg,
                source: "dart-analyzer".to_string(),
                code,
                snippet: None,
            });
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clang_and_swift_diagnostics() {
        let sample_output = r#"
Sources/App/Main.swift:42:15: error: cannot find 'ContentView' in scope
/home/user/project/AppDelegate.m:88:5: warning: incompatible pointer types assigning to 'NSString *' from 'NSNumber *'
main.c:10:2: note: did you mean 'printf'?
"#;

        let diags = DiagnosticsParser::parse(sample_output, "clang");
        assert_eq!(diags.len(), 3);

        assert_eq!(diags[0].file_path, PathBuf::from("Sources/App/Main.swift"));
        assert_eq!(diags[0].line, 42);
        assert_eq!(diags[0].column, 15);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
        assert!(diags[0].message.contains("cannot find 'ContentView'"));

        assert_eq!(diags[1].line, 88);
        assert_eq!(diags[1].severity, DiagnosticSeverity::Warning);

        assert_eq!(diags[2].line, 10);
        assert_eq!(diags[2].severity, DiagnosticSeverity::Information);
    }

    #[test]
    fn test_parse_dart_diagnostics() {
        let sample_output = r#"
  info • The declaration '_counter' isn't used • lib/main.dart:25:7 • unused_field
  error • Undefined class 'CupertinoTheme' • lib/main.dart:50:12 • undefined_class
"#;

        let diags = DiagnosticsParser::parse(sample_output, "flutter");
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Information);
        assert_eq!(diags[0].code.as_deref(), Some("unused_field"));
        assert_eq!(diags[1].severity, DiagnosticSeverity::Error);
        assert_eq!(diags[1].line, 50);
    }
}
