import React from 'react';
import { FolderGit2, FileCode, Hammer, Archive, Wrench, Settings } from 'lucide-react';
import { NavTab } from '../types';

interface SidebarProps {
  currentTab: NavTab;
  onTabChange: (tab: NavTab) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ currentTab, onTabChange }) => {
  const navItems = [
    { id: 'projects' as NavTab, label: 'Projects', icon: FolderGit2 },
    { id: 'editor' as NavTab, label: 'Editor', icon: FileCode },
    { id: 'builds' as NavTab, label: 'Build Center', icon: Hammer },
    { id: 'artifacts' as NavTab, label: 'Artifacts', icon: Archive },
    { id: 'toolchain' as NavTab, label: 'Toolchain', icon: Wrench },
    { id: 'settings' as NavTab, label: 'Settings', icon: Settings },
  ];

  return (
    <aside className="w-56 bg-slate-900 border-r border-slate-800 flex flex-col flex-shrink-0 select-none">
      <div className="h-14 flex items-center px-4 border-b border-slate-800 gap-3">
        <div className="w-8 h-8 rounded-lg bg-sky-500/20 border border-sky-500/40 flex items-center justify-center text-sky-400 font-bold text-lg">
          R
        </div>
        <div>
          <h1 className="text-sm font-semibold text-slate-100 tracking-tight">Ridu IDE</h1>
          <p className="text-[10px] text-slate-400">Flutter & iOS Studio</p>
        </div>
      </div>

      <nav className="p-3 space-y-1 flex-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                isActive
                  ? 'bg-sky-500/10 text-sky-400 border border-sky-500/30'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'
              }`}
            >
              <Icon className={`w-4 h-4 ${isActive ? 'text-sky-400' : 'text-slate-400'}`} />
              <span>{item.label}</span>
            </button>
          );
        })}
      </nav>

      <div className="p-3 border-t border-slate-800 text-[11px] text-slate-500 flex justify-between items-center">
        <span>Arch Linux / Garuda</span>
        <span className="inline-block w-2 h-2 rounded-full bg-emerald-500"></span>
      </div>
    </aside>
  );
};
