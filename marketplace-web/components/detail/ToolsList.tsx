'use client';

import { useState } from 'react';
import { Tool } from '@/lib/types';

interface ToolsListProps {
  tools: Tool[];
  skillId: string;
}

export default function ToolsList({ tools, skillId }: ToolsListProps) {
  const [expandedTools, setExpandedTools] = useState<Set<string>>(new Set());

  const toggleTool = (toolName: string) => {
    const newExpanded = new Set(expandedTools);
    if (newExpanded.has(toolName)) {
      newExpanded.delete(toolName);
    } else {
      newExpanded.add(toolName);
    }
    setExpandedTools(newExpanded);
  };

  const getParameterTypeColor = (type: string) => {
    switch (type) {
      case 'string':
        return 'text-green-700 bg-green-50';
      case 'number':
        return 'text-blue-700 bg-blue-50';
      case 'boolean':
        return 'text-purple-700 bg-purple-50';
      case 'object':
        return 'text-orange-700 bg-orange-50';
      case 'array':
        return 'text-pink-700 bg-pink-50';
      default:
        return 'text-gray-700 bg-gray-50';
    }
  };

  return (
    <div className="space-y-3">
      {tools.map((tool) => {
        const isExpanded = expandedTools.has(tool.name);
        const hasParameters = tool.parameters && tool.parameters.length > 0;

        return (
          <div
            key={tool.name}
            className="border border-gray-200 rounded-lg overflow-hidden bg-white"
          >
            {/* Tool header - clickable */}
            <button
              onClick={() => toggleTool(tool.name)}
              className="w-full px-4 py-3 flex items-start justify-between hover:bg-gray-50 transition-colors text-left"
            >
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-1">
                  <code className="text-sm font-semibold text-gray-900 font-mono">
                    {tool.name}
                  </code>
                  {hasParameters && tool.parameters && (
                    <span className="text-xs text-gray-500">
                      {tool.parameters.length} parameter{tool.parameters.length !== 1 ? 's' : ''}
                    </span>
                  )}
                </div>
                <p className="text-sm text-gray-600">{tool.description}</p>
              </div>

              {/* Expand/collapse icon */}
              <svg
                className={`w-5 h-5 text-gray-400 transition-transform flex-shrink-0 ml-3 ${
                  isExpanded ? 'rotate-180' : ''
                }`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>

            {/* Tool details - expandable */}
            {isExpanded && hasParameters && tool.parameters && (
              <div className="px-4 pb-4 border-t border-gray-100 bg-gray-50">
                <div className="mt-3 space-y-3">
                  {tool.parameters.map((param) => (
                    <div key={param.name} className="bg-white p-3 rounded border border-gray-200">
                      <div className="flex items-start justify-between mb-2">
                        <code className="text-sm font-medium text-gray-900 font-mono">
                          {param.name}
                        </code>
                        <div className="flex items-center gap-2">
                          <span
                            className={`text-xs px-2 py-0.5 rounded font-medium ${getParameterTypeColor(
                              param.type
                            )}`}
                          >
                            {param.type}
                          </span>
                          {param.required && (
                            <span className="text-xs px-2 py-0.5 rounded font-medium text-red-700 bg-red-50">
                              required
                            </span>
                          )}
                        </div>
                      </div>

                      <p className="text-sm text-gray-600 mb-2">{param.description}</p>

                      {/* Additional parameter info */}
                      <div className="flex flex-wrap gap-3 text-xs text-gray-500">
                        {param.default !== undefined && (
                          <div>
                            <span className="font-medium">Default:</span>{' '}
                            <code className="bg-gray-100 px-1 rounded">
                              {JSON.stringify(param.default)}
                            </code>
                          </div>
                        )}
                        {param.enum && param.enum.length > 0 && (
                          <div>
                            <span className="font-medium">Options:</span>{' '}
                            {param.enum.map((val, idx) => (
                              <code
                                key={idx}
                                className="bg-gray-100 px-1 rounded ml-1"
                              >
                                {val}
                              </code>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>

                {/* Usage example */}
                <div className="mt-4 p-3 bg-white border border-gray-200 rounded">
                  <p className="text-xs font-semibold text-gray-700 mb-2">Example Usage:</p>
                  <code className="text-xs text-gray-800 font-mono break-all">
                    skill run {skillId}:{tool.name}
                    {tool.parameters
                      .filter((p) => p.required)
                      .map((p) => ` ${p.name}=<value>`)
                      .join('')}
                  </code>
                </div>
              </div>
            )}

            {/* No parameters message */}
            {isExpanded && !hasParameters && (
              <div className="px-4 pb-3 border-t border-gray-100 bg-gray-50">
                <p className="text-sm text-gray-500 mt-2">No parameters required</p>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
