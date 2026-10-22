import React, { useState, useRef, useEffect } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Info,
  Terminal as TerminalIcon,
  FileCode,
  Trash2,
  Maximize2,
  Minimize2,
  X,
  Play,
  Search,
} from 'lucide-react';
import { BottomPanelTab, Diagnostic } from '../types';

interface BottomPanelProps {
  activeTab: BottomPanelTab;
  onTabChange: (tab: BottomPanelTab) => void;
  onClose: () => void;
  diagnostics: Diagnostic[];
  outputLogs: string[];
  statusText: string;
  onSelectDiagnostic?: (diag: Diagnostic) => void;
  onClearLogs?: () => void;
  onExecuteCommand?: (cmd: string) => void;
}

export const BottomPanel: React.FC<BottomPanelProps> = ({
  activeTab,
  onTabChange,
  onClose,
  diagnostics,
  outputLogs,
  statusText,
  onSelectDiagnostic,
  onClearLogs,
  onExecuteCommand,
}) => {
  const [panelHeight] = useState(240);
  const [isMaximized, setIsMaximized] = useState(false);
  const [selectedChannel, setSelectedChannel] = useState<string>('all');
  const [logFilter, setLogFilter] = useState('');
  const [debugInput, setDebugInput] = useState('');

  const [terminalInput, setTerminalInput] = useState('');
  const [debugLogs, setDebugLogs] = useState<string[]>([
    '[LLDB] Debugger engine initialized.',
    '[LLDB] Target architecture: arm64-apple-darwin / x86_64-linux-gnu.',
  ]);

  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [outputLogs, debugLogs]);

  const errorCount = diagnostics.filter((d) => d.severity === 'error').length;
  const warningCount = diagnostics.filter((d) => d.severity === 'warning').length;

  const filteredLogs = outputLogs.filter((line) => {
    if (logFilter && !line.toLowerCase().includes(logFilter.toLowerCase())) {
      return false;
    }
    if (selectedChannel === 'all') return true;
    if (selectedChannel === 'build') return line.includes('[Build]') || line.includes('[Compiler]');
    if (selectedChannel === 'flutter') return line.includes('[Flutter]') || line.includes('flutter');
    if (selectedChannel === 'system') return line.includes('[System]') || line.includes('[Toolchain]');
    return true;
  });

  const handleDebugSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!debugInput.trim()) return;
    const cmd = debugInput.trim();
    setDebugLogs((prev) => [...prev, `(lldb) ${cmd}`, `Executed: ${cmd}`]);
    setDebugInput('');
  };

  const handleTerminalSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!terminalInput.trim()) return;
    if (onExecuteCommand) onExecuteCommand(terminalInput.trim());
    setTerminalInput('');
  };

  return (
    <div
      style={{ height: isMaximized ? '60vh' : `${panelHeight}px` }}
      className="bg-slate-950 border-t border-slate-800 flex flex-col select-none text-xs flex-shrink-0 z-20 transition-all duration-150"
    >
      {/* Panel Tab Header */}
      <div className="h-9 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-3">
        {/* Tabs */}
        <div className="flex items-center gap-1">
          {/* Tab: Problems */}
          <button
            onClick={() => onTabChange('problems')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-t font-medium transition-colors ${
              activeTab === 'problems'
                ? 'text-sky-400 bg-slate-950 border-t border-x border-slate-800'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <AlertCircle className="w-3.5 h-3.5 text-rose-400" />
            <span>Problems</span>
            {(errorCount > 0 || warningCount > 0) && (
              <span className="ml-1 px-1.5 py-0.2 rounded-full text-[10px] bg-slate-800 font-mono font-bold text-slate-300">
                {errorCount}E {warningCount}W
              </span>
            )}
          </button>

          {/* Tab: Output */}
          <button
            onClick={() => onTabChange('output')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-t font-medium transition-colors ${
              activeTab === 'output'
                ? 'text-sky-400 bg-slate-950 border-t border-x border-slate-800'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <FileCode className="w-3.5 h-3.5" />
            <span>Output</span>
          </button>

          {/* Tab: Debug Console */}
          <button
            onClick={() => onTabChange('debug')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-t font-medium transition-colors ${
              activeTab === 'debug'
                ? 'text-sky-400 bg-slate-950 border-t border-x border-slate-800'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Play className="w-3.5 h-3.5 text-emerald-400" />
            <span>Debug Console</span>
          </button>

          {/* Tab: Terminal */}
          <button
            onClick={() => onTabChange('terminal')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-t font-medium transition-colors ${
              activeTab === 'terminal'
                ? 'text-sky-400 bg-slate-950 border-t border-x border-slate-800'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <TerminalIcon className="w-3.5 h-3.5" />
            <span>Terminal</span>
          </button>
        </div>

        {/* Right actions (Clear, Maximize, Close) */}
        <div className="flex items-center gap-1 text-slate-400">
          {statusText && (
            <span className="text-[10px] font-mono text-slate-500 mr-2 uppercase">
              [{statusText}]
            </span>
          )}
          {activeTab === 'output' && onClearLogs && (

            <button
              onClick={onClearLogs}
              title="Clear Output"
              className="p-1 hover:text-slate-200 hover:bg-slate-800 rounded"
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          )}

          <button
            onClick={() => setIsMaximized(!isMaximized)}
            title={isMaximized ? 'Restore Panel' : 'Maximize Panel'}
            className="p-1 hover:text-slate-200 hover:bg-slate-800 rounded"
          >
            {isMaximized ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
          </button>

          <button
            onClick={onClose}
            title="Close Panel (Ctrl+`)"
            className="p-1 hover:text-slate-200 hover:bg-slate-800 rounded"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Panel Body */}
      <div className="flex-1 overflow-y-auto">
        {/* PROBLEMS TAB */}
        {activeTab === 'problems' && (
          <div className="p-2">
            {diagnostics.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-28 text-slate-500">
                <AlertCircle className="w-6 h-6 mb-1 text-slate-600" />
                <span>No problems have been detected in the workspace.</span>
              </div>
            ) : (
              <table className="w-full text-left font-mono text-[11px] border-collapse">
                <thead>
                  <tr className="border-b border-slate-800 text-slate-500 text-[10px] uppercase">
                    <th className="py-1 px-2 w-8">Type</th>
                    <th className="py-1 px-2">Message</th>
                    <th className="py-1 px-2 w-48">Location</th>
                    <th className="py-1 px-2 w-24">Source</th>
                  </tr>
                </thead>
                <tbody>
                  {diagnostics.map((diag) => (
                    <tr
                      key={diag.id}
                      onClick={() => onSelectDiagnostic && onSelectDiagnostic(diag)}
                      className="border-b border-slate-900/60 hover:bg-slate-900 cursor-pointer transition-colors"
                    >
                      <td className="py-1.5 px-2">
                        {diag.severity === 'error' ? (
                          <AlertCircle className="w-3.5 h-3.5 text-rose-400" />
                        ) : diag.severity === 'warning' ? (
                          <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
                        ) : (
                          <Info className="w-3.5 h-3.5 text-sky-400" />
                        )}
                      </td>
                      <td className="py-1.5 px-2 text-slate-200 font-sans font-medium">
                        {diag.message}
                      </td>
                      <td className="py-1.5 px-2 text-slate-400">
                        {diag.file_path}:{diag.line}:{diag.column}
                      </td>
                      <td className="py-1.5 px-2 text-slate-500">{diag.source}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {/* OUTPUT TAB */}
        {activeTab === 'output' && (
          <div className="flex flex-col h-full">
            <div className="h-8 border-b border-slate-800/80 px-3 flex items-center justify-between bg-slate-900/60 text-[11px]">
              <div className="flex items-center gap-2">
                <span className="text-slate-500">Channel:</span>
                <select
                  value={selectedChannel}
                  onChange={(e) => setSelectedChannel(e.target.value)}
                  className="bg-slate-950 border border-slate-800 rounded px-2 py-0.5 text-slate-200 outline-none"
                >
                  <option value="all">All Channels</option>
                  <option value="build">Build &amp; Compiler</option>
                  <option value="flutter">Flutter Tools</option>
                  <option value="system">System &amp; Toolchain</option>
                </select>
              </div>

              <div className="flex items-center gap-1.5 bg-slate-950 px-2 py-0.5 rounded border border-slate-800">
                <Search className="w-3 h-3 text-slate-500" />
                <input
                  type="text"
                  placeholder="Filter logs..."
                  value={logFilter}
                  onChange={(e) => setLogFilter(e.target.value)}
                  className="bg-transparent border-none outline-none text-slate-300 text-[10px] w-32"
                />
              </div>
            </div>

            <div className="flex-1 p-3 font-mono text-[11px] text-slate-300 overflow-y-auto space-y-0.5">
              {filteredLogs.map((log, idx) => (
                <div
                  key={idx}
                  className={`leading-relaxed whitespace-pre-wrap ${
                    log.includes('[Error]') || log.includes('error:')
                      ? 'text-rose-400 font-semibold'
                      : log.includes('[Warning]') || log.includes('warning:')
                      ? 'text-amber-300'
                      : log.includes('[Success]')
                      ? 'text-emerald-400 font-semibold'
                      : 'text-slate-300'
                  }`}
                >
                  {log}
                </div>
              ))}
              <div ref={logsEndRef} />
            </div>
          </div>
        )}

        {/* DEBUG CONSOLE TAB */}
        {activeTab === 'debug' && (
          <div className="flex flex-col h-full p-2">
            <div className="flex-1 overflow-y-auto font-mono text-[11px] text-slate-300 space-y-1">
              {debugLogs.map((log, idx) => (
                <div key={idx} className="leading-relaxed">
                  {log}
                </div>
              ))}
            </div>
            <form onSubmit={handleDebugSubmit} className="mt-2 flex gap-2">
              <input
                type="text"
                placeholder="Evaluate expression or enter LLDB command (e.g. po self, bt)..."
                value={debugInput}
                onChange={(e) => setDebugInput(e.target.value)}
                className="flex-1 bg-slate-900 border border-slate-700 rounded px-2.5 py-1 text-xs text-slate-200 outline-none focus:border-sky-500 font-mono"
              />
              <button
                type="submit"
                className="px-3 py-1 bg-sky-600 hover:bg-sky-500 text-white rounded font-medium text-xs"
              >
                Evaluate
              </button>
            </form>
          </div>
        )}

        {/* TERMINAL TAB */}
        {activeTab === 'terminal' && (
          <div className="flex flex-col h-full p-2">
            <div className="mb-2 flex items-center gap-2">
              <span className="text-[10px] text-slate-500 uppercase font-bold tracking-wider">
                Quick Actions:
              </span>
              <button
                onClick={() => onExecuteCommand && onExecuteCommand('flutter doctor')}
                className="px-2 py-0.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded text-[10px]"
              >
                flutter doctor
              </button>
              <button
                onClick={() => onExecuteCommand && onExecuteCommand('cargo test')}
                className="px-2 py-0.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded text-[10px]"
              >
                cargo test
              </button>
              <button
                onClick={() => onExecuteCommand && onExecuteCommand('git status')}
                className="px-2 py-0.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded text-[10px]"
              >
                git status
              </button>
            </div>

            <div className="flex-1 overflow-y-auto font-mono text-[11px] text-slate-300 bg-slate-950 p-2.5 rounded border border-slate-900 space-y-1">
              <div className="text-slate-500">$ # Ridu IDE Integrated Shell Session</div>
              {outputLogs.slice(-10).map((l, i) => (
                <div key={i}>{l}</div>
              ))}
            </div>

            <form onSubmit={handleTerminalSubmit} className="mt-2 flex gap-2">
              <input
                type="text"
                placeholder="Enter command (e.g. flutter pub get, git status)..."
                value={terminalInput}
                onChange={(e) => setTerminalInput(e.target.value)}
                className="flex-1 bg-slate-900 border border-slate-700 rounded px-2.5 py-1 text-xs text-slate-200 outline-none focus:border-sky-500 font-mono"
              />
              <button
                type="submit"
                className="px-3 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded font-medium text-xs"
              >
                Run
              </button>
            </form>
          </div>
        )}
      </div>
    </div>
  );
};
