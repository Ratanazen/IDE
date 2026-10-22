import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  FileCode,
  Save,
  Folder,
  File,
  Check,
  X,
  ChevronRight,
  Code2,
} from 'lucide-react';
import { FileEntry, ProjectMetadata } from '../types';

interface OpenTab {
  path: string;
  name: string;
  content: string;
  originalContent: string;
}

interface EditorViewProps {
  project: ProjectMetadata | null;
  onLog: (line: string) => void;
  externalSelectedFile?: string | null;
}

export const EditorView: React.FC<EditorViewProps> = ({
  project,
  onLog,
  externalSelectedFile,
}) => {
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [openTabs, setOpenTabs] = useState<OpenTab[]>([]);
  const [activeTabPath, setActiveTabPath] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [savedSuccess, setSavedSuccess] = useState(false);
  const [subpath, setSubpath] = useState<string | null>(null);
  const [cursorPos, setCursorPos] = useState({ line: 1, col: 1 });

  const loadDirectory = async (sub?: string) => {
    if (!project) return;
    try {
      const entries = await invoke<FileEntry[]>('list_project_files', {
        projectPath: project.path,
        subpath: sub || null,
      });
      setFiles(entries);
    } catch (err: any) {
      onLog(`[Editor] Error listing files: ${err}`);
    }
  };

  useEffect(() => {
    if (project) {
      loadDirectory();
    }
  }, [project]);

  useEffect(() => {
    if (externalSelectedFile && (!activeTabPath || activeTabPath !== externalSelectedFile)) {
      openFileByPath(externalSelectedFile);
    }
  }, [externalSelectedFile]);

  const openFileByPath = async (filePath: string) => {
    const existing = openTabs.find((t) => t.path === filePath);
    if (existing) {
      setActiveTabPath(filePath);
      return;
    }

    try {
      const content = await invoke<string>('read_source_file', {
        filePath,
      });
      const name = filePath.split('/').pop() || 'Untitled';
      const newTab: OpenTab = {
        path: filePath,
        name,
        content,
        originalContent: content,
      };
      setOpenTabs((prev) => [...prev, newTab]);
      setActiveTabPath(filePath);
      onLog(`[Editor] Opened ${name}`);
    } catch (err: any) {
      onLog(`[Editor] Failed to open ${filePath}: ${err}`);
    }
  };

  const handleSelectFile = async (entry: FileEntry) => {
    if (entry.is_dir) {
      const nextSub = subpath ? `${subpath}/${entry.name}` : entry.name;
      setSubpath(nextSub);
      loadDirectory(nextSub);
    } else {
      openFileByPath(entry.path);
    }
  };

  const handleNavigateUp = () => {
    if (!subpath) return;
    const parts = subpath.split('/');
    parts.pop();
    const nextSub = parts.length > 0 ? parts.join('/') : null;
    setSubpath(nextSub);
    loadDirectory(nextSub || undefined);
  };

  const handleCloseTab = (e: React.MouseEvent, path: string) => {
    e.stopPropagation();
    const updated = openTabs.filter((t) => t.path !== path);
    setOpenTabs(updated);

    if (activeTabPath === path) {
      setActiveTabPath(updated.length > 0 ? updated[updated.length - 1].path : null);
    }
  };

  const activeTab = openTabs.find((t) => t.path === activeTabPath);

  const handleContentChange = (newContent: string) => {
    if (!activeTabPath) return;
    setOpenTabs((prev) =>
      prev.map((t) => (t.path === activeTabPath ? { ...t, content: newContent } : t))
    );
  };

  const handleSave = async () => {
    if (!activeTab) return;
    setSaving(true);
    try {
      await invoke('save_source_file', {
        filePath: activeTab.path,
        content: activeTab.content,
      });
      setOpenTabs((prev) =>
        prev.map((t) =>
          t.path === activeTab.path ? { ...t, originalContent: t.content } : t
        )
      );
      setSavedSuccess(true);
      setTimeout(() => setSavedSuccess(false), 2000);
      onLog(`[Editor] Saved ${activeTab.name}`);
    } catch (err: any) {
      onLog(`[Editor] Save error: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const isDirty = activeTab ? activeTab.content !== activeTab.originalContent : false;
  const lineCount = activeTab ? activeTab.content.split('\n').length : 0;

  const getLanguageName = (path?: string) => {
    if (!path) return 'Plain Text';
    const ext = path.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'swift':
        return 'Swift';
      case 'm':
        return 'Objective-C';
      case 'mm':
        return 'Objective-C++';
      case 'c':
        return 'C';
      case 'cpp':
      case 'cc':
        return 'C++';
      case 'h':
        return 'C/C++ Header';
      case 'dart':
        return 'Dart';
      case 'yaml':
      case 'yml':
        return 'YAML';
      case 'json':
        return 'JSON';
      case 'pbxproj':
        return 'Xcode PBX';
      case 'plist':
        return 'XML Plist';
      case 'md':
        return 'Markdown';
      default:
        return 'Plain Text';
    }
  };

  if (!project) {
    return (
      <div className="flex-1 flex items-center justify-center text-slate-500 text-xs bg-slate-950">
        Open a project to explore files and edit code.
      </div>
    );
  }

  const breadcrumbs = activeTab
    ? activeTab.path.replace(project.path, project.name).split('/')
    : [];

  return (
    <div className="flex-1 flex h-full overflow-hidden bg-slate-950 select-none">
      {/* File Explorer Tree */}
      <div className="w-60 bg-slate-900 border-r border-slate-800 flex flex-col flex-shrink-0">
        <div className="h-9 px-3 border-b border-slate-800 flex items-center justify-between text-xs font-semibold text-slate-300">
          <span>Project Explorer</span>
          {subpath && (
            <button
              onClick={handleNavigateUp}
              className="text-[11px] text-sky-400 hover:text-sky-300 font-mono"
            >
              .. (up)
            </button>
          )}
        </div>

        <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5 text-xs">
          {files.map((entry) => {
            const isSelected = activeTabPath === entry.path;
            const ext = entry.name.split('.').pop()?.toLowerCase();
            return (
              <button
                key={entry.path}
                onClick={() => handleSelectFile(entry)}
                className={`w-full flex items-center gap-2 px-2 py-1.5 rounded transition-all text-left ${
                  isSelected
                    ? 'bg-sky-500/15 text-sky-300 font-medium'
                    : 'text-slate-300 hover:bg-slate-800/80 hover:text-slate-100'
                }`}
              >
                {entry.is_dir ? (
                  <Folder className="w-3.5 h-3.5 text-sky-400 fill-sky-400/20 flex-shrink-0" />
                ) : ext === 'swift' ? (
                  <Code2 className="w-3.5 h-3.5 text-amber-400 flex-shrink-0" />
                ) : ext === 'm' || ext === 'mm' ? (
                  <Code2 className="w-3.5 h-3.5 text-sky-400 flex-shrink-0" />
                ) : (
                  <File className="w-3.5 h-3.5 text-slate-400 flex-shrink-0" />
                )}
                <span className="truncate">{entry.name}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Editor Main Area */}
      <div className="flex-1 flex flex-col bg-slate-950 overflow-hidden">
        {/* Editor Tab Bar */}
        <div className="h-9 bg-slate-900/90 border-b border-slate-800 flex items-center justify-between px-2 overflow-x-auto">
          <div className="flex items-center gap-1 overflow-x-auto">
            {openTabs.map((tab) => {
              const isActive = tab.path === activeTabPath;
              const tabDirty = tab.content !== tab.originalContent;
              return (
                <div
                  key={tab.path}
                  onClick={() => setActiveTabPath(tab.path)}
                  className={`flex items-center gap-2 px-3 py-1.5 rounded-t text-xs cursor-pointer border-t border-x transition-colors ${
                    isActive
                      ? 'bg-slate-950 text-slate-100 border-slate-800 font-medium'
                      : 'bg-slate-900 text-slate-400 border-transparent hover:bg-slate-800/60'
                  }`}
                >
                  <FileCode className="w-3.5 h-3.5 text-sky-400 flex-shrink-0" />
                  <span className="truncate max-w-[120px]">{tab.name}</span>
                  {tabDirty && <span className="text-amber-400 font-bold">*</span>}
                  <button
                    onClick={(e) => handleCloseTab(e, tab.path)}
                    className="p-0.5 hover:bg-slate-800 rounded text-slate-500 hover:text-slate-300 ml-1"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
              );
            })}
          </div>

          {activeTab && (
            <div className="flex items-center gap-2 flex-shrink-0">
              {savedSuccess && (
                <span className="text-[11px] text-emerald-400 flex items-center gap-1 font-mono">
                  <Check className="w-3 h-3" /> Saved
                </span>
              )}
              <button
                onClick={handleSave}
                disabled={!isDirty || saving}
                className={`flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium transition-all ${
                  isDirty
                    ? 'bg-sky-500 hover:bg-sky-400 text-slate-950 cursor-pointer shadow-sm'
                    : 'bg-slate-800 text-slate-500 cursor-not-allowed'
                }`}
              >
                <Save className="w-3.5 h-3.5" />
                <span>{saving ? 'Saving...' : 'Save'}</span>
              </button>
            </div>
          )}
        </div>

        {/* Breadcrumbs Header */}
        {activeTab && (
          <div className="h-7 border-b border-slate-800/60 bg-slate-950/80 px-3 flex items-center text-[11px] text-slate-400 gap-1.5 font-mono">
            {breadcrumbs.map((crumb, idx) => (
              <React.Fragment key={idx}>
                {idx > 0 && <ChevronRight className="w-3 h-3 text-slate-600 flex-shrink-0" />}
                <span className={idx === breadcrumbs.length - 1 ? 'text-slate-200 font-medium' : ''}>
                  {crumb}
                </span>
              </React.Fragment>
            ))}
          </div>
        )}

        {/* Code Content & Line Numbers */}
        {activeTab ? (
          <div className="flex-1 flex overflow-hidden">
            {/* Line numbers */}
            <div className="w-12 bg-slate-950 border-r border-slate-800/60 text-right pr-2 pt-3 select-none text-[11px] font-mono text-slate-600 space-y-0.5">
              {Array.from({ length: Math.min(lineCount, 1000) }, (_, i) => (
                <div key={i + 1}>{i + 1}</div>
              ))}
            </div>

            {/* Textarea */}
            <textarea
              value={activeTab.content}
              onChange={(e) => handleContentChange(e.target.value)}
              onKeyUp={(e) => {
                const target = e.currentTarget;
                const lines = target.value.substr(0, target.selectionStart).split('\n');
                setCursorPos({
                  line: lines.length,
                  col: lines[lines.length - 1].length + 1,
                });
              }}
              onClick={(e) => {
                const target = e.currentTarget;
                const lines = target.value.substr(0, target.selectionStart).split('\n');
                setCursorPos({
                  line: lines.length,
                  col: lines[lines.length - 1].length + 1,
                });
              }}
              onKeyDown={(e) => {
                if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                  e.preventDefault();
                  handleSave();
                }
              }}
              spellCheck={false}
              className="flex-1 p-3 bg-transparent text-slate-200 font-mono text-xs leading-relaxed outline-none resize-none overflow-auto whitespace-pre selection:bg-sky-500/30"
            />
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-slate-600 text-xs">
            <FileCode className="w-10 h-10 mb-2 opacity-30" />
            <span>Select a file from Project Explorer to begin editing</span>
          </div>
        )}

        {/* Editor Status Bar */}
        {activeTab && (
          <div className="h-6 border-t border-slate-800/80 bg-slate-900/60 px-3 flex items-center justify-between text-[10px] text-slate-400 font-mono select-none">
            <div className="flex items-center gap-3">
              <span>Ln {cursorPos.line}, Col {cursorPos.col}</span>
              <span>{lineCount} lines</span>
              <span>Spaces: 2</span>
            </div>

            <div className="flex items-center gap-3">
              <span>UTF-8</span>
              <span>LF</span>
              <span className="text-sky-400 font-medium">{getLanguageName(activeTab.path)}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
