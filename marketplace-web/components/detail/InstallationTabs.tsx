'use client';

import { useState } from 'react';
import { SkillManifest } from '@/lib/types';
import { generateTOML } from '@/lib/generators/tomlGenerator';
import { generateCLI } from '@/lib/generators/cliGenerator';
import { generateMCP } from '@/lib/generators/mcpGenerator';
import { generateGitHubInstructions } from '@/lib/generators/readmeGenerator';

type InstallMethod = 'toml' | 'cli' | 'mcp' | 'github';

interface InstallationTabsProps {
  skill: SkillManifest;
}

export default function InstallationTabs({ skill }: InstallationTabsProps) {
  const [activeMethod, setActiveMethod] = useState<InstallMethod>('toml');

  const methods = [
    { id: 'toml' as const, label: 'TOML', description: 'Add to .skill-engine.toml' },
    { id: 'cli' as const, label: 'CLI', description: 'Install via command line' },
    { id: 'mcp' as const, label: 'MCP', description: 'Configure MCP server' },
    { id: 'github' as const, label: 'GitHub', description: 'Clone and manual setup' },
  ];

  const getContent = () => {
    switch (activeMethod) {
      case 'toml':
        return { code: generateTOML(skill), language: 'toml' };
      case 'cli':
        return { code: generateCLI(skill), language: 'bash' };
      case 'mcp':
        return { code: generateMCP(skill), language: 'json' };
      case 'github':
        return { code: generateGitHubInstructions(skill), language: 'markdown' };
    }
  };

  const content = getContent();

  const copyToClipboard = () => {
    navigator.clipboard.writeText(content.code);
    // Could add toast notification here
  };

  return (
    <div className="space-y-4">
      {/* Tab buttons */}
      <div className="border-b border-gray-200">
        <div className="flex space-x-8">
          {methods.map((method) => (
            <button
              key={method.id}
              onClick={() => setActiveMethod(method.id)}
              className={`pb-4 px-1 border-b-2 font-medium text-sm transition-colors ${
                activeMethod === method.id
                  ? 'border-primary text-primary'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              {method.label}
            </button>
          ))}
        </div>
      </div>

      {/* Current method description */}
      <p className="text-sm text-gray-600">
        {methods.find((m) => m.id === activeMethod)?.description}
      </p>

      {/* Code block */}
      <div className="relative">
        <pre className="bg-gray-50 border border-gray-200 rounded-lg p-4 overflow-x-auto text-sm">
          <code className="text-gray-800">{content.code}</code>
        </pre>

        {/* Copy button */}
        <button
          onClick={copyToClipboard}
          className="absolute top-4 right-4 px-3 py-1.5 bg-white border border-gray-200 rounded-md text-xs font-medium text-gray-700 hover:bg-gray-50 hover:border-gray-300 transition-colors"
        >
          Copy
        </button>
      </div>

      {/* Environment variables if required */}
      {skill.installation.envVars && skill.installation.envVars.length > 0 && (
        <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <h4 className="text-sm font-semibold text-gray-900 mb-2">Required Environment Variables</h4>
          <div className="space-y-2">
            {skill.installation.envVars.map((envVar) => (
              <div key={envVar.name} className="text-sm">
                <code className="text-blue-700 font-mono bg-white px-2 py-0.5 rounded">
                  {envVar.name}
                </code>
                {envVar.required && (
                  <span className="ml-2 text-xs text-red-600 font-medium">Required</span>
                )}
                <p className="text-gray-600 mt-1 ml-1">{envVar.description}</p>
                {envVar.default && (
                  <p className="text-gray-500 text-xs mt-0.5 ml-1">
                    Default: <code className="bg-gray-100 px-1 rounded">{envVar.default}</code>
                  </p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
