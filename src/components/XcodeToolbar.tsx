import React, { useState } from 'react';
import {
  Play,
  Square,
  Hammer,
  Trash2,
  Zap,
  RotateCcw,
  Smartphone,
  Monitor,
  Globe,
  ChevronDown,
  LayoutTemplate,
  PanelLeft,
  PanelBottom,
  PanelRight,
  Search,
  Check,
} from 'lucide-react';
import {
  BuildMode,
  BuildTarget,
  Device,
  EcosystemProject,
  ProjectMetadata,
  SchemeModel,
} from '../types';

interface XcodeToolbarProps {
  project: ProjectMetadata | null;
  ecosystemProject: EcosystemProject | null;
  target: BuildTarget;
  mode: BuildMode;
  devices: Device[];
  selectedDevice: string;
  isBuilding: boolean;
  statusText: string;
  isSidebarOpen: boolean;
  isBottomPanelOpen: boolean;
  isInspectorOpen: boolean;
  isSwiftUiPreviewOpen: boolean;
  onTargetChange: (target: BuildTarget) => void;
  onModeChange: (mode: BuildMode) => void;
  onDeviceChange: (deviceId: string) => void;
  onSchemeChange?: (schemeName: string) => void;
  onBuild: () => void;
  onRun: () => void;
  onStop: () => void;
  onClean: () => void;
  onHotReload: () => void;
  onHotRestart: () => void;
  onToggleSidebar: () => void;
  onToggleBottomPanel: () => void;
  onToggleInspector: () => void;
  onToggleSwiftUiPreview: () => void;
  onOpenCommandPalette: () => void;
}

