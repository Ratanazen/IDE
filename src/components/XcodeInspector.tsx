import React, { useState } from 'react';
import {
  FileText,
  Sliders,
  Shield,
  LayoutTemplate,
  Check,
  Search,
  Plus,
  Info,
  Smartphone,
  Lock,
} from 'lucide-react';
import { EcosystemProject, ProjectMetadata } from '../types';

interface XcodeInspectorProps {
  project: ProjectMetadata | null;
  ecosystemProject: EcosystemProject | null;
  activeFilePath?: string;
  onUpdateBundleId?: (newId: string) => void;
}

export const XcodeInspector: React.FC<XcodeInspectorProps> = ({
  project,
  ecosystemProject,
  activeFilePath,
  onUpdateBundleId,
}) => {

  const [activeTab, setActiveTab] = useState<'file' | 'build-settings' | 'signing' | 'preview'>('signing');
  const [buildSettingsSearch, setBuildSettingsSearch] = useState('');
  const [bundleId, setBundleId] = useState(
    ecosystemProject?.bundle_identifier || 'com.example.fixtureApp'
  );
  const [selectedTeam, setSelectedTeam] = useState('Personal Team (Apple Developer)');

  const [capabilities, setCapabilities] = useState<string[]>([
    'Push Notifications',
    'Background Modes (Fetch, Remote Notifications)',
    'Sign in with Apple',
  ]);

  const [isAddCapabilityOpen, setIsAddCapabilityOpen] = useState(false);

  const availableCapabilities = [
    'iCloud & CloudKit',
    'App Groups',
    'Associated Domains (Universal Links)',
    'HealthKit',
    'HomeKit',
    'SiriKit',
    'Wallet & Apple Pay',
  ];

  const handleAddCapability = (cap: string) => {
    if (!capabilities.includes(cap)) {
      setCapabilities([...capabilities, cap]);
    }
    setIsAddCapabilityOpen(false);
  };

  const handleRemoveCapability = (cap: string) => {
    setCapabilities(capabilities.filter((c) => c !== cap));
  };

  const fileName = activeFilePath
    ? activeFilePath.split('/').pop() || 'Selected File'
    : 'No File Selected';

  const fileExt = fileName.includes('.') ? fileName.split('.').pop() || '' : '';

  const buildSettingsList = [
    { key: 'PRODUCT_BUNDLE_IDENTIFIER', label: 'Product Bundle Identifier', value: bundleId, category: 'Packaging' },
    { key: 'IPHONEOS_DEPLOYMENT_TARGET', label: 'iOS Deployment Target', value: ecosystemProject?.deployment_target || '15.0', category: 'Deployment' },
    { key: 'SWIFT_VERSION', label: 'Swift Language Version', value: 'Swift 5.9', category: 'Swift Compiler' },
    { key: 'ARCHS', label: 'Architectures', value: 'arm64, x86_64', category: 'Architectures' },
    { key: 'VALID_ARCHS', label: 'Valid Architectures', value: 'arm64 (iOS Device), x86_64 (Simulator)', category: 'Architectures' },
    { key: 'HEADER_SEARCH_PATHS', label: 'Header Search Paths', value: '$(PROJECT_DIR)/include, $(SDKROOT)/usr/include', category: 'Search Paths' },
    { key: 'FRAMEWORK_SEARCH_PATHS', label: 'Framework Search Paths', value: '$(SDKROOT)/System/Library/Frameworks', category: 'Search Paths' },
    { key: 'OTHER_LDFLAGS', label: 'Other Linker Flags', value: '-ld64.lld -framework Foundation -framework UIKit', category: 'Linking' },
    { key: 'ENABLE_BITCODE', label: 'Enable Bitcode', value: 'NO (Apple deprecated)', category: 'Build Options' },
    { key: 'CLANG_ENABLE_OBJC_ARC', label: 'Objective-C Automatic Reference Counting', value: 'YES', category: 'Apple LLVM' },
  ];

  const filteredSettings = buildSettingsList.filter(
    (s) =>
      s.label.toLowerCase().includes(buildSettingsSearch.toLowerCase()) ||
      s.key.toLowerCase().includes(buildSettingsSearch.toLowerCase()) ||
      s.category.toLowerCase().includes(buildSettingsSearch.toLowerCase())
  );

  return (
    <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col h-full select-none text-xs flex-shrink-0 z-10">
      {/* Inspector Header & Tab Bar */}
      <div className="h-10 border-b border-slate-800 flex items-center justify-around px-2 bg-slate-950">
        <button
          onClick={() => setActiveTab('file')}
          title="File Inspector"
          className={`p-1.5 rounded transition-colors ${
            activeTab === 'file'
              ? 'text-sky-400 bg-slate-900 border border-slate-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileText className="w-4 h-4" />
        </button>

        <button
          onClick={() => setActiveTab('build-settings')}
          title="Build Settings"
          className={`p-1.5 rounded transition-colors ${
            activeTab === 'build-settings'
              ? 'text-sky-400 bg-slate-900 border border-slate-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Sliders className="w-4 h-4" />
        </button>

        <button
          onClick={() => setActiveTab('signing')}
          title="Signing & Capabilities"
          className={`p-1.5 rounded transition-colors ${
            activeTab === 'signing'
              ? 'text-sky-400 bg-slate-900 border border-slate-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Shield className="w-4 h-4" />
        </button>

        <button
          onClick={() => setActiveTab('preview')}
          title="SwiftUI Preview"
          className={`p-1.5 rounded transition-colors ${
            activeTab === 'preview'
              ? 'text-sky-400 bg-slate-900 border border-slate-800'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <LayoutTemplate className="w-4 h-4" />
        </button>
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-y-auto p-3">
        {/* TAB 1: File Inspector */}
        {activeTab === 'file' && (
          <div className="space-y-4">
            <div>
              <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">
                Identity and Type
              </span>
              <div className="space-y-2 bg-slate-950/60 p-2.5 rounded-lg border border-slate-800">
                <div>
                  <span className="text-slate-500 text-[11px] block">Project</span>
                  <span className="text-slate-300 font-medium">
                    {project ? project.name : ecosystemProject?.name || 'Workspace'}
                  </span>
                </div>
                <div>
                  <span className="text-slate-500 text-[11px] block">Name</span>
                  <span className="font-semibold text-slate-200">{fileName}</span>
                </div>

                <div>
                  <span className="text-slate-500 text-[11px] block">Type</span>
                  <span className="text-slate-300">
                    {fileExt === 'swift'
                      ? 'Swift Source Code'
                      : fileExt === 'm'
                      ? 'Objective-C Source'
                      : fileExt === 'dart'
                      ? 'Dart Source Code'
                      : fileExt === 'plist'
                      ? 'Property List'
                      : fileExt === 'pbxproj'
                      ? 'Xcode Project File'
                      : 'Source Document'}
                  </span>
                </div>
                <div>
                  <span className="text-slate-500 text-[11px] block">Full Path</span>
                  <span className="text-slate-400 break-all text-[10px] font-mono">
                    {activeFilePath || 'None'}
                  </span>
                </div>
              </div>
            </div>

            <div>
              <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">
                Target Membership
              </span>
              <div className="space-y-1.5 bg-slate-950/60 p-2.5 rounded-lg border border-slate-800">
                {(ecosystemProject?.targets || [
                  { id: '1', name: 'Runner' },
                  { id: '2', name: 'RunnerTests' },
                ]).map((t) => (
                  <label key={t.id} className="flex items-center gap-2 cursor-pointer text-slate-300 hover:text-white">
                    <input type="checkbox" defaultChecked className="accent-sky-500 rounded" />
                    <span>{t.name}</span>
                  </label>
                ))}
              </div>
            </div>

            <div>
              <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">
                Text Settings
              </span>
              <div className="bg-slate-950/60 p-2.5 rounded-lg border border-slate-800 space-y-2 text-slate-300">
                <div className="flex justify-between">
                  <span className="text-slate-500">Indent:</span>
                  <span>Spaces (2)</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Encoding:</span>
                  <span>UTF-8</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Line Endings:</span>
                  <span>LF (Unix)</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* TAB 2: Build Settings */}
        {activeTab === 'build-settings' && (
          <div className="space-y-3">
            <div className="relative">
              <Search className="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-2.5" />
              <input
                type="text"
                placeholder="Search Build Settings..."
                value={buildSettingsSearch}
                onChange={(e) => setBuildSettingsSearch(e.target.value)}
                className="w-full bg-slate-950 border border-slate-800 rounded-md pl-8 pr-2.5 py-1.5 text-xs text-slate-200 outline-none focus:border-sky-500"
              />
            </div>

            <div className="space-y-3">
              {filteredSettings.map((item) => (
                <div key={item.key} className="bg-slate-950/60 p-2.5 rounded-lg border border-slate-800">
                  <div className="flex justify-between items-start mb-1">
                    <span className="text-slate-200 font-medium">{item.label}</span>
                    <span className="text-[9px] px-1 py-0.5 rounded bg-slate-800 text-slate-400">
                      {item.category}
                    </span>
                  </div>
                  <span className="text-[10px] font-mono text-slate-500 block mb-1">{item.key}</span>
                  <span className="text-sky-400 font-mono text-[11px] break-all">{item.value}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* TAB 3: Signing & Capabilities */}
        {activeTab === 'signing' && (
          <div className="space-y-4">
            <div>
              <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">
                Signing Certificate &amp; Team
              </span>
              <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800 space-y-2.5">
                <div>
                  <span className="text-slate-400 text-[11px] block mb-1">Team</span>
                  <select
                    value={selectedTeam}
                    onChange={(e) => setSelectedTeam(e.target.value)}
                    className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-slate-200 outline-none focus:border-sky-500"
                  >
                    <option value="Personal Team (Apple Developer)">Personal Team (Apple Developer)</option>
                    <option value="Enterprise Team (In-House)">Enterprise Team (In-House)</option>
                    <option value="Ad-Hoc Local Signature">Ad-Hoc Local Signature (Linux Compatible)</option>
                  </select>
                </div>

                <div>
                  <span className="text-slate-400 text-[11px] block mb-1">Bundle Identifier</span>
                  <input
                    type="text"
                    value={bundleId}
                    onChange={(e) => {
                      setBundleId(e.target.value);
                      if (onUpdateBundleId) onUpdateBundleId(e.target.value);
                    }}
                    className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 font-mono text-slate-200 outline-none focus:border-sky-500"
                  />
                </div>

                <div className="pt-2 border-t border-slate-800 text-[11px] text-slate-400 flex items-center justify-between">
                  <span>Signing Status:</span>
                  <span className="text-emerald-400 font-medium flex items-center gap-1">
                    <Check className="w-3.5 h-3.5" /> Ad-Hoc / CodeResources Ready
                  </span>
                </div>
              </div>
            </div>

            {/* Capabilities */}
            <div>
              <div className="flex items-center justify-between mb-1.5">
                <span className="text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                  Capabilities ({capabilities.length})
                </span>
                <button
                  onClick={() => setIsAddCapabilityOpen(!isAddCapabilityOpen)}
                  className="flex items-center gap-1 text-sky-400 hover:text-sky-300 text-[11px] font-medium"
                >
                  <Plus className="w-3.5 h-3.5" /> Capability
                </button>
              </div>

              {isAddCapabilityOpen && (
                <div className="mb-2 bg-slate-950 border border-sky-500/40 rounded-lg p-2 space-y-1">
                  <span className="text-[10px] text-slate-400 block mb-1 font-semibold">
                    Select capability to add:
                  </span>
                  {availableCapabilities.map((cap) => (
                    <button
                      key={cap}
                      onClick={() => handleAddCapability(cap)}
                      className="w-full text-left px-2 py-1 rounded text-slate-300 hover:bg-slate-800 hover:text-white"
                    >
                      + {cap}
                    </button>
                  ))}
                </div>
              )}

              <div className="space-y-2">
                {capabilities.map((cap) => (
                  <div
                    key={cap}
                    className="bg-slate-950/60 p-2.5 rounded-lg border border-slate-800 flex items-center justify-between"
                  >
                    <div className="flex items-center gap-2">
                      <Lock className="w-3.5 h-3.5 text-sky-400" />
                      <span className="text-slate-200">{cap}</span>
                    </div>
                    <button
                      onClick={() => handleRemoveCapability(cap)}
                      className="text-slate-500 hover:text-rose-400 text-xs px-1"
                    >
                      &times;
                    </button>
                  </div>
                ))}
              </div>
            </div>

            {/* Honest Notice */}
            <div className="p-2.5 rounded-lg bg-sky-950/20 border border-sky-800/40 text-[11px] text-sky-300 flex items-start gap-2">
              <Info className="w-4 h-4 flex-shrink-0 mt-0.5 text-sky-400" />
              <span>
                Ridu signs bundles on Linux using <code>_CodeSignature/CodeResources</code> with SHA-256 integrity. Official App Store notarization requires Apple Developer connection on macOS.
              </span>
            </div>
          </div>
        )}

        {/* TAB 4: SwiftUI Preview */}
        {activeTab === 'preview' && (
          <div className="space-y-3">
            <div className="p-2.5 rounded-lg bg-slate-950/80 border border-slate-800">
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block mb-1">
                SwiftUI Component Canvas
              </span>
              <p className="text-[11px] text-slate-500 mb-3">
                Target: iPhone 16 Pro (iOS 18)
              </p>

              {/* Simulated iPhone frame */}
              <div className="w-full aspect-[9/16] max-h-72 mx-auto bg-slate-950 rounded-2xl border-2 border-slate-700 p-2 flex flex-col shadow-xl relative overflow-hidden">
                <div className="w-16 h-3 bg-slate-800 rounded-full mx-auto mb-2" />
                <div className="flex-1 flex flex-col items-center justify-center p-2 text-center space-y-2">
                  <div className="w-10 h-10 rounded-full bg-sky-500/20 text-sky-400 flex items-center justify-center">
                    <Smartphone className="w-5 h-5" />
                  </div>
                  <h4 className="font-semibold text-slate-200 text-xs">SwiftUI ContentView</h4>
                  <p className="text-[10px] text-slate-400">
                    Text("Hello, World!")<br />.padding()
                  </p>
                </div>
              </div>
            </div>

            <div className="p-2.5 rounded-lg bg-amber-950/20 border border-amber-800/40 text-[11px] text-amber-300 flex items-start gap-2">
              <Info className="w-4 h-4 flex-shrink-0 mt-0.5 text-amber-400" />
              <span>
                <strong>Linux Host Notice:</strong> Live interactive Metal simulator runtimes are exclusively available on macOS. On Linux, Ridu parses SwiftUI AST and structural declarations into this live component canvas.
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
