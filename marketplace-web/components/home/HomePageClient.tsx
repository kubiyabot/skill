/**
 * Client-side homepage component that handles search and filtering
 * Uses URL search params for state persistence (works with static export)
 */

'use client';

import { Suspense, useMemo } from 'react';
import { useSearchParams } from 'next/navigation';
import { SkillCardData, SkillType, SkillBadge, CategoryId } from '@/lib/types';
import { SkillSearch } from '@/components/home/SkillSearch';
import { SkillFilters } from '@/components/home/SkillFilters';
import { SkillGrid } from '@/components/home/SkillGrid';
import { SkillCardSkeletonGrid } from '@/components/home/SkillCardSkeleton';
import { createSkillSearchIndex, searchSkills } from '@/lib/search/skillSearch';
import { filterSkills } from '@/lib/search/filterSkills';

interface HomePageClientProps {
  skills: SkillCardData[];
  categories: string[];
  categoryCounts: Record<string, number>;
}

export function HomePageClient({
  skills,
  categories,
  categoryCounts,
}: HomePageClientProps) {
  const searchParams = useSearchParams();

  // Parse URL params for filters
  const searchQuery = searchParams.get('q') || '';

  const types = useMemo(() => {
    const typeParams = searchParams.getAll('type');
    return typeParams as SkillType[];
  }, [searchParams]);

  const categoryFilters = useMemo(() => {
    return searchParams.getAll('category') as CategoryId[];
  }, [searchParams]);

  const badges = useMemo(() => {
    const badgeParams = searchParams.getAll('badge');
    return badgeParams as SkillBadge[];
  }, [searchParams]);

  // Apply search first if there's a query
  const displaySkills = useMemo(() => {
    if (!searchQuery.trim()) return skills;
    const searchIndex = createSkillSearchIndex(skills);
    return searchSkills(searchIndex, searchQuery);
  }, [skills, searchQuery]);

  // Then apply filters
  const filteredSkills = useMemo(() => {
    return filterSkills(displaySkills, {
      types,
      categories: categoryFilters,
      badges,
    });
  }, [displaySkills, types, categoryFilters, badges]);

  const hasActiveFilters =
    types.length > 0 ||
    categoryFilters.length > 0 ||
    badges.length > 0 ||
    searchQuery.length > 0;

  return (
    <>
      {/* Search Bar */}
      <div className="-mt-8 relative z-20 mb-12 flex justify-center px-4">
        <SkillSearch skills={skills} />
      </div>

      {/* Main Content */}
      <div className="mb-24">
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar Filters */}
          <aside className="hidden lg:block w-64 flex-shrink-0">
            <div className="sticky top-8">
              <SkillFilters
                availableCategories={categories}
                categoryCounts={categoryCounts}
              />
            </div>
          </aside>

          {/* Skills Grid */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center justify-between mb-8 pb-4 border-b border-white/5">
              <h2 className="text-2xl font-bold text-white flex items-center gap-2">
                {hasActiveFilters ? 'Filtered Skills' : 'All Skills'}
                <span className="text-sm font-normal text-gray-500 bg-white/5 px-2 py-0.5 rounded-full ml-2">
                  {filteredSkills.length}
                </span>
              </h2>
            </div>

            <Suspense fallback={<SkillCardSkeletonGrid count={12} />}>
              <SkillGrid skills={filteredSkills} />
            </Suspense>
          </div>
        </div>
      </div>
    </>
  );
}
