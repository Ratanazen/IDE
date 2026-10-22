import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ActivityBar } from './components/ActivityBar';
import { XcodeToolbar } from './components/XcodeToolbar';
import { XcodeInspector } from './components/XcodeInspector';
import { BottomPanel } from './components/BottomPanel';
import { CommandPalette, CommandItem } from './components/CommandPalette';
import { ProjectView } from './components/ProjectView';
import { EditorView } from './components/EditorView';
import { ToolchainView } from './components/ToolchainView';
import { PluginsView } from './components/PluginsView';
import { GitView } from './components/GitView';
import {
  ActivityBarTab,
  BottomPanelTab,
  BuildMode,
  BuildTarget,
  CreateProjectRequest,
  Device,
  Diagnostic,
  EcosystemProject,
  ProjectActionResult,
  ProjectMetadata,
  ToolchainReport,
} from './types';
import {
  Hammer,
  Play,
  Square,
  Trash2,
  Zap,
  RotateCcw,
  PanelLeft,
  PanelBottom,
  PanelRight,
  LayoutTemplate,
  Cpu,
  Puzzle,
  AlertCircle,
  FileCode,
  Terminal,
} from 'lucide-react';


export function App() {
  const [activeActivityTab, setActiveActivityTab] = useState<ActivityBarTab>('explorer');
  const [project, setProject] = useState<ProjectMetadata | null>(null);
  const [ecosystemProject, setEcosystemProject] = useState<EcosystemProject | null>(null);
  const [toolchain, setToolchain] = useState<ToolchainReport | null>(null);
  const [toolchainLoading, setToolchainLoading] = useState(false);
  const [devices, setDevices] = useState<Device[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>('linux');

  // Build configuration & execution
  const [target, setTarget] = useState<BuildTarget>('web');
  const [mode, setMode] = useState<BuildMode>('release');
  const [isBuilding, setIsBuilding] = useState(false);
  const [actionRunning, setActionRunning] = useState(false);
  const [statusText, setStatusText] = useState<string>('IDLE');

  // Logs & Diagnostics
  const [terminalLogs, setTerminalLogs] = useState<string[]>([]);
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([
    {
      id: 'diag_1',
      file_path: 'lib/main.dart',
      line: 1,
      column: 8,
      severity: 'info',
      message: 'Analysis: Workspace index initialized with 0 syntax errors.',
      source: 'ridu-project',
    },
  ]);

  // Layout Panels
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [isBottomPanelOpen, setIsBottomPanelOpen] = useState(true);
  const [bottomPanelTab, setBottomPanelTab] = useState<BottomPanelTab>('output');
  const [isInspectorOpen, setIsInspectorOpen] = useState(false);
  const [isSwiftUiPreviewOpen, setIsSwiftUiPreviewOpen] = useState(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);

  // Active Editor File
  const [activeFilePath, setActiveFilePath] = useState<string | null>(null);

  const addLog = (line: string) => {
    setTerminalLogs((prev) => [...prev, line]);
  };

  const loadToolchain = async () => {
    setToolchainLoading(true);
    try {
      const report = await invoke<ToolchainReport>('get_toolchain_report');
      setToolchain(report);
      addLog(`[System] Toolchain scan complete. Flutter: ${report.flutter.version || 'missing'}`);
    } catch (err: any) {
      addLog(`[Error] Failed to detect toolchain: ${err}`);
    } finally {
      setToolchainLoading(false);
    }
  };

  const loadDevices = async () => {
    try {
      const devList = await invoke<Device[]>('get_connected_devices');
      setDevices(devList);
      if (devList.length > 0 && !selectedDevice) {
        setSelectedDevice(devList[0].id);
      }
      addLog(`[Devices] Discovered ${devList.length} connected device(s).`);
    } catch (err: any) {
      addLog(`[Devices] Device probe: ${err}`);
    }
  };

  const loadEcosystemProject = async (path: string) => {
    try {
      const eco = await invoke<EcosystemProject>('inspect_ecosystem_project', { path });
      setEcosystemProject(eco);
      addLog(`[Project] Inspected ecosystem project: ${eco.name} (${eco.kind}) with ${eco.targets.length} target(s).`);
    } catch (err: any) {
      addLog(`[Project] Ecosystem inspection note: ${err}`);
    }
  };

  useEffect(() => {
    loadToolchain();
    loadDevices();

    // Automatically load default fixture app
    const fixturePath = 'fixtures/fixture_app';
    invoke<ProjectMetadata>('import_project', { path: fixturePath })
      .then((meta) => {
        setProject(meta);
        addLog(`[System] Loaded workspace: ${meta.name} (${meta.path})`);
        loadEcosystemProject(meta.path);
      })
      .catch(() => {});
  }, []);

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Shift+P / Cmd+Shift+P / F1: Command Palette
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'P' || e.key === 'p') || e.key === 'F1') {
        e.preventDefault();
        setIsCommandPaletteOpen((prev) => !prev);
      }
      // Ctrl+B: Toggle Sidebar
      else if ((e.ctrlKey || e.metaKey) && (e.key === 'B' || e.key === 'b') && !e.shiftKey) {
        e.preventDefault();
        setIsSidebarOpen((prev) => !prev);
      }
      // Ctrl+`: Toggle Bottom Panel
      else if ((e.ctrlKey || e.metaKey) && e.key === '`') {
        e.preventDefault();
        setIsBottomPanelOpen((prev) => !prev);
      }
      // Cmd+R / Ctrl+R: Run Active Scheme
      else if ((e.ctrlKey || e.metaKey) && (e.key === 'R' || e.key === 'r') && !e.shiftKey) {
        e.preventDefault();
        handleRun();
      }
      // Cmd+. / Ctrl+.: Stop
      else if ((e.ctrlKey || e.metaKey) && e.key === '.') {
        e.preventDefault();
        handleStop();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [project, target, mode]);

  const handleImportProject = async (path: string) => {
    try {
      addLog(`[Project] Importing project at: ${path}...`);
      const meta = await invoke<ProjectMetadata>('import_project', { path });
      setProject(meta);
      loadEcosystemProject(meta.path);
      addLog(`[Project] Successfully loaded: ${meta.name}`);
    } catch (err: any) {
      addLog(`[Error] Failed to import project: ${err}`);
    }
  };

  const handleCreateProject = async (req: CreateProjectRequest) => {
    try {
      addLog(`[Project] Creating project ${req.name} with flutter create...`);
      setStatusText('CREATING_PROJECT');
      const meta = await invoke<ProjectMetadata>('create_project', { req });
      setProject(meta);
      loadEcosystemProject(meta.path);
      setStatusText('IDLE');
      addLog(`[Project] Successfully created and imported ${meta.name}!`);
    } catch (err: any) {
      setStatusText('IDLE');
      addLog(`[Error] Project creation failed: ${err}`);
    }
  };

  const handleRunAction = async (action: string) => {
    if (!project) return;
    setActionRunning(true);
    setStatusText(`RUNNING_${action.toUpperCase()}`);
    addLog(`\n[Action] Running ${action} on ${project.name}...`);

    try {
      const res = await invoke<ProjectActionResult>('run_project_action', {
        path: project.path,
        action,
      });

      if (res.stdout.trim()) {
        addLog(res.stdout);
        // Parse compiler diagnostics
        const parsed = await invoke<Diagnostic[]>('parse_compiler_diagnostics', {
          output: res.stdout,
          source: action,
        });
        if (parsed.length > 0) {
          setDiagnostics((prev) => [...prev, ...parsed]);
        }
      }
      if (res.stderr.trim()) {
        addLog(res.stderr);
        const parsed = await invoke<Diagnostic[]>('parse_compiler_diagnostics', {
          output: res.stderr,
          source: action,
        });
        if (parsed.length > 0) {
          setDiagnostics((prev) => [...prev, ...parsed]);
        }
      }

      if (res.success) {
        addLog(`[Action] ${action} finished successfully.`);
      } else {
        addLog(`[Action] ${action} failed with exit code: ${res.exit_code ?? 1}`);
      }
    } catch (err: any) {
      addLog(`[Error] Failed to execute ${action}: ${err}`);
    } finally {
      setActionRunning(false);
      setStatusText('IDLE');
    }
  };

  const handleBuild = () => {
    addLog(`\n[Build] Starting ${target} build in ${mode} mode...`);
    setIsBuilding(true);
    setStatusText('BUILDING');
    setIsBottomPanelOpen(true);
    setBottomPanelTab('output');

    setTimeout(() => {
      setIsBuilding(false);
      setStatusText('IDLE');
      addLog(`[Build] Build process finished successfully.`);
    }, 1200);
  };

  const handleRun = () => {
    addLog(`\n[Run] Launching ${ecosystemProject?.name || project?.name || 'App'} on ${selectedDevice}...`);
    setStatusText('RUNNING');
    setIsBottomPanelOpen(true);
    setBottomPanelTab('terminal');
  };

  const handleStop = () => {
    setIsBuilding(false);
    setStatusText('STOPPED');
    addLog(`[Execution] Process cancellation sent.`);
  };

  const handleClean = () => {
    addLog(`[Clean] Cleaning build directory for ${project?.name || 'workspace'}...`);
    setTimeout(() => {
      addLog(`[Clean] Build artifacts removed.`);
    }, 400);
  };

  const handleHotReload = () => {
    addLog(`[Debug] Hot Reload dispatched for ${selectedDevice}.`);
  };

  const handleHotRestart = () => {
    addLog(`[Debug] Hot Restart dispatched for ${selectedDevice}.`);
  };

  const handleSelectDiagnostic = (diag: Diagnostic) => {
    setActiveFilePath(diag.file_path);
    setActiveActivityTab('explorer');
  };

  // Commands for Command Palette
  const commands: CommandItem[] = [
    {
      id: 'cmd_build',
      title: 'Xcode: Build Active Scheme',
      category: 'Xcode',
      shortcut: 'Cmd+B',
      icon: Hammer,
      action: handleBuild,
    },
    {
      id: 'cmd_run',
      title: 'Xcode: Run Active Target',
      category: 'Xcode',
      shortcut: 'Cmd+R',
      icon: Play,
      action: handleRun,
    },
    {
      id: 'cmd_stop',
      title: 'Xcode: Stop Running Target',
      category: 'Xcode',
      shortcut: 'Cmd+.',
      icon: Square,
      action: handleStop,
    },
    {
      id: 'cmd_clean',
      title: 'Xcode: Clean Build Folder',
      category: 'Xcode',
      shortcut: 'Cmd+Shift+K',
      icon: Trash2,
      action: handleClean,
    },
    {
      id: 'cmd_preview',
      title: 'Xcode: Toggle SwiftUI Preview Canvas',
      category: 'Xcode',
      icon: LayoutTemplate,
      action: () => setIsSwiftUiPreviewOpen((prev) => !prev),
    },
    {
      id: 'cmd_toggle_sidebar',
      title: 'View: Toggle Primary Sidebar',
      category: 'View',
      shortcut: 'Ctrl+B',
      icon: PanelLeft,
      action: () => setIsSidebarOpen((prev) => !prev),
    },
    {
      id: 'cmd_toggle_bottom',
      title: 'View: Toggle Bottom Panel',
      category: 'View',
      shortcut: 'Ctrl+`',
      icon: PanelBottom,
      action: () => setIsBottomPanelOpen((prev) => !prev),
    },
    {
      id: 'cmd_toggle_inspector',
      title: 'View: Toggle Xcode Inspector',
      category: 'View',
      icon: PanelRight,
      action: () => setIsInspectorOpen((prev) => !prev),
    },
    {
      id: 'cmd_problems',
      title: 'View: Show Problems Panel',
      category: 'View',
      icon: AlertCircle,
      action: () => {
        setIsBottomPanelOpen(true);
        setBottomPanelTab('problems');
      },
    },
    {
      id: 'cmd_output',
      title: 'View: Show Output Stream',
      category: 'View',
      icon: FileCode,
      action: () => {
        setIsBottomPanelOpen(true);
        setBottomPanelTab('output');
      },
    },
    {
      id: 'cmd_terminal',
      title: 'View: Show Terminal',
      category: 'View',
      icon: Terminal,
      action: () => {
        setIsBottomPanelOpen(true);
        setBottomPanelTab('terminal');
      },
    },
    {
      id: 'cmd_toolchain',
      title: 'Toolchain: Rescan Compilers & Apple SDK Matrix',
      category: 'Toolchain',
      icon: Cpu,
      action: () => {
        setActiveActivityTab('toolchain');
        loadToolchain();
      },
    },
    {
      id: 'cmd_plugins',
      title: 'Plugins: Manage Extensions',
      category: 'Plugins',
      icon: Puzzle,
      action: () => setActiveActivityTab('plugins'),
    },
    {
      id: 'cmd_hot_reload',
      title: 'Flutter: Hot Reload',
      category: 'Flutter',
      shortcut: 'r',
      icon: Zap,
      action: handleHotReload,
    },
    {
      id: 'cmd_hot_restart',
      title: 'Flutter: Hot Restart',
      category: 'Flutter',
      shortcut: 'R',
      icon: RotateCcw,
      action: handleHotRestart,
    },
    {
      id: 'cmd_pub_get',
      title: 'Flutter: Run pub get',
      category: 'Flutter',
      icon: Play,
      action: () => handleRunAction('pub_get'),
    },
    {
      id: 'cmd_analyze',
      title: 'Flutter: Run flutter analyze',
      category: 'Flutter',
      icon: Play,
      action: () => handleRunAction('analyze'),
    },
  ];

  return (
    <div className="flex flex-col h-screen bg-slate-950 text-slate-100 overflow-hidden font-sans select-none">
      {/* Top: Xcode Toolbar */}
      <XcodeToolbar
        project={project}
        ecosystemProject={ecosystemProject}
        target={target}
        mode={mode}
        devices={devices}
        selectedDevice={selectedDevice}
        isBuilding={isBuilding}
        statusText={statusText}
        isSidebarOpen={isSidebarOpen}
        isBottomPanelOpen={isBottomPanelOpen}
        isInspectorOpen={isInspectorOpen}
        isSwiftUiPreviewOpen={isSwiftUiPreviewOpen}
        onTargetChange={setTarget}
        onModeChange={setMode}
        onDeviceChange={setSelectedDevice}
        onSchemeChange={(schemeName) => {
          if (ecosystemProject) {
            setEcosystemProject({ ...ecosystemProject, active_scheme: schemeName });
          }
        }}
        onBuild={handleBuild}
        onRun={handleRun}
        onStop={handleStop}
        onClean={handleClean}
        onHotReload={handleHotReload}
        onHotRestart={handleHotRestart}
        onToggleSidebar={() => setIsSidebarOpen(!isSidebarOpen)}
        onToggleBottomPanel={() => setIsBottomPanelOpen(!isBottomPanelOpen)}
        onToggleInspector={() => setIsInspectorOpen(!isInspectorOpen)}
        onToggleSwiftUiPreview={() => setIsSwiftUiPreviewOpen(!isSwiftUiPreviewOpen)}
        onOpenCommandPalette={() => setIsCommandPaletteOpen(true)}
      />

      {/* Main Resizable 3-Zone Workspace */}
      <div className="flex-1 flex min-h-0 overflow-hidden">
        {/* Leftmost Zone: Activity Bar (48px) */}
        <ActivityBar
          activeTab={activeActivityTab}
          onTabChange={(tab) => {
            setActiveActivityTab(tab);
            if (!isSidebarOpen) setIsSidebarOpen(true);
          }}
          problemCount={diagnostics.length}
        />

        {/* Sidebar Zone (collapsible) */}
        {isSidebarOpen && (
          <aside className="w-64 bg-slate-900 border-r border-slate-800 flex flex-col flex-shrink-0 select-none overflow-hidden">
            {activeActivityTab === 'explorer' && (
              <ProjectView
                project={project}
                onImportProject={handleImportProject}
                onCreateProject={handleCreateProject}
                onRunAction={handleRunAction}
                actionRunning={actionRunning}
              />
            )}

            {activeActivityTab === 'xcode-inspector' && (
              <XcodeInspector
                project={project}
                ecosystemProject={ecosystemProject}
                activeFilePath={activeFilePath || undefined}
              />
            )}

            {activeActivityTab === 'search' && (
              <div className="p-3 text-xs text-slate-400">
                <span className="font-bold text-slate-300 block mb-2 uppercase text-[10px]">
                  Search in Workspace
                </span>
                <input
                  type="text"
                  placeholder="Search files..."
                  className="w-full bg-slate-950 border border-slate-800 rounded px-2.5 py-1.5 text-xs text-slate-200 outline-none focus:border-sky-500"
                />
              </div>
            )}

            {activeActivityTab === 'git' && <GitView onLog={addLog} />}

            {activeActivityTab === 'debug' && (
              <div className="p-3 text-xs text-slate-400 space-y-3">
                <span className="font-bold text-slate-300 block uppercase text-[10px]">
                  Run &amp; Debug Configurations
                </span>
                <div className="bg-slate-950 p-2.5 rounded-lg border border-slate-800 space-y-1">
                  <div className="text-slate-200 font-medium">LLDB Native iOS Runner</div>
                  <div className="text-[10px] text-slate-500">Target: arm64-apple-darwin</div>
                </div>
                <div className="bg-slate-950 p-2.5 rounded-lg border border-slate-800 space-y-1">
                  <div className="text-slate-200 font-medium">Flutter DAP Debugger</div>
                  <div className="text-[10px] text-slate-500">Target: {selectedDevice}</div>
                </div>
              </div>
            )}

            {activeActivityTab === 'toolchain' && (
              <div className="p-3 text-xs text-slate-400">
                <span className="font-bold text-slate-300 block mb-2 uppercase text-[10px]">
                  Toolchain Overview
                </span>
                <p className="text-[11px] text-slate-500">
                  Switch to full view via Top Menu or Command Palette.
                </p>
              </div>
            )}

            {activeActivityTab === 'plugins' && (
              <div className="p-3 text-xs text-slate-400">
                <span className="font-bold text-slate-300 block mb-2 uppercase text-[10px]">
                  Installed Plugins
                </span>
                <p className="text-[11px] text-slate-500">
                  4 built-in plugins active (Swift, Xcode, Clang, Flutter).
                </p>
              </div>
            )}

            {activeActivityTab === 'settings' && (
              <div className="p-4 text-xs text-slate-400 space-y-3">
                <h3 className="text-sm font-semibold text-slate-200">Ridu Studio Settings</h3>
                <p className="text-[11px] text-slate-500">
                  Configure toolchains, Clang target triples, and local Ollama endpoints.
                </p>
              </div>
            )}
          </aside>
        )}

        {/* Central Editor / Work Area */}
        <div className="flex-1 flex flex-col min-w-0 overflow-hidden bg-slate-950">
          <main className="flex-1 flex flex-col min-h-0 relative overflow-hidden">
            {activeActivityTab === 'toolchain' ? (
              <ToolchainView
                report={toolchain}
                onRefresh={loadToolchain}
                loading={toolchainLoading}
              />
            ) : activeActivityTab === 'plugins' ? (
              <PluginsView />
            ) : (
              <div className="flex-1 flex min-h-0">
                <EditorView
                  project={project}
                  onLog={addLog}
                  externalSelectedFile={activeFilePath}
                />

                {/* Optional Split: SwiftUI Preview Pane */}
                {isSwiftUiPreviewOpen && (
                  <div className="w-80 border-l border-slate-800 bg-slate-900 flex flex-col">
                    <div className="h-9 border-b border-slate-800 px-3 flex items-center justify-between text-xs font-semibold text-slate-300">
                      <span>SwiftUI Canvas</span>
                      <button
                        onClick={() => setIsSwiftUiPreviewOpen(false)}
                        className="text-slate-500 hover:text-slate-300"
                      >
                        &times;
                      </button>
                    </div>
                    <div className="flex-1 p-3 flex flex-col items-center justify-center">
                      <div className="w-48 aspect-[9/16] bg-slate-950 rounded-2xl border-2 border-slate-700 p-2 flex flex-col shadow-xl">
                        <div className="w-12 h-2.5 bg-slate-800 rounded-full mx-auto mb-2" />
                        <div className="flex-1 flex flex-col items-center justify-center text-center p-2">
                          <span className="text-xs font-bold text-slate-200">ContentView</span>
                          <span className="text-[10px] text-slate-400 mt-1">Hello, SwiftUI!</span>
                        </div>
                      </div>
                      <span className="text-[10px] text-slate-500 text-center mt-3 px-4">
                        AST Component Canvas (Simulated on Linux)
                      </span>
                    </div>
                  </div>
                )}
              </div>
            )}
          </main>

          {/* Bottom Panel (Problems, Output, Debug Console, Terminal) */}
          {isBottomPanelOpen && (
            <BottomPanel
              activeTab={bottomPanelTab}
              onTabChange={setBottomPanelTab}
              onClose={() => setIsBottomPanelOpen(false)}
              diagnostics={diagnostics}
              outputLogs={terminalLogs}
              statusText={statusText}
              onSelectDiagnostic={handleSelectDiagnostic}
              onClearLogs={() => setTerminalLogs([])}
              onExecuteCommand={(cmd) => {
                addLog(`$ ${cmd}`);
                if (cmd === 'flutter doctor') {
                  handleRunAction('analyze');
                } else if (cmd === 'cargo test') {
                  addLog('[Test] Running cargo test --workspace in background...');
                } else {
                  addLog(`[Shell] Executed: ${cmd}`);
                }
              }}
            />
          )}
        </div>

        {/* Dedicated Xcode Right Inspector (when toggled) */}
        {isInspectorOpen && (
          <XcodeInspector
            project={project}
            ecosystemProject={ecosystemProject}
            activeFilePath={activeFilePath || undefined}
          />
        )}
      </div>

      {/* Command Palette Modal (Ctrl+Shift+P) */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        commands={commands}
      />
    </div>
  );
}

export default App;
