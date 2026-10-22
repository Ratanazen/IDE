import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FileCode, Save, Folder, File, Check } from 'lucide-react';
import { FileEntry, ProjectMetadata } from '../types';

interface EditorViewProps {
  project: ProjectMetadata | null;
  onLog: (line: string) => void;
}

export const EditorView: React.FC<EditorViewProps> = ({ project, onLog }) => {
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [originalContent, setOriginalContent] = useState<string>('');
  const [saving, setSaving] = useState(false);
  const [savedSuccess, setSavedSuccess] = useState(false);
  const [subpath, setSubpath] = useState<string | null>(null);

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

  const handleSelectFile = async (entry: FileEntry) => {
    if (entry.is_dir) {
      // Navigate into folder
      const nextSub = subpath ? `${subpath}/${entry.name}` : entry.name;
      setSubpath(nextSub);
      loadDirectory(nextSub);
    } else {
      // Read file
      try {
        const content = await invoke<string>('read_source_file', {
          filePath: entry.path,
        });
        setSelectedFile(entry.path);
        setFileContent(content);
        setOriginalContent(content);
        setSavedSuccess(false);
      } catch (err: any) {
        onLog(`[Editor] Failed to open ${entry.name}: ${err}`);
      }
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

  const handleSave = async () => {
    if (!selectedFile) return;
    setSaving(true);
    try {
      await invoke('save_source_file', {
        filePath: selectedFile,
        content: fileContent,
      });
      setOriginalContent(fileContent);
      setSavedSuccess(true);
      setTimeout(() => setSavedSuccess(false), 2000);
      onLog(`[Editor] Saved ${selectedFile}`);
    } catch (err: any) {
      onLog(`[Editor] Save error: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const isDirty = fileContent !== originalContent;
  const fileName = selectedFile ? selectedFile.split('/').pop() : 'No file open';
  const lineCount = fileContent.split('\n').length;

  if (!project) {
    return (
      <div className="flex-1 flex items-center justify-center text-slate-500 text-xs">
        Open a project to explore files and edit code.
      </div>
    );
  }

  return (
    <div className="flex-1 flex h-full overflow-hidden">
      {/* File Explorer Tree */}
      <div className="w-64 bg-slate-900 border-r border-slate-800 flex flex-col select-none">
        <div className="h-10 px-3 border-b border-slate-800 flex items-center justify-between text-xs font-semibold text-slate-300">
          <span>Project Explorer</span>
          {subpath && (
            <button
              onClick={handleNavigateUp}
              className="text-[11px] text-sky-400 hover:text-sky-300"
            >
              .. (up)
            </button>
          )}
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-0.5 text-xs">
          {files.map((entry) => {
            const isSelected = selectedFile === entry.path;
            return (
              <button
                key={entry.path}
                onClick={() => handleSelectFile(entry)}
                className={`w-full flex items-center gap-2 px-2 py-1.5 rounded transition-all text-left ${
                  isSelected
                    ? 'bg-sky-500/10 text-sky-400 font-medium'
                    : 'text-slate-300 hover:bg-slate-800/80 hover:text-slate-100'
                }`}
              >
                {entry.is_dir ? (
                  <Folder className="w-3.5 h-3.5 text-sky-400 fill-sky-400/20 flex-shrink-0" />
                ) : (
                  <File className="w-3.5 h-3.5 text-slate-400 flex-shrink-0" />
                )}
                <span className="truncate">{entry.name}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Editor Area */}
      <div className="flex-1 flex flex-col bg-slate-950 overflow-hidden">
        {/* Editor Tab Header */}
        <div className="h-10 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 select-none">
          <div className="flex items-center gap-2 text-xs">
            <FileCode className="w-4 h-4 text-sky-400" />
            <span className="font-semibold text-slate-200">{fileName}</span>
            {isDirty && <span className="text-amber-400 font-bold">*</span>}
            <span className="text-[11px] text-slate-500">({lineCount} lines)</span>
          </div>

          <div className="flex items-center gap-2">
            {savedSuccess && (
              <span className="text-[11px] text-emerald-400 flex items-center gap-1">
                <Check className="w-3 h-3" /> Saved
              </span>
            )}
            <button
              onClick={handleSave}
              disabled={!isDirty || saving || !selectedFile}
              className={`flex items-center gap-1.5 px-3 py-1 rounded text-xs font-medium transition-all ${
                isDirty && selectedFile
                  ? 'bg-sky-500 hover:bg-sky-400 text-slate-950 cursor-pointer'
                  : 'bg-slate-800 text-slate-500 cursor-not-allowed'
              }`}
            >
              <Save className="w-3.5 h-3.5" />
              <span>{saving ? 'Saving...' : 'Save'}</span>
            </button>
          </div>
        </div>

        {/* Code Content */}
        {selectedFile ? (
          <div className="flex-1 flex overflow-hidden">
            {/* Line numbers */}
            <div className="w-12 bg-slate-950/60 border-r border-slate-800/80 text-right pr-2 pt-3 select-none text-[11px] font-mono text-slate-600 space-y-0.5">
              {Array.from({ length: Math.min(lineCount, 500) }, (_, i) => (
                <div key={i + 1}>{i + 1}</div>
              ))}
            </div>

            {/* Textarea */}
            <textarea
              value={fileContent}
              onChange={(e) => setFileContent(e.target.value)}
              onKeyDown={(e) => {
                if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                  e.preventDefault();
                  handleSave();
                }
              }}
              spellCheck={false}
              className="flex-1 p-3 bg-transparent text-slate-200 font-mono text-xs leading-relaxed outline-none resize-none overflow-auto whitespace-pre"
            />
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-slate-600 text-xs">
            <FileCode className="w-10 h-10 mb-2 opacity-40" />
            <span>Select a file from the Project Explorer to edit</span>
          </div>
        )}
      </div>
    </div>
  );
};
