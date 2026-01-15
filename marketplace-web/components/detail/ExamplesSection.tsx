'use client';

import { Example } from '@/lib/types';

interface ExamplesSectionProps {
  examples: Example[];
}

export default function ExamplesSection({ examples }: ExamplesSectionProps) {
  const copyToClipboard = (code: string) => {
    navigator.clipboard.writeText(code);
  };

  if (!examples || examples.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        <p>No examples available yet.</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {examples.map((example, idx) => (
        <div key={idx} className="border border-gray-200 rounded-lg overflow-hidden bg-white">
          {/* Example header */}
          <div className="px-4 py-3 bg-gray-50 border-b border-gray-200">
            <h4 className="text-sm font-semibold text-gray-900">{example.title}</h4>
            {example.description && (
              <p className="text-xs text-gray-600 mt-1">{example.description}</p>
            )}
          </div>

          {/* Code block */}
          <div className="relative">
            <pre className="p-4 overflow-x-auto text-sm bg-white">
              <code className="text-gray-800 font-mono">{example.code}</code>
            </pre>

            {/* Copy button */}
            <button
              onClick={() => copyToClipboard(example.code)}
              className="absolute top-3 right-3 px-2.5 py-1 bg-gray-50 border border-gray-200 rounded text-xs font-medium text-gray-700 hover:bg-gray-100 hover:border-gray-300 transition-colors"
            >
              Copy
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
