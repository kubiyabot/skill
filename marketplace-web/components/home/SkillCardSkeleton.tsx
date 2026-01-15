/**
 * Skeleton loader for SkillCard component
 * Matches the structure and dimensions of the actual SkillCard
 */

import { Skeleton } from '@/components/ui/Skeleton';

export function SkillCardSkeleton() {
  return (
    <div className="glass-card h-full flex flex-col p-5">
      {/* Header with icon, title and badges */}
      <div className="flex items-start gap-4 mb-4">
        {/* Icon skeleton */}
        <Skeleton variant="rectangular" width={48} height={48} className="flex-shrink-0 rounded-xl" />

        <div className="flex-grow min-w-0 space-y-2 pt-1">
          {/* Title skeleton */}
          <Skeleton variant="text" width="70%" height={24} />
          {/* Type badge skeleton */}
          <Skeleton variant="text" width={60} height={20} className="rounded" />
        </div>
      </div>

      {/* Description skeleton */}
      <div className="space-y-2 mb-6 flex-grow">
        <Skeleton variant="text" width="100%" />
        <Skeleton variant="text" width="85%" />
      </div>

      {/* Footer metadata skeleton */}
      <div className="flex items-center justify-between pt-4 border-t border-white/5 mt-auto">
        <Skeleton variant="text" width={60} height={16} />
        <Skeleton variant="text" width={80} height={16} />
      </div>
    </div>
  );
}

/**
 * Grid of skeleton cards for loading states
 */
export function SkillCardSkeletonGrid({ count = 6 }: { count?: number }) {
  return (
    <>
      {Array.from({ length: count }).map((_, i) => (
        <SkillCardSkeleton key={i} />
      ))}
    </>
  );
}
