import React, { useState, useRef, useEffect } from 'react';
import { Terminal, Copy, Trash2, Check, Search } from 'lucide-react';

interface TerminalViewProps {
  logs: string[];
  statusText?: string;
  onClear: () => void;
}

export const TerminalView: React.FC<TerminalViewProps> = ({ logs, statusText, onClear }) => {
  const [filter, setFilter] = useState('');
  const [copied, setCopied] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);

  const filteredLogs = filter.trim()
    ? logs.filter((line) => line.toLowerCase().includes(filter.toLowerCase()))
    : logs;

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  const handleCopy = () => {
    navigator.clipboard.writeText(logs.join('\n'));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="h-64 bg-slate-950 border-t border-slate-800 flex flex-col font-mono select-text">
      {/* Terminal Header */}
      <div className="h-9 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-3 select-none">
        <div className="flex items-center gap-2 text-xs text-slate-400">
          <Terminal className="w-3.5 h-3.5 text-sky-400" />
          <span className="font-semibold text-slate-200">Terminal & Build Output</span>
          {statusText && (
            <span className="text-[11px] px-2 py-0.5 rounded bg-sky-500/10 text-sky-400 border border-sky-500/30">
              {statusText}
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          {/* Search */}
          <div className="flex items-center bg-slate-950 px-2 py-1 rounded border border-slate-800 text-xs">
            <Search className="w-3 h-3 text-slate-500 mr-1.5" />
            <input
              type="text"
              placeholder="Filter output..."
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              className="bg-transparent text-slate-300 text-[11px] outline-none w-28"
            />
          </div>

          {/* Copy button */}
          <button
            onClick={handleCopy}
            title="Copy logs"
            className="p-1 text-slate-400 hover:text-slate-200 transition-colors"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
          </button>

          {/* Clear button */}
          <button
            onClick={onClear}
            title="Clear logs"
            className="p-1 text-slate-400 hover:text-slate-200 transition-colors"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Terminal Log Lines */}
      <div className="flex-1 p-3 overflow-y-auto text-[11px] leading-relaxed text-slate-300 space-y-0.5">
        {filteredLogs.length === 0 ? (
          <div className="text-slate-600 italic">No output yet. Run a project action or build to view output.</div>
        ) : (
          filteredLogs.map((line, idx) => (
            <div key={idx} className="whitespace-pre-wrap font-mono break-all">
              {line}
            </div>
          ))
        )}
        <div ref={logEndRef} />
      </div>
    </div>
  );
};