export const XcodeToolbar: React.FC<XcodeToolbarProps> = ({
  project,
  ecosystemProject,
  target,
  mode,
  devices,
  selectedDevice,
  isBuilding,
  statusText,
  isSidebarOpen,
  isBottomPanelOpen,
  isInspectorOpen,
  isSwiftUiPreviewOpen,
  onTargetChange,
  onModeChange,
  onDeviceChange,
  onSchemeChange,
  onBuild,
  onRun,
  onStop,
  onClean,
  onHotReload,
  onHotRestart,
  onToggleSidebar,
  onToggleBottomPanel,
  onToggleInspector,
  onToggleSwiftUiPreview,
  onOpenCommandPalette,
}) => {
  const [isSchemePopoverOpen, setIsSchemePopoverOpen] = useState(false);

  const activeSchemeName =
    ecosystemProject?.active_scheme ||
    (project ? `${project.name} (${target})` : 'Default Scheme');

  const currentDevice = devices.find((d) => d.id === selectedDevice);
  const deviceName = currentDevice
    ? currentDevice.name
    : selectedDevice === 'linux'
    ? 'Linux (Desktop)'
    : selectedDevice === 'web-server' || target === 'web'
    ? 'Web (Chrome)'
    : 'Default Runner';

  const getDeviceIcon = () => {
    if (target === 'web' || selectedDevice.includes('web') || selectedDevice.includes('chrome')) {
      return <Globe className="w-3.5 h-3.5 text-emerald-400" />;
    }
    if (target === 'linux' || selectedDevice === 'linux') {
      return <Monitor className="w-3.5 h-3.5 text-sky-400" />;
    }
    return <Smartphone className="w-3.5 h-3.5 text-amber-400" />;
  };

  const schemes: SchemeModel[] = ecosystemProject?.schemes && ecosystemProject.schemes.length > 0
    ? ecosystemProject.schemes
    : [
        {
          name: `${project?.name || 'App'} (Release)`,
          build_target_id: 'default_target',
          default_configuration: 'Release',
          launch_arguments: [],
          environment_variables: {},
        },
      ];

  return (
    <header className="h-12 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-3 gap-3 select-none flex-shrink-0 z-30">
      {/* Left: Execution Controls (Run, Stop, Build, Clean, Hot Reload) */}
      <div className="flex items-center gap-1.5">
        <button
          onClick={onRun}
          disabled={isBuilding}
          title="Run Active Scheme (Cmd+R)"
          className="p-1.5 rounded-md hover:bg-slate-800 text-slate-300 hover:text-emerald-400 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          <Play className="w-4 h-4 fill-current" />
        </button>

        <button
          onClick={onStop}
          title="Stop Execution (Cmd+.)"
          className="p-1.5 rounded-md hover:bg-slate-800 text-slate-300 hover:text-rose-400 transition-colors"
        >
          <Square className="w-4 h-4 fill-current" />
        </button>

        <button
          onClick={onBuild}
          disabled={isBuilding}
          title="Build Scheme (Cmd+B)"
          className="p-1.5 rounded-md hover:bg-slate-800 text-slate-300 hover:text-sky-400 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          <Hammer className="w-4 h-4" />
        </button>

        <button
          onClick={onClean}
          disabled={isBuilding}
          title="Clean Build Folder (Cmd+Shift+K)"
          className="p-1.5 rounded-md hover:bg-slate-800 text-slate-400 hover:text-amber-400 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          <Trash2 className="w-4 h-4" />
        </button>

        <div className="h-4 w-px bg-slate-800 mx-1" />

        <button
          onClick={onHotReload}
          title="Hot Reload ('r')"
          className="p-1.5 rounded-md hover:bg-slate-800 text-slate-400 hover:text-amber-300 transition-colors"
        >
          <Zap className="w-3.5 h-3.5" />
        </button>

        <button
          onClick={onHotRestart}
          title="Hot Restart ('R')"
          className="p-1.5 rounded-md hover:bg-slate-800 text-slate-400 hover:text-sky-300 transition-colors"
        >
          <RotateCcw className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Center: The Xcode Scheme & Destination Pill */}
      <div className="relative flex-1 max-w-xl mx-auto flex items-center justify-center">
        <div
          onClick={() => setIsSchemePopoverOpen(!isSchemePopoverOpen)}
          className="flex items-center gap-2 bg-slate-950/80 hover:bg-slate-950 border border-slate-700/80 hover:border-slate-600 rounded-lg px-3 py-1 text-xs cursor-pointer transition-all shadow-inner max-w-full"
        >
          <div className="font-semibold text-slate-200 truncate max-w-[160px]">
            {activeSchemeName}
          </div>

          <span className="text-slate-600 font-bold">&gt;</span>

          <span className="px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider bg-slate-800 text-sky-400 border border-slate-700">
            {mode}
          </span>

          <span className="text-slate-600 font-bold">|</span>

          <div className="flex items-center gap-1.5 text-slate-300">
            {getDeviceIcon()}
            <span className="truncate max-w-[140px] text-[11px]">{deviceName}</span>
          </div>

          <ChevronDown className="w-3 h-3 text-slate-500 ml-0.5" />
        </div>

        {/* Scheme & Device Popover */}
        {isSchemePopoverOpen && (
          <div className="absolute top-10 w-96 bg-slate-900 border border-slate-700 rounded-xl shadow-2xl p-3 z-50 text-xs">
            <div className="flex justify-between items-center mb-2 pb-1.5 border-b border-slate-800">
              <span className="font-semibold text-slate-200">Scheme &amp; Destination</span>
              <button
                onClick={() => setIsSchemePopoverOpen(false)}
                className="text-slate-500 hover:text-slate-300 text-xs"
              >
                Done
              </button>
            </div>

            {/* Scheme List */}
            <div className="mb-3">
              <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1">
                Schemes / Targets
              </span>
              <div className="space-y-0.5 max-h-32 overflow-y-auto pr-1">
                {schemes.map((s) => (
                  <button
                    key={s.name}
                    onClick={() => {
                      if (onSchemeChange) onSchemeChange(s.name);
                      setIsSchemePopoverOpen(false);
                    }}
                    className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-md text-left transition-colors ${
                      activeSchemeName === s.name
                        ? 'bg-sky-500/10 text-sky-400 font-medium'
                        : 'text-slate-300 hover:bg-slate-800/80'
                    }`}
                  >
                    <span className="truncate">{s.name}</span>
                    {activeSchemeName === s.name && <Check className="w-3.5 h-3.5 text-sky-400" />}
                  </button>
                ))}
              </div>
            </div>

            {/* Build Configuration (Debug / Release) */}
            <div className="mb-3">
              <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1">
                Build Configuration
              </span>
              <div className="flex gap-1.5">
                {(['debug', 'release', 'profile'] as BuildMode[]).map((m) => (
                  <button
                    key={m}
                    onClick={() => onModeChange(m)}
                    className={`flex-1 py-1 text-center rounded-md text-[11px] font-medium capitalize border transition-all ${
                      mode === m
                        ? 'bg-sky-500/20 text-sky-300 border-sky-500/40 shadow-sm'
                        : 'bg-slate-800/60 text-slate-400 border-slate-700/60 hover:text-slate-200'
                    }`}
                  >
                    {m}
                  </button>
                ))}
              </div>
            </div>

            {/* Destination Devices */}
            <div>
              <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1">
                Destination Device
              </span>
              <div className="space-y-0.5 max-h-36 overflow-y-auto pr-1">
                {devices.map((d) => (
                  <button
                    key={d.id}
                    onClick={() => {
                      onDeviceChange(d.id);
                      setIsSchemePopoverOpen(false);
                    }}
                    className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-md text-left transition-colors ${
                      selectedDevice === d.id
                        ? 'bg-sky-500/10 text-sky-400 font-medium'
                        : 'text-slate-300 hover:bg-slate-800/80'
                    }`}
                  >
                    <Smartphone className="w-3.5 h-3.5 text-amber-400" />
                    <span className="flex-1 truncate">{d.name}</span>
                    <span className="text-[10px] text-slate-500">{d.targetPlatform}</span>
                  </button>
                ))}

                {/* Default Linux Desktop runner */}
                <button
                  onClick={() => {
                    onDeviceChange('linux');
                    onTargetChange('linux');
                    setIsSchemePopoverOpen(false);
                  }}
                  className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-md text-left transition-colors ${
                    selectedDevice === 'linux'
                      ? 'bg-sky-500/10 text-sky-400 font-medium'
                      : 'text-slate-300 hover:bg-slate-800/80'
                  }`}
                >
                  <Monitor className="w-3.5 h-3.5 text-sky-400" />
                  <span className="flex-1">Linux Desktop (x86_64)</span>
                  <span className="text-[10px] text-slate-500">Native GTK</span>
                </button>

                {/* Web runner */}
                <button
                  onClick={() => {
                    onDeviceChange('web-server');
                    onTargetChange('web');
                    setIsSchemePopoverOpen(false);
                  }}
                  className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-md text-left transition-colors ${
                    target === 'web'
                      ? 'bg-sky-500/10 text-sky-400 font-medium'
                      : 'text-slate-300 hover:bg-slate-800/80'
                  }`}
                >
                  <Globe className="w-3.5 h-3.5 text-emerald-400" />
                  <span className="flex-1">Chrome / Web Server</span>
                  <span className="text-[10px] text-slate-500">HTML5/Wasm</span>
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Right: Search / Palette & Panel Layout Toggles */}
      <div className="flex items-center gap-1.5">
        <button
          onClick={onOpenCommandPalette}
          title="Command Palette (Ctrl+Shift+P)"
          className="flex items-center gap-1.5 bg-slate-950/60 hover:bg-slate-950 text-slate-400 hover:text-slate-200 border border-slate-800 rounded-md px-2 py-1 text-[11px] transition-colors"
        >
          <Search className="w-3.5 h-3.5 text-slate-500" />
          <span className="hidden md:inline">Commands</span>
          <kbd className="text-[9px] bg-slate-800 px-1 rounded text-slate-400">Ctrl+Shift+P</kbd>
        </button>

        <div className="h-4 w-px bg-slate-800 mx-1" />

        <button
          onClick={onToggleSwiftUiPreview}
          title="Toggle SwiftUI Preview Canvas"
          className={`p-1.5 rounded-md transition-colors ${
            isSwiftUiPreviewOpen
              ? 'bg-sky-500/10 text-sky-400 border border-sky-500/30'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          <LayoutTemplate className="w-4 h-4" />
        </button>

        <button
          onClick={onToggleSidebar}
          title="Toggle Primary Sidebar (Ctrl+B)"
          className={`p-1.5 rounded-md transition-colors ${
            isSidebarOpen
              ? 'bg-sky-500/10 text-sky-400'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          <PanelLeft className="w-4 h-4" />
        </button>

        <button
          onClick={onToggleBottomPanel}
          title="Toggle Bottom Panel (Ctrl+`)"
          className={`p-1.5 rounded-md transition-colors ${
            isBottomPanelOpen
              ? 'bg-sky-500/10 text-sky-400'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          <PanelBottom className="w-4 h-4" />
        </button>

        <button
          onClick={onToggleInspector}
          title="Toggle Xcode Inspector"
          className={`p-1.5 rounded-md transition-colors ${
            isInspectorOpen
              ? 'bg-sky-500/10 text-sky-400'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          }`}
        >
          <PanelRight className="w-4 h-4" />
        </button>

        {/* Status pill */}
        <div className="ml-1 flex items-center gap-1.5 pl-2 border-l border-slate-800">
          <div
            className={`w-2 h-2 rounded-full ${
              isBuilding
                ? 'bg-amber-400 animate-pulse'
                : statusText === 'ERROR'
                ? 'bg-rose-500'
                : 'bg-emerald-500'
            }`}
          />
          <span className="text-[10px] font-mono text-slate-400 uppercase hidden lg:inline">
            {isBuilding ? 'BUILDING' : statusText}
          </span>
        </div>
      </div>
    </header>
  );
};
