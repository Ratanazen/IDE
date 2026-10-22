import React from 'react';
import {
  Files,
  Sliders,
  Search,
  GitBranch,
  PlayCircle,
  Cpu,
  Puzzle,
  Settings,
} from 'lucide-react';
import { ActivityBarTab } from '../types';

interface ActivityBarProps {
  activeTab: ActivityBarTab;
  onTabChange: (tab: ActivityBarTab) => void;
  problemCount?: number;
  gitChangeCount?: number;
}

export const ActivityBar: React.FC<ActivityBarProps> = ({
  activeTab,
  onTabChange,
  problemCount = 0,
  gitChangeCount = 0,
}) => {
  const topTabs = [
    {
      id: 'explorer' as ActivityBarTab,
      label: 'Explorer (Ctrl+Shift+E)',
      icon: Files,
    },
    {
      id: 'xcode-inspector' as ActivityBarTab,
      label: 'Xcode Inspector & Settings',
      icon: Sliders,
      badge: problemCount > 0 ? problemCount : undefined,
    },

    {
      id: 'search' as ActivityBarTab,
      label: 'Search in Files (Ctrl+Shift+F)',
      icon: Search,
    },
    {
      id: 'git' as ActivityBarTab,
      label: 'Source Control (Ctrl+Shift+G)',
      icon: GitBranch,
      badge: gitChangeCount > 0 ? gitChangeCount : undefined,
    },
    {
      id: 'debug' as ActivityBarTab,
      label: 'Run & Debug (Ctrl+Shift+D)',
      icon: PlayCircle,
    },
    {
      id: 'toolchain' as ActivityBarTab,
      label: 'Toolchain & Apple SDK Matrix',
      icon: Cpu,
    },
    {
      id: 'plugins' as ActivityBarTab,
      label: 'Plugins & Extensions (Ctrl+Shift+X)',
      icon: Puzzle,
    },
  ];

  return (
    <div className="w-12 bg-slate-950 border-r border-slate-800/80 flex flex-col items-center justify-between py-2 select-none flex-shrink-0 z-20">
      <div className="flex flex-col items-center gap-1 w-full">
        {/* Ridu Logo */}
        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-sky-500 to-blue-600 flex items-center justify-center text-white font-bold text-sm shadow-md shadow-sky-950 mb-2">
          R
        </div>

        {topTabs.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              title={item.label}
              className={`relative w-10 h-10 rounded-lg flex items-center justify-center transition-all cursor-pointer group ${
                isActive
                  ? 'text-sky-400 bg-slate-900 border border-slate-700/60 shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'
              }`}
            >
              {isActive && (
                <div className="absolute left-0 top-2 bottom-2 w-0.5 bg-sky-400 rounded-r" />
              )}
              <Icon className="w-5 h-5" />
              {item.badge !== undefined && (
                <span className="absolute top-1 right-1 px-1 min-w-[14px] h-[14px] rounded-full bg-sky-500 text-[9px] font-bold text-white flex items-center justify-center leading-none">
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      <div className="flex flex-col items-center gap-1 w-full">
        <button
          onClick={() => onTabChange('settings')}
          title="Studio Settings"
          className={`w-10 h-10 rounded-lg flex items-center justify-center transition-all cursor-pointer ${
            activeTab === 'settings'
              ? 'text-sky-400 bg-slate-900 border border-slate-700/60'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'
          }`}
        >
          <Settings className="w-5 h-5" />
        </button>
      </div>
    </div>
  );
};
