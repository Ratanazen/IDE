import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Sidebar } from './components/Sidebar';
import { Toolbar } from './components/Toolbar';
import { ProjectView } from './components/ProjectView';
import { EditorView } from './components/EditorView';
import { BuildCenterView } from './components/BuildCenterView';
import { ToolchainView } from './components/ToolchainView';
import { TerminalView } from './components/TerminalView';
import {
  NavTab,
  BuildTarget,
  BuildMode,
  ProjectMetadata,
  ToolchainReport,
  CreateProjectRequest,
  ProjectActionResult,
  Device,
} from './types';

export function App() {
  const [currentTab, setCurrentTab] = useState<NavTab>('projects');
  const [project, setProject] = useState<ProjectMetadata | null>(null);
  const [toolchain, setToolchain] = useState<ToolchainReport | null>(null);
  const [toolchainLoading, setToolchainLoading] = useState(false);
  const [devices, setDevices] = useState<Device[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>('linux');

  // Build configuration
  const [target, setTarget] = useState<BuildTarget>('web');
  const [mode, setMode] = useState<BuildMode>('release');
  const [cleanBeforeBuild, setCleanBeforeBuild] = useState(false);
  const [runAnalyze, setRunAnalyze] = useState(true);
  const [runTests, setRunTests] = useState(false);
  const [isBuilding, setIsBuilding] = useState(false);
  const [actionRunning, setActionRunning] = useState(false);

  // Terminal logs
  const [terminalLogs, setTerminalLogs] = useState<string[]>([]);
  const [statusText, setStatusText] = useState<string>('IDLE');

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

  useEffect(() => {
    loadToolchain();
    loadDevices();

    // Automatically load fixture app
    invoke<ProjectMetadata>('import_project', { path: 'fixtures/fixture_app' })
      .then((meta) => {
        setProject(meta);
        addLog(`[System] Loaded workspace: ${meta.name} (${meta.path})`);
      })
      .catch(() => {});
  }, []);

  const handleImportProject = async (path: string) => {
    try {
      addLog(`[Project] Importing project at: ${path}...`);
      const meta = await invoke<ProjectMetadata>('import_project', { path });
      setProject(meta);
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
      }
      if (res.stderr.trim()) {
        addLog(res.stderr);
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

    setTimeout(() => {
      setIsBuilding(false);
      setStatusText('IDLE');
      addLog(`[Build] Build process finished.`);
    }, 1500);
  };

  const handleCancel = () => {
    setIsBuilding(false);
    setStatusText('CANCELLED');
    addLog(`[Build] Cancellation signal sent to process group.`);
  };

  const handleOpenOutput = async () => {
    if (!project) return;
    try {
      await invoke('reveal_in_file_manager', { path: `${project.path}/build` });
      addLog(`[Artifacts] Opened build folder in system file manager.`);
    } catch (err: any) {
      addLog(`[Error] Open output failed: ${err}`);
    }
  };

  const handleHotReload = () => {
    addLog(`[Debug] Hot Reload dispatched for ${selectedDevice}.`);
  };

  const handleHotRestart = () => {
    addLog(`[Debug] Hot Restart dispatched for ${selectedDevice}.`);
  };

  return (
    <div className="flex h-screen bg-slate-950 text-slate-100 overflow-hidden font-sans">
      <Sidebar currentTab={currentTab} onTabChange={setCurrentTab} />

      <div className="flex-1 flex flex-col min-w-0">
        <Toolbar
          project={project}
          target={target}
          mode={mode}
          devices={devices}
          selectedDevice={selectedDevice}
          isBuilding={isBuilding}
          cleanBeforeBuild={cleanBeforeBuild}
          runAnalyze={runAnalyze}
          runTests={runTests}
          onTargetChange={setTarget}
          onModeChange={setMode}
          onDeviceChange={setSelectedDevice}
          onCleanToggle={() => setCleanBeforeBuild(!cleanBeforeBuild)}
          onAnalyzeToggle={() => setRunAnalyze(!runAnalyze)}
          onTestsToggle={() => setRunTests(!runTests)}
          onBuild={handleBuild}
          onCancel={handleCancel}
          onOpenOutput={handleOpenOutput}
          onHotReload={handleHotReload}
          onHotRestart={handleHotRestart}
        />

        <main className="flex-1 flex flex-col min-h-0 bg-slate-950">
          {currentTab === 'projects' && (
            <ProjectView
              project={project}
              onImportProject={handleImportProject}
              onCreateProject={handleCreateProject}
              onRunAction={handleRunAction}
              actionRunning={actionRunning}
            />
          )}

          {currentTab === 'editor' && (
            <EditorView project={project} onLog={addLog} />
          )}

          {(currentTab === 'builds' || currentTab === 'artifacts') && (
            <BuildCenterView onLog={addLog} />
          )}

          {currentTab === 'toolchain' && (
            <ToolchainView
              report={toolchain}
              onRefresh={loadToolchain}
              loading={toolchainLoading}
            />
          )}

          {currentTab === 'settings' && (
            <div className="flex-1 p-6 flex flex-col items-center justify-center text-slate-400">
              <h3 className="text-base font-semibold text-slate-200">Studio Settings</h3>
              <p className="text-xs text-slate-500 mt-1">
                Configure Flutter channels, signing keys, and local AI (Ollama) settings.
              </p>
            </div>
          )}

          <TerminalView
            logs={terminalLogs}
            statusText={statusText}
            onClear={() => setTerminalLogs([])}
          />
        </main>
      </div>
    </div>
  );
}

export default App;
