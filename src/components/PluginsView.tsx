import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Puzzle, Shield, Search, RefreshCw } from 'lucide-react';
import { PluginManifest } from '../types';

export const PluginsView: React.FC = () => {
  const [plugins, setPlugins] = useState<PluginManifest[]>([]);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(false);



  const loadPlugins = async () => {
    setLoading(true);
    try {
      const list = await invoke<PluginManifest[]>('list_plugins');
      setPlugins(list);
    } catch {
      // Fallback default builtins if IPC error
      setPlugins([
        {
          id: 'ridu.apple-swift',
          name: 'Apple Swift & SwiftUI Support',
          version: '1.0.0',
          description: 'Native Swift compilation, SourceKit-LSP, and SwiftUI preview workflows.',
          author: 'Ridu Core Team',
          permissions: ['FilesystemRead', 'ProcessSpawn', 'ToolchainAccess'],
          activation_events: ['onLanguage:swift', 'onProject:spm'],
          is_enabled: true,
          is_builtin: true,
        },
        {
          id: 'ridu.xcode-inspector',
          name: 'Xcode Project & Target Inspector',
          version: '1.0.0',
          description: 'Inspects .xcodeproj/project.pbxproj, build settings, schemes, and signing capabilities.',
          author: 'Ridu Core Team',
          permissions: ['FilesystemRead', 'FilesystemWrite'],
          activation_events: ['onProject:xcode'],
          is_enabled: true,
          is_builtin: true,
        },
        {
          id: 'ridu.clang-llvm',
          name: 'LLVM Clang & ld64.lld Toolchain',
          version: '1.0.0',
          description: 'C, C++, Objective-C compiler driver and Apple Mach-O linker integration.',
          author: 'Ridu Core Team',
          permissions: ['FilesystemRead', 'ProcessSpawn', 'ToolchainAccess'],
          activation_events: ['onLanguage:objective-c', 'onLanguage:c'],
          is_enabled: true,
          is_builtin: true,
        },
        {
          id: 'ridu.flutter-dart',
          name: 'Flutter & Dart Desktop Studio',
          version: '1.0.0',
          description: 'Multi-platform Flutter build engine, hot reload, and dependency manager.',
          author: 'Ridu Core Team',
          permissions: ['FilesystemRead', 'FilesystemWrite', 'ProcessSpawn', 'ToolchainAccess'],
          activation_events: ['onProject:flutter'],
          is_enabled: true,
          is_builtin: true,
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadPlugins();
  }, []);

  const togglePlugin = (id: string) => {
    setPlugins((prev) =>
      prev.map((p) => (p.id === id ? { ...p, is_enabled: !p.is_enabled } : p))
    );
  };

  const filtered = plugins.filter(
    (p) =>
      p.name.toLowerCase().includes(search.toLowerCase()) ||
      p.description.toLowerCase().includes(search.toLowerCase()) ||
      p.author.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="flex-1 overflow-y-auto p-6 bg-slate-950 text-slate-100">
      <div className="max-w-4xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
          <div>
            <h2 className="text-xl font-bold tracking-tight text-slate-100 flex items-center gap-2">
              <Puzzle className="w-5 h-5 text-sky-400" />
              Plugins &amp; Extensions Architecture
              {loading && <RefreshCw className="w-4 h-4 animate-spin text-sky-400" />}
            </h2>

            <p className="text-xs text-slate-400 mt-1">
              Modular toolchain adapters, language servers, and development workflows for Ridu IDE.
            </p>
          </div>

          <div className="relative w-64">
            <Search className="w-4 h-4 text-slate-500 absolute left-3 top-2.5" />
            <input
              type="text"
              placeholder="Search extensions..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full bg-slate-900 border border-slate-700 rounded-lg pl-9 pr-3 py-1.5 text-xs text-slate-200 outline-none focus:border-sky-500"
            />
          </div>
        </div>

        {/* Plugin Cards Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {filtered.map((plugin) => (
            <div
              key={plugin.id}
              className={`p-4 rounded-xl border transition-all ${
                plugin.is_enabled
                  ? 'bg-slate-900/80 border-slate-700/80 hover:border-slate-600'
                  : 'bg-slate-900/30 border-slate-800/60 opacity-60'
              }`}
            >
              <div className="flex items-start justify-between gap-3 mb-2">
                <div>
                  <h3 className="font-semibold text-slate-200 text-sm flex items-center gap-1.5">
                    {plugin.name}
                    {plugin.is_builtin && (
                      <span className="text-[9px] font-bold px-1.5 py-0.2 rounded bg-sky-500/20 text-sky-400 border border-sky-500/30">
                        BUILT-IN
                      </span>
                    )}
                  </h3>
                  <span className="text-[11px] text-slate-500">
                    v{plugin.version} • {plugin.author}
                  </span>
                </div>

                <button
                  onClick={() => togglePlugin(plugin.id)}
                  className={`px-3 py-1 rounded-md text-xs font-medium transition-colors ${
                    plugin.is_enabled
                      ? 'bg-slate-800 text-slate-300 hover:bg-slate-700 hover:text-white'
                      : 'bg-sky-600 text-white hover:bg-sky-500'
                  }`}
                >
                  {plugin.is_enabled ? 'Disable' : 'Enable'}
                </button>
              </div>

              <p className="text-xs text-slate-400 mb-3 leading-relaxed">
                {plugin.description}
              </p>

              {/* Permissions & Security */}
              <div className="pt-3 border-t border-slate-800/80">
                <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500 block mb-1.5">
                  Declared Permissions:
                </span>
                <div className="flex flex-wrap gap-1.5">
                  {plugin.permissions.map((perm) => (
                    <span
                      key={perm}
                      className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] bg-slate-950 text-slate-400 border border-slate-800"
                    >
                      <Shield className="w-3 h-3 text-sky-400" />
                      {perm}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
