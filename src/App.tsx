import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Sidebar } from './components/Sidebar';
import { Toolbar } from './components/Toolbar';
import { ProjectView } from './components/ProjectView';
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
} from './types';

export function App() {
  const [currentTab, setCurrentTab] = useState<NavTab>('projects');
  const [project, setProject] = useState<ProjectMetadata | null>(null);
  const [toolchain, setToolchain] = useState<ToolchainReport | null>(null);
  const [toolchainLoading, setToolchainLoading] = useState(false);

  // Build configuration
  const [target, setTarget] = useState<BuildTarget>('web');
  const [mode, setMode] = useState<BuildMode>('release');
  const [cleanBeforeBuild, setCleanBeforeBuild] = useState(false);
  const [runAnalyze, setRunAnalyze] = useState(true);
  const [runTests, setRunTests] = useState(false);
  const [isBuilding] = useState(false);
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

  // On mount, scan toolchain and try loading fixtures/fixture_app if it exists
  useEffect(() => {
    loadToolchain();

    // Check if fixture app exists in the repo
    invoke<ProjectMetadata>('import_project', { path: 'fixtures/fixture_app' })
      .then((meta) => {
        setProject(meta);
        addLog(`[System] Automatically loaded fixture app: ${meta.name}`);
      })
      .catch(() => {
        // No fixture loaded yet, that's okay
      });
  }, []);

  const handleImportProject = async (path: string) => {
    try {
      addLog(`[Project] Importing project at: ${path}...`);
      const meta = await invoke<ProjectMetadata>('import_project', { path });
      setProject(meta);
      addLog(`[Project] Successfully loaded: ${meta.name} (platforms: ${meta.platforms.join(', ')})`);
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
        addLog(`[Action] ${action} finished successfully (exit code: ${res.exit_code ?? 0})`);
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
    addLog(`[Build] Target ${target} in ${mode} mode dispatched.`);
  };

  const handleCancel = () => {
    addLog(`[Build] Cancellation requested.`);
  };

  const handleOpenOutput = () => {
    addLog(`[Artifacts] Open output directory triggered.`);
  };

  return (
    <div className="flex h-screen bg-slate-950 text-slate-100 overflow-hidden font-sans">
      <Sidebar currentTab={currentTab} onTabChange={setCurrentTab} />

      <div className="flex-1 flex flex-col min-w-0">
        <Toolbar
          project={project}
          target={target}
          mode={mode}
          isBuilding={isBuilding}
          cleanBeforeBuild={cleanBeforeBuild}
          runAnalyze={runAnalyze}
          runTests={runTests}
          onTargetChange={setTarget}
          onModeChange={setMode}
          onCleanToggle={() => setCleanBeforeBuild(!cleanBeforeBuild)}
          onAnalyzeToggle={() => setRunAnalyze(!runAnalyze)}
          onTestsToggle={() => setRunTests(!runTests)}
          onBuild={handleBuild}
          onCancel={handleCancel}
          onOpenOutput={handleOpenOutput}
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

          {currentTab === 'toolchain' && (
            <ToolchainView
              report={toolchain}
              onRefresh={loadToolchain}
              loading={toolchainLoading}
            />
          )}

          {currentTab === 'builds' && (
            <div className="flex-1 p-6 flex flex-col items-center justify-center text-slate-400">
              <h3 className="text-base font-semibold text-slate-200">Build Center</h3>
              <p className="text-xs text-slate-500 mt-1">
                Configure build targets above and trigger real builds with streaming logs.
              </p>
            </div>
          )}

          {currentTab === 'artifacts' && (
            <div className="flex-1 p-6 flex flex-col items-center justify-center text-slate-400">
              <h3 className="text-base font-semibold text-slate-200">Artifact Browser</h3>
              <p className="text-xs text-slate-500 mt-1">
                Discovered and verified release binaries will appear here with checksums.
              </p>
            </div>
          )}

          {currentTab === 'settings' && (
            <div className="flex-1 p-6 flex flex-col items-center justify-center text-slate-400">
              <h3 className="text-base font-semibold text-slate-200">Studio Settings</h3>
              <p className="text-xs text-slate-500 mt-1">
                Configure Flutter channels, signing keys, and Ollama AI parameters.
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
