import { notFound } from 'next/navigation';
import { Metadata } from 'next';
import Link from 'next/link';
import {
  ArrowLeft,
  Star,
  GitBranch,
  ExternalLink,
  Terminal,
  Book,
  Layers,
  Package,
} from 'lucide-react';
import {
  loadSkillById,
  getAllSkillIds,
  getRelatedSkills,
} from '@/lib/data/loadSkills';
import { SkillGrid } from '@/components/home/SkillGrid';
import SkillTabs from '@/components/detail/SkillTabs';

// Export these functions for static generation
export async function generateStaticParams() {
  const ids = await getAllSkillIds();
  return ids.map((id) => ({ id }));
}

export async function generateMetadata({
  params,
}: {
  params: { id: string };
}): Promise<Metadata> {
  const skill = await loadSkillById(params.id);

  if (!skill) {
    return {
      title: 'Skill Not Found',
    };
  }

  return {
    title: `${skill.name} - Agentic Skills Library`,
    description: skill.description,
  };
}

// Type badge colors
const getTypeBadgeStyles = (type: string) => {
  switch (type) {
    case 'wasm':
      return 'bg-purple-100 text-purple-700 border-purple-200';
    case 'native':
      return 'bg-blue-100 text-blue-700 border-blue-200';
    case 'docker':
      return 'bg-cyan-100 text-cyan-700 border-cyan-200';
    default:
      return 'bg-gray-100 text-gray-700 border-gray-200';
  }
};

// Type icons
const getTypeIcon = (type: string) => {
  switch (type) {
    case 'wasm':
      return <Package className="w-4 h-4" />;
    case 'native':
      return <Terminal className="w-4 h-4" />;
    case 'docker':
      return <Layers className="w-4 h-4" />;
    default:
      return <Package className="w-4 h-4" />;
  }
};

