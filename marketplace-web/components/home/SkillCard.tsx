import Link from 'next/link';
import Image from 'next/image';
import { SkillCardData } from '@/lib/types';

interface SkillCardProps {
  skill: SkillCardData;
}

export function SkillCard({ skill }: SkillCardProps) {
  return (
    <Link href={`/skills/${skill.slug}`} className="block h-full group">
      <div className="glass-card glass-card-hover h-full flex flex-col p-5 relative overflow-hidden">
        {/* Glow effect on hover */}
        <div className="absolute top-0 right-0 w-32 h-32 bg-primary-500/10 rounded-full blur-2xl -translate-y-16 translate-x-16 group-hover:bg-primary-500/20 transition-colors duration-500" />

        {/* Header with icon, title and badges */}
        <div className="flex items-start gap-4 mb-4 relative z-10">
          {/* Icon */}
          {skill.icon && (
            <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-dark-900/50 border border-white/10 flex items-center justify-center p-2.5 shadow-inner">
              <img
                src={skill.icon}
                alt={`${skill.name} icon`}
                className="w-full h-full object-contain filter drop-shadow-lg"
              />
            </div>
          )}

          <div className="flex-grow min-w-0 pt-1">
            <div className="flex items-center justify-between gap-2 mb-1">
              <h3 className="text-lg font-bold text-gray-100 group-hover:text-primary-400 transition-colors truncate">
                {skill.name}
              </h3>
              {skill.badges?.includes('official') && (
                <span className="badge badge-primary flex-shrink-0">
                  Official
                </span>
              )}
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs font-mono text-gray-500 bg-white/5 px-1.5 py-0.5 rounded border border-white/5">
                {skill.type}
              </span>
              {skill.badges?.includes('verified') && !skill.badges.includes('official') && (
                <span className="text-[10px] text-green-400 border border-green-500/20 bg-green-500/10 px-1.5 py-0.5 rounded-full">
                  Verified
                </span>
              )}
            </div>
          </div>
        </div>

        {/* Description */}
        <p className="text-sm text-gray-400 mb-6 flex-grow line-clamp-2 leading-relaxed relative z-10">
          {skill.description}
        </p>

        {/* Footer metadata */}
        <div className="flex items-center justify-between text-xs text-gray-500 pt-4 border-t border-white/5 mt-auto relative z-10">
          <div className="flex items-center gap-1.5">
            <svg className="w-3.5 h-3.5 opacity-70" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
            </svg>
            <span className="font-medium group-hover:text-gray-300 transition-colors">{skill.toolsCount} tools</span>
          </div>
          <div className="flex items-center gap-1.5 opacity-60 hover:opacity-100 transition-opacity">
            <span className="truncate max-w-[100px]">{skill.author.name}</span>
          </div>
        </div>
      </div>
    </Link>
  );
}
