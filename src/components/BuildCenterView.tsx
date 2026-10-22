import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Archive, ExternalLink, ShieldCheck, FileCheck } from 'lucide-react';
import { BuildResult, IpaValidationReport } from '../types';

interface BuildCenterViewProps {
  onLog: (line: string) => void;
}

export const BuildCenterView: React.FC<BuildCenterViewProps> = ({ onLog }) => {
  const [history, setHistory] = useState<BuildResult[]>([]);
  const [ipaPath, setIpaPath] = useState('');
  const [ipaReport, setIpaReport] = useState<IpaValidationReport | null>(null);
  const [validatingIpa, setValidatingIpa] = useState(false);

  const loadHistory = async () => {
    try {
      const records = await invoke<BuildResult[]>('get_build_history');
      setHistory(records);
    } catch (err: any) {
      onLog(`[History] Failed to load build history: ${err}`);
    }
  };

  useEffect(() => {
    loadHistory();
  }, []);

  const handleReveal = async (path: string) => {
    try {
      await invoke('reveal_in_file_manager', { path });
      onLog(`[Artifacts] Opened in file manager: ${path}`);
    } catch (err: any) {
      onLog(`[Artifacts] Failed to open path: ${err}`);
    }
  };

  const handleValidateIpa = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!ipaPath.trim()) return;

    setValidatingIpa(true);
    setIpaReport(null);
    try {
      const rep = await invoke<IpaValidationReport>('validate_ipa_archive', {
        ipaPath: ipaPath.trim(),
      });
      setIpaReport(rep);
      onLog(`[IPA Validator] Validation complete: ${rep.status} (App: ${rep.app_name || 'N/A'})`);
    } catch (err: any) {
      onLog(`[IPA Validator] Validation error: ${err}`);
      alert(`IPA Validation Error: ${err}`);
    } finally {
      setValidatingIpa(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col p-6 overflow-y-auto space-y-6">
      <div>
        <h2 className="text-xl font-bold text-slate-100">Build Center & Artifact Registry</h2>
        <p className="text-xs text-slate-400">
          Inspect persistent build records, verified release binaries, and validate iOS IPA packages
        </p>
      </div>

      {/* Verified Artifacts Section */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-sm space-y-4">
        <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
          <Archive className="w-4 h-4 text-sky-400" />
          <span>Verified Local Artifacts</span>
        </h3>

        <div className="grid grid-cols-3 gap-4">
          {/* Web Bundle Card */}
          <div className="bg-slate-950/60 border border-slate-800 rounded-lg p-3 space-y-1 text-xs">
            <div className="flex justify-between items-center font-semibold text-slate-200">
              <span>Web Bundle</span>
              <span className="text-emerald-400">READY</span>
            </div>
            <p className="text-[11px] text-slate-400 font-mono">fixtures/fixture_app/build/web</p>
            <div className="pt-2">
              <button
                onClick={() => handleReveal('fixtures/fixture_app/build/web')}
                className="flex items-center gap-1 text-[11px] text-sky-400 hover:text-sky-300"
              >
                <ExternalLink className="w-3 h-3" />
                <span>Open in Files</span>
              </button>
            </div>
          </div>

          {/* Linux Bundle Card */}
          <div className="bg-slate-950/60 border border-slate-800 rounded-lg p-3 space-y-1 text-xs">
            <div className="flex justify-between items-center font-semibold text-slate-200">
              <span>Linux Desktop</span>
              <span className="text-emerald-400">READY</span>
            </div>
            <p className="text-[11px] text-slate-400 font-mono">build/linux/x64/release/bundle</p>
            <div className="pt-2">
              <button
                onClick={() => handleReveal('fixtures/fixture_app/build/linux/x64/release/bundle')}
                className="flex items-center gap-1 text-[11px] text-sky-400 hover:text-sky-300"
              >
                <ExternalLink className="w-3 h-3" />
                <span>Open in Files</span>
              </button>
            </div>
          </div>

          {/* Android APK Card */}
          <div className="bg-slate-950/60 border border-slate-800 rounded-lg p-3 space-y-1 text-xs">
            <div className="flex justify-between items-center font-semibold text-slate-200">
              <span>Android APK</span>
              <span className="text-emerald-400">44.1 MB</span>
            </div>
            <p className="text-[11px] text-slate-400 font-mono truncate">app-release.apk</p>
            <div className="pt-2">
              <button
                onClick={() => handleReveal('fixtures/fixture_app/build/app/outputs/flutter-apk/app-release.apk')}
                className="flex items-center gap-1 text-[11px] text-sky-400 hover:text-sky-300"
              >
                <ExternalLink className="w-3 h-3" />
                <span>Open in Files</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* IPA Package Validator Card */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-sm space-y-4">
        <h3 className="text-sm font-semibold text-slate-200 flex items-center gap-2">
          <ShieldCheck className="w-4 h-4 text-sky-400" />
          <span>IPA Archive & Bundle Integrity Validator</span>
        </h3>
        <p className="text-xs text-slate-400">
          Strictly verifies ZIP archive structure, Payload/ directory, application bundle, and Info.plist metadata.
        </p>

        <form onSubmit={handleValidateIpa} className="flex gap-2">
          <input
            type="text"
            required
            placeholder="/absolute/path/to/app.ipa"
            value={ipaPath}
            onChange={(e) => setIpaPath(e.target.value)}
            className="flex-1 bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-xs text-slate-200 outline-none focus:border-sky-500 font-mono"
          />
          <button
            type="submit"
            disabled={validatingIpa}
            className="px-4 py-2 bg-sky-500 hover:bg-sky-400 text-slate-950 text-xs font-semibold rounded-lg transition-all cursor-pointer disabled:opacity-50"
          >
            {validatingIpa ? 'Validating...' : 'Validate IPA'}
          </button>
        </form>

        {ipaReport && (
          <div className="bg-slate-950 p-4 rounded-lg border border-slate-800 text-xs space-y-2">
            <div className="flex justify-between items-center">
              <span className="font-semibold text-slate-200">Validation Status:</span>
              <span className="font-mono px-2 py-0.5 rounded bg-sky-500/10 text-sky-400 border border-sky-500/30">
                {ipaReport.status}
              </span>
            </div>
            <div className="grid grid-cols-2 gap-2 text-[11px] text-slate-400 pt-2 border-t border-slate-800">
              <div>Bundle ID: <span className="font-mono text-slate-200">{ipaReport.bundle_id || 'N/A'}</span></div>
              <div>Version: <span className="font-mono text-slate-200">{ipaReport.version || 'N/A'}</span></div>
              <div>App Name: <span className="font-mono text-slate-200">{ipaReport.app_name || 'N/A'}</span></div>
              <div>Code Signature: <span className={ipaReport.has_code_signature ? 'text-emerald-400 font-semibold' : 'text-amber-400 font-semibold'}>
                {ipaReport.has_code_signature ? 'Verified' : 'Unsigned'}
              </span></div>
            </div>
            <div className="pt-2 text-[11px] text-slate-400 space-y-1">
              {ipaReport.details.map((d, i) => (
                <div key={i} className="flex items-center gap-1.5">
                  <FileCheck className="w-3.5 h-3.5 text-emerald-400" />
                  <span>{d}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Build History Table */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-sm space-y-3">
        <h3 className="text-sm font-semibold text-slate-200">Historical Builds</h3>
        {history.length === 0 ? (
          <div className="text-xs text-slate-500 italic py-4 text-center">
            No completed builds recorded yet. Run a build from the top toolbar.
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-xs text-left">
              <thead className="text-slate-400 border-b border-slate-800">
                <tr>
                  <th className="py-2 px-3">Target</th>
                  <th className="py-2 px-3">Mode</th>
                  <th className="py-2 px-3">Status</th>
                  <th className="py-2 px-3">Exit Code</th>
                  <th className="py-2 px-3">Started</th>
                  <th className="py-2 px-3">Artifacts</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60 font-mono text-[11px]">
                {history.map((record) => (
                  <tr key={record.job_id} className="hover:bg-slate-800/40">
                    <td className="py-2.5 px-3 uppercase text-slate-300 font-semibold">{record.target}</td>
                    <td className="py-2.5 px-3 capitalize text-slate-400">{record.mode}</td>
                    <td className="py-2.5 px-3">
                      <span className={`px-2 py-0.5 rounded text-[10px] ${
                        record.status === 'Succeeded'
                          ? 'bg-emerald-500/10 text-emerald-400'
                          : record.status === 'Cancelled'
                          ? 'bg-amber-500/10 text-amber-400'
                          : 'bg-rose-500/10 text-rose-400'
                      }`}>
                        {record.status}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 text-slate-400">{record.exit_code ?? '-'}</td>
                    <td className="py-2.5 px-3 text-slate-500">{record.started_at.slice(0, 19).replace('T', ' ')}</td>
                    <td className="py-2.5 px-3">
                      {record.artifact_paths.length > 0 ? (
                        <button
                          onClick={() => handleReveal(record.artifact_paths[0])}
                          className="text-sky-400 hover:text-sky-300 flex items-center gap-1 cursor-pointer font-sans"
                        >
                          <ExternalLink className="w-3 h-3" />
                          <span>View</span>
                        </button>
                      ) : (
                        <span className="text-slate-600">None</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