export default async function SkillDetailPage({
  params,
}: {
  params: { id: string };
}) {
  const skill = await loadSkillById(params.id);

  if (!skill) {
    notFound();
  }

  const relatedSkills = await getRelatedSkills(params.id);

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Breadcrumb */}
        <Link
          href="/"
          className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-primary transition-colors mb-8 group"
        >
          <ArrowLeft className="w-4 h-4 group-hover:-translate-x-1 transition-transform" />
          Back to all skills
        </Link>

        {/* Header Card */}
        <div className="card mb-8">
          <div className="flex items-start justify-between mb-6">
            <div className="flex-grow">
              <div className="flex items-center gap-4 mb-4">
                {/* Skill Icon */}
                {skill.icon && (
                  <div className="flex-shrink-0 w-16 h-16 rounded-xl bg-gray-50 border border-gray-200 flex items-center justify-center p-3">
                    <img
                      src={skill.icon}
                      alt={`${skill.name} icon`}
                      className="w-full h-full object-contain"
                    />
                  </div>
                )}

                <div className="flex-grow">
                  <div className="flex items-center gap-3 mb-2">
                    <h1 className="text-4xl font-bold text-gray-900">
                      {skill.name}
                    </h1>
                    <div className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-sm font-semibold border ${getTypeBadgeStyles(skill.type)}`}>
                      {getTypeIcon(skill.type)}
                      {skill.type.toUpperCase()}
                    </div>
                  </div>
                </div>
              </div>
              <p className="text-lg text-gray-600 leading-relaxed mb-4">
                {skill.description}
              </p>
              <div className="flex flex-wrap items-center gap-4 text-sm text-gray-500">
                <span className="flex items-center gap-1.5">
                  <GitBranch className="w-4 h-4" />
                  by{' '}
                  {skill.author.github ? (
                    <a
                      href={`https://github.com/${skill.author.github}`}
                      className="text-primary hover:text-blue-700 font-medium"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      {skill.author.name}
                    </a>
                  ) : (
                    <span className="font-medium text-gray-700">{skill.author.name}</span>
                  )}
                </span>
                <span className="text-gray-300">•</span>
                <span className="font-mono text-xs bg-gray-100 px-2 py-1 rounded">
                  v{skill.version}
                </span>
                {skill.tools && skill.tools.length > 0 && (
                  <>
                    <span className="text-gray-300">•</span>
                    <span className="flex items-center gap-1.5 font-medium text-gray-700">
                      <Terminal className="w-4 h-4" />
                      {skill.tools.length} tools
                    </span>
                  </>
                )}
              </div>
            </div>
            {skill.badges?.includes('official') && (
              <div className="ml-6">
                <span className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-semibold bg-blue-50 text-blue-700 border border-blue-200">
                  <Star className="w-4 h-4 fill-current" />
                  Official
                </span>
              </div>
            )}
          </div>
        </div>

        {/* Two column layout */}
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          {/* Main content - 3 columns */}
          <div className="lg:col-span-3">
            <SkillTabs skill={skill} />
          </div>

          {/* Sidebar - 1 column */}
          <div className="space-y-6">
            {/* Quick Links */}
            {skill.links && (
              <div className="card">
                <h3 className="text-base font-semibold text-gray-900 mb-4 flex items-center gap-2">
                  <ExternalLink className="w-5 h-5" />
                  Links
                </h3>
                <div className="space-y-3">
                  {skill.links.github && (
                    <a
                      href={skill.links.github}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-2 text-sm text-primary hover:text-blue-700 transition-colors font-medium group"
                    >
                      <GitBranch className="w-4 h-4" />
                      <span>GitHub Repository</span>
                      <ExternalLink className="w-3 h-3 ml-auto opacity-0 group-hover:opacity-100 transition-opacity" />
                    </a>
                  )}
                  {skill.links.documentation && (
                    <a
                      href={skill.links.documentation}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-2 text-sm text-primary hover:text-blue-700 transition-colors font-medium group"
                    >
                      <Book className="w-4 h-4" />
                      <span>Documentation</span>
                      <ExternalLink className="w-3 h-3 ml-auto opacity-0 group-hover:opacity-100 transition-opacity" />
                    </a>
                  )}
                  {skill.links.homepage && (
                    <a
                      href={skill.links.homepage}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-2 text-sm text-primary hover:text-blue-700 transition-colors font-medium group"
                    >
                      <ExternalLink className="w-4 h-4" />
                      <span>Homepage</span>
                      <ExternalLink className="w-3 h-3 ml-auto opacity-0 group-hover:opacity-100 transition-opacity" />
                    </a>
                  )}
                </div>
              </div>
            )}

            {/* Requirements */}
            {skill.requirements && (
              <div className="card">
                <h3 className="text-base font-semibold text-gray-900 mb-4">
                  Requirements
                </h3>
                <div className="space-y-4 text-sm">
                  {skill.requirements.cli && skill.requirements.cli.length > 0 && (
                    <div>
                      <p className="font-medium text-gray-700 mb-2 flex items-center gap-2">
                        <Terminal className="w-4 h-4" />
                        CLI Tools
                      </p>
                      <div className="space-y-2">
                        {skill.requirements.cli.map((cli) => (
                          <div
                            key={cli}
                            className="flex items-center gap-2 text-gray-600"
                          >
                            <div className="w-1.5 h-1.5 rounded-full bg-gray-400" />
                            <code className="text-xs bg-gray-100 px-2 py-1 rounded font-mono">
                              {cli}
                            </code>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                  {skill.requirements.platform &&
                    skill.requirements.platform.length > 0 && (
                      <div>
                        <p className="font-medium text-gray-700 mb-2 flex items-center gap-2">
                          <Layers className="w-4 h-4" />
                          Platforms
                        </p>
                        <div className="flex flex-wrap gap-2">
                          {skill.requirements.platform.map((platform) => (
                            <span
                              key={platform}
                              className="px-2.5 py-1 bg-gray-100 text-gray-700 rounded-md text-xs font-medium capitalize border border-gray-200"
                            >
                              {platform}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}
                </div>
              </div>
            )}

            {/* Categories */}
            <div className="card">
              <h3 className="text-base font-semibold text-gray-900 mb-4">
                Categories
              </h3>
              <div className="flex flex-wrap gap-2">
                {skill.categories.map((category) => (
                  <span
                    key={category}
                    className="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-full text-xs font-medium capitalize border border-gray-200 hover:border-gray-300 transition-colors"
                  >
                    {category}
                  </span>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Related Skills */}
        {relatedSkills.length > 0 && (
          <div className="mt-16 pt-16 border-t border-gray-200">
            <h2 className="text-2xl font-bold text-gray-900 mb-6">
              Related Skills
            </h2>
            <SkillGrid skills={relatedSkills} />
          </div>
        )}
      </div>
    </div>
  );
}
