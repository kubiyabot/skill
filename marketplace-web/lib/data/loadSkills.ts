/**
 * Data loading utilities for Skills Marketplace
 * Loads and parses skill manifests at build time
 */

import fs from 'fs';
import path from 'path';
import { SkillManifest, SkillCardData, CategoriesData } from '@/lib/types';

const MARKETPLACE_DIR = path.join(process.cwd(), 'data');
const SKILLS_DIR = path.join(MARKETPLACE_DIR, 'skills');
const CATEGORIES_FILE = path.join(MARKETPLACE_DIR, 'categories.json');

/**
 * Load all skill manifests from the marketplace/skills directory
 * Enriches data with computed fields (toolsCount, slug)
 * Sorts by featured status, then alphabetically
 */
export async function loadAllSkills(): Promise<SkillCardData[]> {
  if (!fs.existsSync(SKILLS_DIR)) {
    console.warn('Skills directory not found:', SKILLS_DIR);
    return [];
  }

  const files = fs.readdirSync(SKILLS_DIR).filter((f) => f.endsWith('.json'));

  const skills: SkillCardData[] = files.map((file) => {
    const filePath = path.join(SKILLS_DIR, file);
    const content = fs.readFileSync(filePath, 'utf8');
    const skill = JSON.parse(content) as SkillManifest;

    // Enrich with computed fields
    return {
      ...skill,
      toolsCount: skill.tools?.length || 0,
      slug: skill.id,
    };
  });

  // Sort: featured first, then official, then alphabetically
  return skills.sort((a, b) => {
    const aFeatured = a.badges?.includes('featured') ? 1 : 0;
    const bFeatured = b.badges?.includes('featured') ? 1 : 0;
    if (aFeatured !== bFeatured) return bFeatured - aFeatured;

    const aOfficial = a.badges?.includes('official') ? 1 : 0;
    const bOfficial = b.badges?.includes('official') ? 1 : 0;
    if (aOfficial !== bOfficial) return bOfficial - aOfficial;

    return a.name.localeCompare(b.name);
  });
}

/**
 * Load a single skill manifest by ID
 * Returns null if not found
 * Also loads SKILL.md content if skillMdPath is provided
 */
export async function loadSkillById(
  id: string
): Promise<SkillManifest | null> {
  const filePath = path.join(SKILLS_DIR, `${id}.json`);

  if (!fs.existsSync(filePath)) {
    return null;
  }

  const content = fs.readFileSync(filePath, 'utf8');
  const skill = JSON.parse(content) as SkillManifest;

  // Load SKILL.md content if path is provided
  if (skill.skillMdPath) {
    const skillMdPath = path.join(MARKETPLACE_DIR, skill.skillMdPath);
    if (fs.existsSync(skillMdPath)) {
      try {
        let skillMdContent = fs.readFileSync(skillMdPath, 'utf8');
        // Remove frontmatter if present
        skillMdContent = skillMdContent.replace(/^---\n[\s\S]*?\n---\n/, '');
        skill.skillMdContent = skillMdContent;
      } catch (err) {
        console.warn(`Failed to load SKILL.md for ${id}:`, err);
      }
    }
  }

  return skill;
}

/**
 * Get all skill IDs for static path generation
 */
export async function getAllSkillIds(): Promise<string[]> {
  if (!fs.existsSync(SKILLS_DIR)) {
    return [];
  }

  const files = fs.readdirSync(SKILLS_DIR).filter((f) => f.endsWith('.json'));
  return files.map((file) => path.basename(file, '.json'));
}

/**
 * Load categories configuration
 */
export async function loadCategories(): Promise<CategoriesData> {
  if (!fs.existsSync(CATEGORIES_FILE)) {
    console.warn('Categories file not found:', CATEGORIES_FILE);
    return { categories: [] };
  }

  const content = fs.readFileSync(CATEGORIES_FILE, 'utf8');
  return JSON.parse(content) as CategoriesData;
}

/**
 * Get skill count by category
 */
export async function getSkillCountsByCategory(): Promise<
  Record<string, number>
> {
  const skills = await loadAllSkills();
  const counts: Record<string, number> = {};

  skills.forEach((skill) => {
    skill.categories.forEach((category) => {
      counts[category] = (counts[category] || 0) + 1;
    });
  });

  return counts;
}

/**
 * Get related skills (same category, excluding current skill)
 * Returns up to 4 related skills
 */
export async function getRelatedSkills(
  skillId: string,
  limit: number = 4
): Promise<SkillCardData[]> {
  const currentSkill = await loadSkillById(skillId);
  if (!currentSkill) return [];

  const allSkills = await loadAllSkills();

  // Filter skills that share at least one category
  const related = allSkills
    .filter((skill) => {
      if (skill.id === skillId) return false;
      return skill.categories.some((cat) =>
        currentSkill.categories.includes(cat)
      );
    })
    .slice(0, limit);

  return related;
}

/**
 * Get statistics about the marketplace
 */
export async function getMarketplaceStats() {
  const skills = await loadAllSkills();
  const categories = await loadCategories();

  const typeCount = skills.reduce(
    (acc, skill) => {
      acc[skill.type] = (acc[skill.type] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  return {
    totalSkills: skills.length,
    totalCategories: categories.categories.length,
    totalTools: skills.reduce((sum, skill) => sum + (skill.toolsCount || 0), 0),
    byType: typeCount,
    featuredCount: skills.filter((s) => s.badges?.includes('featured')).length,
    officialCount: skills.filter((s) => s.badges?.includes('official')).length,
  };
}
