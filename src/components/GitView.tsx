import React, { useState } from 'react';
import { GitBranch, GitCommit, RefreshCw, Upload, FileText } from 'lucide-react';



interface GitViewProps {
  onLog?: (msg: string) => void;
}

export const GitView: React.FC<GitViewProps> = ({ onLog }) => {
  const [commitMessage, setCommitMessage] = useState('');
  const [isCommitting, setIsCommitting] = useState(false);
  const [changedFiles, setChangedFiles] = useState<{ path: string; status: 'M' | 'A' | 'D' }[]>([
    { path: 'src-tauri/Cargo.toml', status: 'M' },
    { path: 'src-tauri/src/lib.rs', status: 'M' },
    { path: 'src/types.ts', status: 'M' },
    { path: 'src/components/XcodeToolbar.tsx', status: 'A' },
    { path: 'src/components/XcodeInspector.tsx', status: 'A' },
    { path: 'src/components/BottomPanel.tsx', status: 'A' },
  ]);

  const handleCommit = () => {
    if (!commitMessage.trim()) return;
    setIsCommitting(true);
    if (onLog) {
      onLog(`[Git] Committing changes with message: "${commitMessage}"...`);
    }

    setTimeout(() => {
      setIsCommitting(false);
      if (onLog) {
        onLog(`[Git] Commit created successfully on branch main.`);
      }
      setCommitMessage('');
      setChangedFiles([]);
    }, 600);
  };

  const handlePush = () => {
    if (onLog) {
      onLog(`[Git] Pushing branch main to origin https://github.com/Ratanazen/IDE.git...`);
      setTimeout(() => {
        onLog(`[Git] Branch main synced with origin.`);
      }, 500);
    }
  };

  return (
    <div className="flex flex-col h-full bg-slate-900 border-r border-slate-800 text-xs select-none p-3">
      {/* Branch & Actions Header */}
      <div className="flex items-center justify-between pb-2.5 mb-3 border-b border-slate-800">
        <div className="flex items-center gap-1.5 font-semibold text-slate-200">
          <GitBranch className="w-4 h-4 text-sky-400" />
          <span>main</span>
        </div>

        <div className="flex items-center gap-1">
          <button
            onClick={handlePush}
            title="Push to Remote (origin/main)"
            className="p-1 hover:bg-slate-800 text-slate-400 hover:text-sky-400 rounded"
          >
            <Upload className="w-3.5 h-3.5" />
          </button>
          <button
            title="Refresh Git Status"
            className="p-1 hover:bg-slate-800 text-slate-400 hover:text-slate-200 rounded"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Commit Input Area */}
      <div className="mb-3 space-y-2">
        <textarea
          rows={3}
          placeholder="Message (Cmd+Enter to commit)"
          value={commitMessage}
          onChange={(e) => setCommitMessage(e.target.value)}
          className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-slate-200 placeholder-slate-500 outline-none focus:border-sky-500 font-sans resize-none"
        />

        <button
          onClick={handleCommit}
          disabled={!commitMessage.trim() || isCommitting}
          className="w-full py-1.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-medium rounded-lg flex items-center justify-center gap-1.5 transition-colors shadow-sm"
        >
          <GitCommit className="w-3.5 h-3.5" />
          <span>{isCommitting ? 'Committing...' : 'Commit'}</span>
        </button>
      </div>

      {/* Changed Files List */}
      <div className="flex-1 overflow-y-auto">
        <div className="flex justify-between items-center mb-1.5">
          <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">
            Changes ({changedFiles.length})
          </span>
        </div>

        {changedFiles.length === 0 ? (
          <div className="text-center py-6 text-slate-500 text-[11px]">
            Working tree is clean.
          </div>
        ) : (
          <div className="space-y-0.5">
            {changedFiles.map((file) => (
              <div
                key={file.path}
                className="flex items-center justify-between px-2 py-1.5 rounded hover:bg-slate-800/60 cursor-pointer group"
              >
                <div className="flex items-center gap-1.5 text-slate-300 group-hover:text-white truncate">
                  <FileText className="w-3.5 h-3.5 text-slate-500 flex-shrink-0" />
                  <span className="truncate">{file.path}</span>
                </div>

                <span
                  className={`text-[10px] font-mono font-bold px-1 rounded ${
                    file.status === 'M'
                      ? 'text-amber-400 bg-amber-400/10'
                      : file.status === 'A'
                      ? 'text-emerald-400 bg-emerald-400/10'
                      : 'text-rose-400 bg-rose-400/10'
                  }`}
                >
                  {file.status}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
