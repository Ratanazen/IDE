import React from 'react';
import { CheckCircle2, XCircle, AlertCircle, RefreshCw } from 'lucide-react';
import { ToolchainReport } from '../types';

interface ToolchainViewProps {
  report: ToolchainReport | null;
  onRefresh: () => void;
  loading: boolean;
}

export const ToolchainView: React.FC<ToolchainViewProps> = ({ report, onRefresh, loading }) => {
  if (loading || !report) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex items-center gap-3 text-slate-400 text-xs">
          <RefreshCw className="w-5 h-5 animate-spin text-sky-400" />
          <span>Scanning host toolchains and runtimes...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col p-6 overflow-y-auto space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-100">Toolchain & Runtimes</h2>
          <p className="text-xs text-slate-400">Real-time status of compilers, SDKs, and platform libraries</p>
        </div>
        <button
          onClick={onRefresh}
          className="flex items-center gap-2 px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-lg border border-slate-700 transition-all cursor-pointer"
        >
          <RefreshCw className="w-3.5 h-3.5 text-sky-400" />
          <span>Refresh Detectors</span>
        </button>
      </div>

      {/* Target Readiness Matrix */}
      <div className="grid grid-cols-4 gap-4">
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-semibold text-slate-300">Web Target</span>
            {report.ready_for_web ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            ) : (
              <XCircle className="w-4 h-4 text-rose-400" />
            )}
          </div>
          <span className={`text-[11px] px-2 py-0.5 rounded font-mono font-medium ${
            report.ready_for_web ? 'bg-emerald-500/10 text-emerald-400' : 'bg-rose-500/10 text-rose-400'
          }`}>
            {report.ready_for_web ? 'READY' : 'UNAVAILABLE'}
          </span>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-semibold text-slate-300">Linux Desktop</span>
            {report.ready_for_linux ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            ) : (
              <XCircle className="w-4 h-4 text-rose-400" />
            )}
          </div>
          <span className={`text-[11px] px-2 py-0.5 rounded font-mono font-medium ${
            report.ready_for_linux ? 'bg-emerald-500/10 text-emerald-400' : 'bg-rose-500/10 text-rose-400'
          }`}>
            {report.ready_for_linux ? 'READY' : 'MISSING DEPS'}
          </span>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-semibold text-slate-300">Android APK/AAB</span>
            {report.ready_for_android ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            ) : (
              <AlertCircle className="w-4 h-4 text-amber-400" />
            )}
          </div>
          <span className={`text-[11px] px-2 py-0.5 rounded font-mono font-medium ${
            report.ready_for_android ? 'bg-emerald-500/10 text-emerald-400' : 'bg-amber-500/10 text-amber-400'
          }`}>
            {report.ready_for_android ? 'READY' : 'ACTION NEEDED'}
          </span>
        </div>

        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-semibold text-slate-300">iOS Target</span>
            <AlertCircle className="w-4 h-4 text-sky-400" />
          </div>
          <span className="text-[11px] px-2 py-0.5 rounded font-mono font-medium bg-sky-500/10 text-sky-400">
            EXPERIMENTAL (Linux)
          </span>
        </div>
      </div>

      {/* Diagnostics / Action Required */}
      {report.diagnostics.length > 0 && (
        <div className="bg-amber-500/10 border border-amber-500/30 rounded-xl p-4">
          <h4 className="text-xs font-semibold text-amber-400 flex items-center gap-2 mb-2">
            <AlertCircle className="w-4 h-4" />
            <span>Diagnostics & Required Actions:</span>
          </h4>
          <ul className="list-disc list-inside space-y-1 text-xs text-slate-300">
            {report.diagnostics.map((diag, idx) => (
              <li key={idx}>{diag}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Detailed Tool List */}
      <div className="grid grid-cols-2 gap-4">
        {/* Core SDKs */}
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3">
          <h3 className="text-xs font-semibold text-slate-300 uppercase tracking-wider mb-2">
            Flutter & Dart Runtimes
          </h3>

          <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800/80">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-200">Flutter SDK</span>
              <span className="text-xs text-emerald-400 font-mono">
                {report.flutter.available ? 'Installed' : 'Missing'}
              </span>
            </div>
            <p className="text-[11px] text-slate-400 mt-1 font-mono">{report.flutter.version || 'N/A'}</p>
          </div>

          <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800/80">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-200">Dart SDK</span>
              <span className="text-xs text-emerald-400 font-mono">
                {report.dart.available ? 'Installed' : 'Missing'}
              </span>
            </div>
            <p className="text-[11px] text-slate-400 mt-1 font-mono">{report.dart.version || 'N/A'}</p>
          </div>
        </div>

        {/* Android SDK & JDK */}
        <div className="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3">
          <h3 className="text-xs font-semibold text-slate-300 uppercase tracking-wider mb-2">
            Android & Java Toolchains
          </h3>

          <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800/80">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-200">Android SDK</span>
              <span className="text-xs text-emerald-400 font-mono">
                {report.android ? 'Detected' : 'Missing'}
              </span>
            </div>
            {report.android && (
              <div className="text-[11px] text-slate-400 mt-1 space-y-0.5">
                <div>Path: <span className="font-mono text-slate-300">{report.android.path}</span></div>
                <div>Platforms: <span className="font-mono text-slate-300">{report.android.platforms.join(', ')}</span></div>
                <div>Build-Tools: <span className="font-mono text-slate-300">{report.android.build_tools.join(', ')}</span></div>
                <div>cmdline-tools: <span className={report.android.has_cmdline_tools ? 'text-emerald-400 font-medium' : 'text-amber-400 font-medium'}>
                  {report.android.has_cmdline_tools ? 'Present' : 'Missing'}
                </span></div>
              </div>
            )}
          </div>

          <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800/80">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-200">Installed JDKs</span>
              <span className="text-xs text-emerald-400 font-mono">{report.jdks.length} found</span>
            </div>
            <div className="text-[11px] text-slate-400 mt-1 space-y-1">
              {report.jdks.map((jdk) => (
                <div key={jdk.path} className="flex justify-between items-center">
                  <span className="font-mono text-slate-300">{jdk.version}</span>
                  {jdk.is_agp_compatible && (
                    <span className="text-[10px] text-sky-400 bg-sky-500/10 px-1.5 py-0.5 rounded">
                      AGP 8.x Compatible
                    </span>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
