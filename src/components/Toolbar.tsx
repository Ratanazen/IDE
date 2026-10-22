import React from 'react';
import { Play, Square, ExternalLink } from 'lucide-react';
import { BuildMode, BuildTarget, ProjectMetadata } from '../types';

interface ToolbarProps {
  project: ProjectMetadata | null;
  target: BuildTarget;
  mode: BuildMode;
  isBuilding: boolean;
  cleanBeforeBuild: boolean;
  runAnalyze: boolean;
  runTests: boolean;
  onTargetChange: (target: BuildTarget) => void;
  onModeChange: (mode: BuildMode) => void;
  onCleanToggle: () => void;
  onAnalyzeToggle: () => void;
  onTestsToggle: () => void;
  onBuild: () => void;
  onCancel: () => void;
  onOpenOutput: () => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  project,
  target,
  mode,
  isBuilding,
  cleanBeforeBuild,
  runAnalyze,
  runTests,
  onTargetChange,
  onModeChange,
  onCleanToggle,
  onAnalyzeToggle,
  onTestsToggle,
  onBuild,
  onCancel,
  onOpenOutput,
}) => {
  return (
    <header className="h-14 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 gap-4 select-none">
      <div className="flex items-center gap-3">
        <div className="text-xs">
          <span className="text-slate-500">Active Project: </span>
          <span className="font-semibold text-slate-200">
            {project ? project.name : 'No project loaded'}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-3">
        {/* Target Selector */}
        <div className="flex items-center gap-1.5 bg-slate-800/80 rounded-lg p-1 border border-slate-700">
          <span className="text-[11px] text-slate-400 pl-2 pr-1 font-medium">Target:</span>
          <select
            value={target}
            onChange={(e) => onTargetChange(e.target.value as BuildTarget)}
            disabled={isBuilding}
            className="bg-slate-900 text-slate-200 text-xs rounded px-2 py-1 outline-none border border-slate-700 focus:border-sky-500 cursor-pointer"
          >
            <option value="web">Web (dart2js / wasm)</option>
            <option value="linux">Linux Desktop (GTK)</option>
            <option value="apk">Android APK</option>
            <option value="appbundle">Android AAB (App Bundle)</option>
            <option value="ios">iOS (Experimental)</option>
          </select>
        </div>

        {/* Mode Selector */}
        <div className="flex items-center gap-1.5 bg-slate-800/80 rounded-lg p-1 border border-slate-700">
          <span className="text-[11px] text-slate-400 pl-2 pr-1 font-medium">Mode:</span>
          <select
            value={mode}
            onChange={(e) => onModeChange(e.target.value as BuildMode)}
            disabled={isBuilding}
            className="bg-slate-900 text-slate-200 text-xs rounded px-2 py-1 outline-none border border-slate-700 focus:border-sky-500 cursor-pointer"
          >
            <option value="debug">Debug</option>
            <option value="profile">Profile</option>
            <option value="release">Release</option>
          </select>
        </div>

        {/* Pre-step Toggles */}
        <div className="flex items-center gap-3 text-xs text-slate-300 px-2 py-1 bg-slate-800/40 rounded-lg border border-slate-800">
          <label className="flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={cleanBeforeBuild}
              onChange={onCleanToggle}
              disabled={isBuilding}
              className="rounded bg-slate-900 border-slate-700 text-sky-500 focus:ring-0"
            />
            <span>Clean</span>
          </label>
          <label className="flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={runAnalyze}
              onChange={onAnalyzeToggle}
              disabled={isBuilding}
              className="rounded bg-slate-900 border-slate-700 text-sky-500 focus:ring-0"
            />
            <span>Analyze</span>
          </label>
          <label className="flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={runTests}
              onChange={onTestsToggle}
              disabled={isBuilding}
              className="rounded bg-slate-900 border-slate-700 text-sky-500 focus:ring-0"
            />
            <span>Test</span>
          </label>
        </div>

        {/* Build / Cancel / Open buttons */}
        {!isBuilding ? (
          <button
            onClick={onBuild}
            disabled={!project}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold shadow transition-all ${
              project
                ? 'bg-sky-500 hover:bg-sky-400 text-slate-950 cursor-pointer'
                : 'bg-slate-800 text-slate-600 cursor-not-allowed border border-slate-700'
            }`}
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            <span>Build</span>
          </button>
        ) : (
          <button
            onClick={onCancel}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-rose-600 hover:bg-rose-500 text-white shadow transition-all cursor-pointer"
          >
            <Square className="w-3.5 h-3.5 fill-current" />
            <span>Cancel</span>
          </button>
        )}

        <button
          onClick={onOpenOutput}
          disabled={!project}
          title="Open Output Directory"
          className="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition-all cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <ExternalLink className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
