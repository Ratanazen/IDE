import React from 'react';
import { Play, Square, ExternalLink, Zap, RotateCcw, Smartphone } from 'lucide-react';
import { BuildMode, BuildTarget, Device, ProjectMetadata } from '../types';

interface ToolbarProps {
  project: ProjectMetadata | null;
  target: BuildTarget;
  mode: BuildMode;
  devices: Device[];
  selectedDevice: string;
  isBuilding: boolean;
  cleanBeforeBuild: boolean;
  runAnalyze: boolean;
  runTests: boolean;
  onTargetChange: (target: BuildTarget) => void;
  onModeChange: (mode: BuildMode) => void;
  onDeviceChange: (deviceId: string) => void;
  onCleanToggle: () => void;
  onAnalyzeToggle: () => void;
  onTestsToggle: () => void;
  onBuild: () => void;
  onCancel: () => void;
  onOpenOutput: () => void;
  onHotReload: () => void;
  onHotRestart: () => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  project,
  target,
  mode,
  devices,
  selectedDevice,
  isBuilding,
  cleanBeforeBuild,
  runAnalyze,
  runTests,
  onTargetChange,
  onModeChange,
  onDeviceChange,
  onCleanToggle,
  onAnalyzeToggle,
  onTestsToggle,
  onBuild,
  onCancel,
  onOpenOutput,
  onHotReload,
  onHotRestart,
}) => {
  return (
    <header className="h-14 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 gap-3 select-none flex-shrink-0">
      <div className="flex items-center gap-3">
        <div className="text-xs">
          <span className="text-slate-500">Project: </span>
          <span className="font-semibold text-slate-200">
            {project ? project.name : 'No project loaded'}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2">
        {/* Device Selector */}
        <div className="flex items-center gap-1.5 bg-slate-800/80 rounded-lg p-1 border border-slate-700">
          <Smartphone className="w-3.5 h-3.5 text-sky-400 ml-1.5" />
          <select
            value={selectedDevice}
            onChange={(e) => onDeviceChange(e.target.value)}
            className="bg-slate-900 text-slate-200 text-xs rounded px-2 py-1 outline-none border border-slate-700 focus:border-sky-500 cursor-pointer max-w-[150px]"
          >
            {devices.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name} ({d.targetPlatform})
              </option>
            ))}
            {devices.length === 0 && <option value="linux">Linux (Desktop)</option>}
          </select>
        </div>

        {/* Target Selector */}
        <div className="flex items-center gap-1.5 bg-slate-800/80 rounded-lg p-1 border border-slate-700">
          <span className="text-[11px] text-slate-400 pl-1.5 font-medium">Target:</span>
          <select
            value={target}
            onChange={(e) => onTargetChange(e.target.value as BuildTarget)}
            disabled={isBuilding}
            className="bg-slate-900 text-slate-200 text-xs rounded px-2 py-1 outline-none border border-slate-700 focus:border-sky-500 cursor-pointer"
          >
            <option value="web">Web (dart2js)</option>
            <option value="linux">Linux Desktop (GTK)</option>
            <option value="apk">Android APK</option>
            <option value="appbundle">Android AAB</option>
            <option value="ios">iOS (Experimental)</option>
          </select>
        </div>

        {/* Mode Selector */}
        <div className="flex items-center gap-1.5 bg-slate-800/80 rounded-lg p-1 border border-slate-700">
          <span className="text-[11px] text-slate-400 pl-1.5 font-medium">Mode:</span>
          <select
            value={mode}
            onChange={(e) => onModeChange(e.target.value as BuildMode)}
            disabled={isBuilding}
            className="bg-slate-900 text-slate-200 text-xs rounded px-2 py-1 outline-none border border-slate-700 focus:border-sky-500 cursor-pointer"
          >
            <option value="release">Release</option>
            <option value="debug">Debug</option>
            <option value="profile">Profile</option>
          </select>
        </div>

        {/* Pre-step Toggles */}
        <div className="flex items-center gap-2 text-xs text-slate-300 px-2 py-1 bg-slate-800/40 rounded-lg border border-slate-800">
          <label className="flex items-center gap-1 cursor-pointer">
            <input
              type="checkbox"
              checked={cleanBeforeBuild}
              onChange={onCleanToggle}
              disabled={isBuilding}
              className="rounded bg-slate-900 border-slate-700 text-sky-500"
            />
            <span>Clean</span>
          </label>
          <label className="flex items-center gap-1 cursor-pointer">
            <input
              type="checkbox"
              checked={runAnalyze}
              onChange={onAnalyzeToggle}
              disabled={isBuilding}
              className="rounded bg-slate-900 border-slate-700 text-sky-500"
            />
            <span>Analyze</span>
          </label>
          <label className="flex items-center gap-1 cursor-pointer">
            <input
              type="checkbox"
              checked={runTests}
              onChange={onTestsToggle}
              disabled={isBuilding}
              className="rounded bg-slate-900 border-slate-700 text-sky-500"
            />
            <span>Test</span>
          </label>
        </div>

        {/* Hot Reload & Restart */}
        <div className="flex items-center gap-1">
          <button
            onClick={onHotReload}
            title="Hot Reload"
            className="p-1.5 rounded-lg bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 border border-amber-500/30 transition-all cursor-pointer"
          >
            <Zap className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={onHotRestart}
            title="Hot Restart"
            className="p-1.5 rounded-lg bg-sky-500/10 hover:bg-sky-500/20 text-sky-400 border border-sky-500/30 transition-all cursor-pointer"
          >
            <RotateCcw className="w-3.5 h-3.5" />
          </button>
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
