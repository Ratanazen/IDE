import React, { useState, useEffect, useRef } from 'react';
import { Search } from 'lucide-react';


export interface CommandItem {
  id: string;
  title: string;
  category: 'Xcode' | 'Build' | 'View' | 'Toolchain' | 'Flutter' | 'File' | 'Plugins';
  shortcut?: string;
  icon: React.ElementType;
  action: () => void;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: CommandItem[];
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  commands,
}) => {
  const [search, setSearch] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setSearch('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const filtered = commands.filter((cmd) => {
    const query = search.toLowerCase();
    return (
      cmd.title.toLowerCase().includes(query) ||
      cmd.category.toLowerCase().includes(query) ||
      (cmd.shortcut && cmd.shortcut.toLowerCase().includes(query))
    );
  });

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1 < filtered.length ? prev + 1 : 0));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 >= 0 ? prev - 1 : filtered.length - 1));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filtered[selectedIndex]) {
        filtered[selectedIndex].action();
        onClose();
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div
      onClick={onClose}
      className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-start justify-center pt-24 z-50 p-4"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-xl bg-slate-900 border border-slate-700/80 rounded-xl shadow-2xl overflow-hidden flex flex-col text-xs"
      >
        {/* Input */}
        <div className="flex items-center px-3 border-b border-slate-800 bg-slate-950">
          <Search className="w-4 h-4 text-slate-400 mr-2" />
          <input
            ref={inputRef}
            type="text"
            placeholder="Type a command or search..."
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            className="w-full bg-transparent py-3 text-slate-100 placeholder-slate-500 outline-none font-sans text-sm"
          />
        </div>

        {/* Command List */}
        <div className="max-h-80 overflow-y-auto p-1.5 space-y-0.5">
          {filtered.length === 0 ? (
            <div className="p-4 text-center text-slate-500 text-xs">
              No matching commands found.
            </div>
          ) : (
            filtered.map((cmd, idx) => {
              const Icon = cmd.icon;
              const isSelected = idx === selectedIndex;
              return (
                <div
                  key={cmd.id}
                  onClick={() => {
                    cmd.action();
                    onClose();
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={`flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer transition-colors ${
                    isSelected
                      ? 'bg-sky-500/15 text-sky-300 border border-sky-500/30'
                      : 'text-slate-300 hover:bg-slate-800/60'
                  }`}
                >
                  <div className="flex items-center gap-2.5">
                    <Icon className={`w-4 h-4 ${isSelected ? 'text-sky-400' : 'text-slate-400'}`} />
                    <span className="text-[10px] uppercase font-bold text-slate-500 px-1.5 py-0.5 rounded bg-slate-800 border border-slate-700">
                      {cmd.category}
                    </span>
                    <span className="font-medium text-slate-200">{cmd.title}</span>
                  </div>

                  {cmd.shortcut && (
                    <kbd className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 border border-slate-700">
                      {cmd.shortcut}
                    </kbd>
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="px-3 py-1.5 bg-slate-950 border-t border-slate-800 flex justify-between text-[10px] text-slate-500">
          <span>Navigate with &uarr; &darr;, Select with Enter</span>
          <span>Esc to dismiss</span>
        </div>
      </div>
    </div>
  );
};
