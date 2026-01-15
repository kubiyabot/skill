/**
 * Skill search functionality using Fuse.js for fuzzy searching
 */

import Fuse, { IFuseOptions } from 'fuse.js';
import { SkillCardData } from '@/lib/types';

/**
 * Configuration for Fuse.js search
 * - keys: Fields to search with their relative weights
 * - threshold: 0.0 = exact match, 1.0 = match anything (0.3 is a good balance)
 * - includeScore: Include match score in results
 * - minMatchCharLength: Minimum character length for a match
 */
const fuseOptions: IFuseOptions<SkillCardData> = {
  keys: [
    { name: 'name', weight: 0.4 },
    { name: 'description', weight: 0.3 },
    { name: 'tools.name', weight: 0.2 },
    { name: 'tools.description', weight: 0.1 },
  ],
  threshold: 0.3,
  includeScore: true,
  minMatchCharLength: 2,
  ignoreLocation: true, // Search entire string, not just beginning
  useExtendedSearch: false,
};

/**
 * Create a Fuse.js search index from skills data
 */
export function createSkillSearchIndex(skills: SkillCardData[]): Fuse<SkillCardData> {
  return new Fuse(skills, fuseOptions);
}

/**
 * Search skills using the provided search index
 * @param searchIndex - Fuse.js search index created from skills
 * @param query - Search query string
 * @returns Array of matching skills (empty if no query)
 */
export function searchSkills(
  searchIndex: Fuse<SkillCardData>,
  query: string
): SkillCardData[] {
  // Return empty array if no query provided
  if (!query.trim()) {
    return [];
  }

  const results = searchIndex.search(query);
  return results.map((result) => result.item);
}

/**
 * Search skills directly without creating an index first
 * Useful for one-off searches or when the skill list is small
 */
export function searchSkillsDirect(
  skills: SkillCardData[],
  query: string
): SkillCardData[] {
  if (!query.trim()) {
    return [];
  }

  const fuse = createSkillSearchIndex(skills);
  return searchSkills(fuse, query);
}

/**
 * Get search suggestions based on partial query
 * Returns top 5 matching skill names
 */
export function getSearchSuggestions(
  skills: SkillCardData[],
  query: string,
  limit: number = 5
): string[] {
  if (!query.trim() || query.length < 2) {
    return [];
  }

  const results = searchSkillsDirect(skills, query);
  return results.slice(0, limit).map((skill) => skill.name);
}
