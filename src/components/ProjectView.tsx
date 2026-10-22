import React, { useState } from 'react';
import { FolderPlus, FolderOpen, PackageCheck, Cpu } from 'lucide-react';
import { ProjectMetadata, CreateProjectRequest } from '../types';

interface ProjectViewProps {
  project: ProjectMetadata | null;
  onImportProject: (path: string) => void;
  onCreateProject: (req: CreateProjectRequest) => void;
  onRunAction: (action: string) => void;
  actionRunning: boolean;
}

export const ProjectView: React.FC<ProjectViewProps> = ({
  project,
  onImportProject,
  onCreateProject,
  onRunAction,
  actionRunning,
}) => {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);

  // Create form state
  const [newName, setNewName] = useState('');
  const [newOrg, setNewOrg] = useState('com.example');
  const [newParentDir, setNewParentDir] = useState('');
  const [selectedPlatforms, setSelectedPlatforms] = useState<string[]>(['web', 'linux', 'android']);

  // Import form state
  const [importPath, setImportPath] = useState('');

  const togglePlatform = (p: string) => {
    if (selectedPlatforms.includes(p)) {
      setSelectedPlatforms(selectedPlatforms.filter((item) => item !== p));
    } else {
      setSelectedPlatforms([...selectedPlatforms, p]);
    }
  };

  const handleCreateSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim() || !newParentDir.trim()) return;
    onCreateProject({
      name: newName.trim(),
      org: newOrg.trim() || undefined,
      parent_dir: newParentDir.trim(),
      platforms: selectedPlatforms,
    });
    setShowCreateModal(false);
  };

  const handleImportSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!importPath.trim()) return;
    onImportProject(importPath.trim());
    setShowImportModal(false);
  };

  return (
    <div className="flex-1 flex flex-col p-6 overflow-y-auto">
      {/* Top Header Actions */}
      <div className="flex justify-between items-center mb-6">
        <div>
          <h2 className="text-xl font-bold text-slate-100">Flutter Projects</h2>
          <p className="text-xs text-slate-400">Manage, create, and inspect real Flutter applications</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowImportModal(true)}
            className="flex items-center gap-2 px-3 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-lg border border-slate-700 transition-all cursor-pointer"
          >
            <FolderOpen className="w-4 h-4 text-sky-400" />
            <span>Open Existing Project</span>
          </button>
          <button
            onClick={() => setShowCreateModal(true)}
            className="flex items-center gap-2 px-3 py-2 bg-sky-500 hover:bg-sky-400 text-slate-950 text-xs font-semibold rounded-lg shadow transition-all cursor-pointer"
          >
            <FolderPlus className="w-4 h-4" />
            <span>New Flutter Project</span>
          </button>
        </div>
      </div>

      {project ? (
        <div className="space-y-6">
          {/* Project Overview Card */}
          <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-sm">
            <div className="flex justify-between items-start">
              <div>
                <h3 className="text-lg font-bold text-slate-100 flex items-center gap-2">
                  <span>{project.name}</span>
                  {project.version && (
                    <span className="text-xs bg-slate-800 text-slate-400 px-2 py-0.5 rounded-full border border-slate-700">
                      v{project.version}
                    </span>
                  )}
                </h3>
                <p className="text-xs text-slate-400 mt-1">{project.description || 'No description provided'}</p>
                <p className="text-[11px] font-mono text-slate-500 mt-2">{project.path}</p>
              </div>

              {/* Action Buttons */}
              <div className="flex flex-wrap gap-2">
                <button
                  onClick={() => onRunAction('pub_get')}
                  disabled={actionRunning}
                  className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-lg border border-slate-700 transition-all cursor-pointer disabled:opacity-50"
                >
                  pub get
                </button>
                <button
                  onClick={() => onRunAction('analyze')}
                  disabled={actionRunning}
                  className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-lg border border-slate-700 transition-all cursor-pointer disabled:opacity-50"
                >
                  analyze
                </button>
                <button
                  onClick={() => onRunAction('format')}
                  disabled={actionRunning}
                  className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-lg border border-slate-700 transition-all cursor-pointer disabled:opacity-50"
                >
                  format
                </button>
                <button
                  onClick={() => onRunAction('test')}
                  disabled={actionRunning}
                  className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-lg border border-slate-700 transition-all cursor-pointer disabled:opacity-50"
                >
                  test
                </button>
              </div>
            </div>

            {/* Platforms */}
            <div className="mt-5 pt-4 border-t border-slate-800">
              <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
                Configured Platforms:
              </span>
              <div className="flex gap-2 mt-2">
                {project.platforms.map((plat) => (
                  <span
                    key={plat}
                    className="px-2.5 py-1 text-xs font-medium rounded-md bg-sky-500/10 text-sky-400 border border-sky-500/30 uppercase"
                  >
                    {plat}
                  </span>
                ))}
              </div>
            </div>
          </div>

          {/* Dependencies Lists */}
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
              <h4 className="text-xs font-semibold text-slate-300 mb-3 flex items-center gap-2">
                <PackageCheck className="w-4 h-4 text-sky-400" />
                <span>Dependencies ({project.dependencies.length})</span>
              </h4>
              <div className="max-h-48 overflow-y-auto space-y-1">
                {project.dependencies.map((dep) => (
                  <div key={dep} className="text-xs font-mono text-slate-400 bg-slate-950/40 px-2 py-1 rounded">
                    {dep}
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-slate-900 border border-slate-800 rounded-xl p-4">
              <h4 className="text-xs font-semibold text-slate-300 mb-3 flex items-center gap-2">
                <Cpu className="w-4 h-4 text-slate-400" />
                <span>Dev Dependencies ({project.dev_dependencies.length})</span>
              </h4>
              <div className="max-h-48 overflow-y-auto space-y-1">
                {project.dev_dependencies.map((dep) => (
                  <div key={dep} className="text-xs font-mono text-slate-400 bg-slate-950/40 px-2 py-1 rounded">
                    {dep}
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-slate-800 rounded-2xl p-12 text-center">
          <FolderOpen className="w-12 h-12 text-slate-600 mb-4" />
          <h3 className="text-base font-semibold text-slate-200">No Project Open</h3>
          <p className="text-xs text-slate-500 max-w-sm mt-1">
            Open an existing Flutter project or create a new one using the real Flutter CLI to get started.
          </p>
        </div>
      )}

      {/* Modal: Create Project */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-slate-950/80 backdrop-blur-xs flex items-center justify-center z-50 p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl w-full max-w-md p-6 shadow-2xl">
            <h3 className="text-base font-bold text-slate-100 mb-4">Create Flutter Project</h3>
            <form onSubmit={handleCreateSubmit} className="space-y-4">
              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">Project Name</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. my_cool_app"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-xs text-slate-200 outline-none focus:border-sky-500 font-mono"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">Organization</label>
                <input
                  type="text"
                  placeholder="com.example"
                  value={newOrg}
                  onChange={(e) => setNewOrg(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-xs text-slate-200 outline-none focus:border-sky-500 font-mono"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">Parent Directory</label>
                <input
                  type="text"
                  required
                  placeholder="/home/reny/Documents/..."
                  value={newParentDir}
                  onChange={(e) => setNewParentDir(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-xs text-slate-200 outline-none focus:border-sky-500 font-mono"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-2">Target Platforms</label>
                <div className="flex gap-3">
                  {['web', 'linux', 'android'].map((plat) => (
                    <label key={plat} className="flex items-center gap-1.5 text-xs text-slate-300 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={selectedPlatforms.includes(plat)}
                        onChange={() => togglePlatform(plat)}
                        className="rounded bg-slate-950 border-slate-700 text-sky-500"
                      />
                      <span className="capitalize">{plat}</span>
                    </label>
                  ))}
                </div>
              </div>

              <div className="flex justify-end gap-2 pt-4 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="px-3 py-2 text-xs text-slate-400 hover:text-slate-200"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-sky-500 hover:bg-sky-400 text-slate-950 text-xs font-semibold rounded-lg"
                >
                  Run flutter create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal: Import Project */}
      {showImportModal && (
        <div className="fixed inset-0 bg-slate-950/80 backdrop-blur-xs flex items-center justify-center z-50 p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-xl w-full max-w-md p-6 shadow-2xl">
            <h3 className="text-base font-bold text-slate-100 mb-4">Open Existing Flutter Project</h3>
            <form onSubmit={handleImportSubmit} className="space-y-4">
              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">Project Directory Path</label>
                <input
                  type="text"
                  required
                  placeholder="/absolute/path/to/project"
                  value={importPath}
                  onChange={(e) => setImportPath(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-xs text-slate-200 outline-none focus:border-sky-500 font-mono"
                />
              </div>

              <div className="flex justify-end gap-2 pt-4 border-t border-slate-800">
                <button
                  type="button"
                  onClick={() => setShowImportModal(false)}
                  className="px-3 py-2 text-xs text-slate-400 hover:text-slate-200"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-sky-500 hover:bg-sky-400 text-slate-950 text-xs font-semibold rounded-lg"
                >
                  Open Project
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
