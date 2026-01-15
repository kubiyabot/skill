'use client';

import { useEffect, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { FileText, Loader2, AlertCircle } from 'lucide-react';

interface SkillMdViewerProps {
  skillMdUrl: string;
  skillName: string;
}

export default function SkillMdViewer({ skillMdUrl, skillName }: SkillMdViewerProps) {
  const [content, setContent] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchSkillMd() {
      try {
        setLoading(true);
        setError(null);

        const response = await fetch(skillMdUrl);
        if (!response.ok) {
          throw new Error(`Failed to fetch SKILL.md: ${response.statusText}`);
        }

        const text = await response.text();

        // Remove frontmatter if present
        const withoutFrontmatter = text.replace(/^---\n[\s\S]*?\n---\n/, '');

        setContent(withoutFrontmatter);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load SKILL.md');
      } finally {
        setLoading(false);
      }
    }

    fetchSkillMd();
  }, [skillMdUrl]);

  if (loading) {
    return (
      <div className="card">
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 text-gray-400 animate-spin" />
          <span className="ml-3 text-gray-600">Loading SKILL.md...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="card border-red-200 bg-red-50">
        <div className="flex items-start gap-3 text-red-800">
          <AlertCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <div>
            <p className="font-semibold">Failed to load SKILL.md</p>
            <p className="text-sm mt-1">{error}</p>
          </div>
        </div>
      </div>
    );
  }

  return (
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
          {content}
        </ReactMarkdown>
      </div>
    </div>
  );
}
