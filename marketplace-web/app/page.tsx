import { Suspense } from 'react';
import {
  loadAllSkills,
  getMarketplaceStats,
  loadCategories,
  getSkillCountsByCategory,
} from '@/lib/data/loadSkills';
import { HeroSection } from '@/components/home/HeroSection';
import { HomePageClient } from '@/components/home/HomePageClient';
import { SkillCardSkeletonGrid } from '@/components/home/SkillCardSkeleton';

export default async function HomePage() {
  const skills = await loadAllSkills();
  const stats = await getMarketplaceStats();
  const categories = await loadCategories();
  const categoryCounts = await getSkillCountsByCategory();

  return (
    <main className="min-h-screen bg-dark-900">
      <HeroSection stats={stats} />
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        <Suspense fallback={<SkillCardSkeletonGrid count={12} />}>
          <HomePageClient
            skills={skills}
            categories={categories.categories.map((c) => c.id)}
            categoryCounts={categoryCounts}
          />
        </Suspense>
      </div>
    </main>
  );
}
