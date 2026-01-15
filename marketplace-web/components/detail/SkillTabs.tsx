'use client';

import { useState } from 'react';
import { Book, Download, Terminal, Package, FileText } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { SkillManifest } from '@/lib/types';
import InstallationTabs from './InstallationTabs';
import ToolsList from './ToolsList';
import ExamplesSection from './ExamplesSection';
import SkillMdViewer from './SkillMdViewer';

type TabId = 'overview' | 'installation' | 'examples' | 'tools' | 'documentation';

interface SkillTabsProps {
  skill: SkillManifest;
}

export default function SkillTabs({ skill }: SkillTabsProps) {
  const [activeTab, setActiveTab] = useState<TabId>('overview');

  const tabs = [
    { id: 'overview' as const, label: 'Overview', icon: Book },
    { id: 'installation' as const, label: 'Installation', icon: Download },
    ...(skill.examples && skill.examples.length > 0
      ? [{ id: 'examples' as const, label: 'Examples', icon: Terminal }]
      : []),
    ...(skill.tools && skill.tools.length > 0
      ? [{ id: 'tools' as const, label: `Tools (${skill.tools.length})`, icon: Package }]
      : []),
    ...(skill.skillMdContent || skill.skillMdUrl
      ? [{ id: 'documentation' as const, label: 'Documentation', icon: FileText }]
      : []),
  ];

  return (
    <div>
      {/* Tab Navigation */}
      <div className="border-b border-gray-200 mb-8">
        <div className="flex space-x-8 overflow-x-auto">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`pb-4 px-2 border-b-2 font-medium text-sm transition-colors whitespace-nowrap flex items-center gap-2 ${
                  activeTab === tab.id
                    ? 'border-primary text-primary'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* Tab Content */}
      <div>
        {activeTab === 'overview' && (
          <div className="card">
            <div className="prose prose-sm prose-gray max-w-none">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {skill.longDescription || skill.description}
              </ReactMarkdown>
            </div>
          </div>
        )}

        {activeTab === 'installation' && (
          <div className="card">
            <InstallationTabs skill={skill} />
          </div>
        )}

        {activeTab === 'examples' && skill.examples && (
          <ExamplesSection examples={skill.examples} />
        )}

        {activeTab === 'tools' && skill.tools && (
          <ToolsList tools={skill.tools} skillId={skill.id} />
        )}

        {activeTab === 'documentation' && (skill.skillMdContent || skill.skillMdUrl) && (
          <div>
            {skill.skillMdContent ? (
              <div className="card">
                <div className="prose prose-sm prose-gray max-w-none">
                  <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    components={{
                      // Custom rendering for code blocks
                      pre: ({ node, ...props }) => (
                        <pre className="bg-gray-900 text-gray-100 rounded-lg p-4 overflow-x-auto" {...props} />
                      ),
                      code: ({ node, inline, ...props }: any) =>
                        inline ? (
                          <code className="bg-gray-100 text-gray-800 px-1.5 py-0.5 rounded text-sm font-mono" {...props} />
                        ) : (
                          <code className="text-gray-100 font-mono text-sm" {...props} />
                        ),
                      // Add custom styling for headers
                      h1: ({ node, ...props }) => (
                        <h1 className="text-3xl font-bold text-gray-900 mt-8 mb-4 pb-2 border-b border-gray-200" {...props} />
                      ),
                      h2: ({ node, ...props }) => (
                        <h2 className="text-2xl font-bold text-gray-900 mt-6 mb-3" {...props} />
                      ),
                      h3: ({ node, ...props }) => (
                        <h3 className="text-xl font-semibold text-gray-900 mt-4 mb-2" {...props} />
                      ),
                      // Style links
                      a: ({ node, ...props }) => (
                        <a className="text-primary hover:text-blue-700 font-medium underline" target="_blank" rel="noopener noreferrer" {...props} />
                      ),
                    }}
                  >
                    {skill.skillMdContent}
                  </ReactMarkdown>
                </div>
              </div>
            ) : skill.skillMdUrl ? (
              <SkillMdViewer skillMdUrl={skill.skillMdUrl} skillName={skill.name} />
            ) : null}
          </div>
        )}
      </div>
    </div>
  );
}
