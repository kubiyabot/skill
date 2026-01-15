/**
 * Skill filtering functionality
 * Filters skills by categories, types, and badges
 */

import { SkillCardData, SearchFilters, CategoryId, SkillType, SkillBadge } from '@/lib/types';

/**
 * Filter skills based on provided criteria
 * All filters are AND-ed together (skill must match all active filters)
 * Within each filter type, options are OR-ed (skill must match at least one)
 */
export function filterSkills(
  skills: SkillCardData[],
  filters: Partial<SearchFilters>
): SkillCardData[] {
  return skills.filter((skill) => {
    // Filter by categories (skill must have at least one matching category)
    if (filters.categories?.length) {
      const hasMatchingCategory = filters.categories.some((category) =>
        skill.categories.includes(category)
      );
      if (!hasMatchingCategory) {
        return false;
      }
    }

    // Filter by type (skill type must match one of the selected types)
    if (filters.types?.length) {
      if (!filters.types.includes(skill.type)) {
        return false;
      }
    }

    // Filter by badges (skill must have at least one matching badge)
    if (filters.badges?.length) {
      if (!skill.badges || skill.badges.length === 0) {
        return false;
      }
      const hasMatchingBadge = filters.badges.some((badge) =>
        skill.badges?.includes(badge)
      );
      if (!hasMatchingBadge) {
        return false;
      }
    }

    return true;
  });
}

/**
 * Filter skills by a single category
 */
export function filterByCategory(
  skills: SkillCardData[],
  category: CategoryId
): SkillCardData[] {
  return skills.filter((skill) => skill.categories.includes(category));
}

/**
 * Filter skills by a single type
 */
export function filterByType(
  skills: SkillCardData[],
  type: SkillType
): SkillCardData[] {
  return skills.filter((skill) => skill.type === type);
}

/**
 * Filter skills by a single badge
 */
export function filterByBadge(
  skills: SkillCardData[],
  badge: SkillBadge
): SkillCardData[] {
  return skills.filter((skill) => skill.badges?.includes(badge));
}

/**
 * Get all unique categories from a list of skills
 */
export function getAvailableCategories(skills: SkillCardData[]): CategoryId[] {
  const categoriesSet = new Set<CategoryId>();
  skills.forEach((skill) => {
    skill.categories.forEach((category) => categoriesSet.add(category));
  });
  return Array.from(categoriesSet);
}

/**
 * Get all unique types from a list of skills
 */
export function getAvailableTypes(skills: SkillCardData[]): SkillType[] {
  const typesSet = new Set<SkillType>();
  skills.forEach((skill) => typesSet.add(skill.type));
  return Array.from(typesSet);
}

/**
 * Get all unique badges from a list of skills
 */
export function getAvailableBadges(skills: SkillCardData[]): SkillBadge[] {
  const badgesSet = new Set<SkillBadge>();
  skills.forEach((skill) => {
    skill.badges?.forEach((badge) => badgesSet.add(badge));
  });
  return Array.from(badgesSet);
}

/**
 * Count skills by category
 */
export function countByCategory(skills: SkillCardData[]): Map<CategoryId, number> {
  const counts = new Map<CategoryId, number>();
  skills.forEach((skill) => {
    skill.categories.forEach((category) => {
      counts.set(category, (counts.get(category) || 0) + 1);
    });
  });
  return counts;
}

/**
 * Count skills by type
 */
export function countByType(skills: SkillCardData[]): Map<SkillType, number> {
  const counts = new Map<SkillType, number>();
  skills.forEach((skill) => {
    counts.set(skill.type, (counts.get(skill.type) || 0) + 1);
  });
  return counts;
}
