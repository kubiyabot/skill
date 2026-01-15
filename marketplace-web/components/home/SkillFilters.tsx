/**
 * Filter controls for skills with URL state persistence
 * Supports filtering by type, category, and badges
 */

'use client';

import { useRouter, useSearchParams } from 'next/navigation';
import { SkillType, SkillBadge } from '@/lib/types';
import { cn } from '@/lib/utils/cn';

interface SkillFiltersProps {
  availableCategories: string[];
  categoryCounts?: Record<string, number>;
  onFiltersChange?: (filters: {
    types: SkillType[];
    categories: string[];
    badges: SkillBadge[];
  }) => void;
}

export function SkillFilters({
  availableCategories,
  categoryCounts = {},
  onFiltersChange,
}: SkillFiltersProps) {
  const router = useRouter();
  const searchParams = useSearchParams();

  const activeTypes = (searchParams.getAll('type') as SkillType[]) || [];
  const activeCategories = searchParams.getAll('category') || [];
  const activeBadges = (searchParams.getAll('badge') as SkillBadge[]) || [];

  const toggleFilter = (key: string, value: string) => {
    const params = new URLSearchParams(searchParams.toString());
    const current = params.getAll(key);

    if (current.includes(value)) {
      // Remove the filter
      params.delete(key);
      current
        .filter((v) => v !== value)
        .forEach((v) => params.append(key, v));
    } else {
      // Add the filter
      params.append(key, value);
    }

    router.push(`?${params.toString()}`, { scroll: false });

    // Notify parent of filter change
    if (onFiltersChange) {
      const newTypes = params.getAll('type') as SkillType[];
      const newCategories = params.getAll('category');
      const newBadges = params.getAll('badge') as SkillBadge[];
      onFiltersChange({
        types: newTypes,
        categories: newCategories,
        badges: newBadges,
      });
    }
  };

  const clearFilters = () => {
    router.push('/', { scroll: false });
    if (onFiltersChange) {
      onFiltersChange({ types: [], categories: [], badges: [] });
    }
  };

  const hasActiveFilters =
    activeTypes.length > 0 ||
    activeCategories.length > 0 ||
    activeBadges.length > 0;

  const types: SkillType[] = ['native', 'wasm', 'docker'];
  const badges: SkillBadge[] = ['official', 'verified', 'featured', 'community'];

  return (
    <div className="space-y-8 animate-fade-in [animation-delay:200ms]">
      {/* Header with clear button */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-bold text-white">Filters</h3>
        {hasActiveFilters && (
          <button
            onClick={clearFilters}
            className="text-sm text-primary-400 hover:text-primary-300 font-medium transition-colors"
            type="button"
          >
            Clear all
          </button>
        )}
      </div>

      {/* Type Filters */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-4 uppercase tracking-wider">Type</h4>
        <div className="flex flex-wrap gap-2">
          {types.map((type) => (
            <button
              key={type}
              onClick={() => toggleFilter('type', type)}
              className={cn(
                'px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200 border',
                activeTypes.includes(type)
                  ? 'bg-primary-500/20 text-primary-400 border-primary-500/50 shadow-[0_0_15px_rgba(59,130,246,0.2)]'
                  : 'bg-white/5 text-gray-400 border-white/5 hover:bg-white/10 hover:text-white hover:border-white/10'
              )}
              type="button"
              aria-pressed={activeTypes.includes(type)}
            >
              {type.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      {/* Badge Filters */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-4 uppercase tracking-wider">Badges</h4>
        <div className="flex flex-wrap gap-2">
          {badges.map((badge) => (
            <button
              key={badge}
              onClick={() => toggleFilter('badge', badge)}
              className={cn(
                'px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200 border capitalize',
                activeBadges.includes(badge)
                  ? 'bg-primary-500/20 text-primary-400 border-primary-500/50 shadow-[0_0_15px_rgba(59,130,246,0.2)]'
                  : 'bg-white/5 text-gray-400 border-white/5 hover:bg-white/10 hover:text-white hover:border-white/10'
              )}
              type="button"
              aria-pressed={activeBadges.includes(badge)}
            >
              {badge === 'official' && '⭐ '}
              {badge === 'verified' && '✓ '}
              {badge}
            </button>
          ))}
        </div>
      </div>

      {/* Category Filters */}
      {availableCategories.length > 0 && (
        <div>
          <h4 className="text-sm font-medium text-gray-400 mb-4 uppercase tracking-wider">
            Categories
          </h4>
          <div className="flex flex-wrap gap-2">
            {availableCategories.map((category) => (
              <button
                key={category}
                onClick={() => toggleFilter('category', category)}
                className={cn(
                  'px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200 border',
                  activeCategories.includes(category)
                    ? 'bg-primary-500/20 text-primary-400 border-primary-500/50 shadow-[0_0_15px_rgba(59,130,246,0.2)]'
                    : 'bg-white/5 text-gray-400 border-white/5 hover:bg-white/10 hover:text-white hover:border-white/10'
                )}
                type="button"
                aria-pressed={activeCategories.includes(category)}
              >
                {category}
                {categoryCounts[category] && (
                  <span className="ml-1.5 opacity-50 text-xs">
                    {categoryCounts[category]}
                  </span>
                )}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Active filter count */}
      {hasActiveFilters && (
        <div className="pt-4 border-t border-white/5">
          <p className="text-sm text-gray-500 text-center">
            {activeTypes.length + activeCategories.length + activeBadges.length}{' '}
            {activeTypes.length + activeCategories.length + activeBadges.length ===
              1
              ? 'filter'
              : 'filters'}{' '}
            active
          </p>
        </div>
      )}
    </div>
  );
}
