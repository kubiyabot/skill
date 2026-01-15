/**
 * Unit tests for skill filtering functionality
 */

import {
  filterSkills,
  filterByCategory,
  filterByType,
  filterByBadge,
  getAvailableCategories,
  getAvailableTypes,
  getAvailableBadges,
  countByCategory,
  countByType,
} from '@/lib/search/filterSkills';
import { SkillCardData } from '@/lib/types';

// Mock skill data for testing
const mockSkills: SkillCardData[] = [
  {
    id: 'kubernetes',
    name: 'Kubernetes',
    type: 'native',
    description: 'Kubernetes cluster management',
    version: '1.0.0',
    author: { name: 'Test' },
    categories: ['devops', 'cloud'],
    badges: ['official', 'featured'],
    installation: { source: './test' },
    toolsCount: 5,
    slug: 'kubernetes',
  },
  {
    id: 'aws',
    name: 'AWS',
    type: 'wasm',
    description: 'Amazon Web Services integration',
    version: '1.0.0',
    author: { name: 'Test' },
    categories: ['cloud'],
    badges: ['official'],
    installation: { source: './test' },
    toolsCount: 10,
    slug: 'aws',
  },
  {
    id: 'github',
    name: 'GitHub',
    type: 'native',
    description: 'GitHub integration',
    version: '1.0.0',
    author: { name: 'Test' },
    categories: ['api', 'development'],
    badges: ['community'],
    installation: { source: './test' },
    toolsCount: 8,
    slug: 'github',
  },
  {
    id: 'ffmpeg',
    name: 'FFmpeg',
    type: 'docker',
    description: 'Video processing',
    version: '1.0.0',
    author: { name: 'Test' },
    categories: ['media', 'utilities'],
    installation: { source: './test' },
    toolsCount: 3,
    slug: 'ffmpeg',
  },
];

describe('filterSkills', () => {
  describe('filterSkills - combined filters', () => {
    it('should return all skills when no filters applied', () => {
      const results = filterSkills(mockSkills, {});
      expect(results).toHaveLength(4);
    });

    it('should filter by single category', () => {
      const results = filterSkills(mockSkills, {
        categories: ['cloud'],
      });
      expect(results).toHaveLength(2);
      expect(results.map((s) => s.id)).toContain('kubernetes');
      expect(results.map((s) => s.id)).toContain('aws');
    });

    it('should filter by multiple categories (OR logic)', () => {
      const results = filterSkills(mockSkills, {
        categories: ['cloud', 'media'],
      });
      expect(results).toHaveLength(3); // kubernetes, aws, ffmpeg
    });

    it('should filter by type', () => {
      const results = filterSkills(mockSkills, {
        types: ['native'],
      });
      expect(results).toHaveLength(2);
      expect(results.map((s) => s.id)).toContain('kubernetes');
      expect(results.map((s) => s.id)).toContain('github');
    });

    it('should filter by multiple types', () => {
      const results = filterSkills(mockSkills, {
        types: ['wasm', 'docker'],
      });
      expect(results).toHaveLength(2);
      expect(results.map((s) => s.id)).toContain('aws');
      expect(results.map((s) => s.id)).toContain('ffmpeg');
    });

    it('should filter by badge', () => {
      const results = filterSkills(mockSkills, {
        badges: ['official'],
      });
      expect(results).toHaveLength(2);
      expect(results.map((s) => s.id)).toContain('kubernetes');
      expect(results.map((s) => s.id)).toContain('aws');
    });

    it('should filter by multiple badges', () => {
      const results = filterSkills(mockSkills, {
        badges: ['official', 'community'],
      });
      expect(results).toHaveLength(3);
    });

    it('should exclude skills without badges when badge filter is applied', () => {
      const results = filterSkills(mockSkills, {
        badges: ['featured'],
      });
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('kubernetes');
    });

    it('should combine multiple filter types (AND logic)', () => {
      const results = filterSkills(mockSkills, {
        categories: ['cloud'],
        types: ['native'],
      });
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('kubernetes');
    });

    it('should return empty array when no matches', () => {
      const results = filterSkills(mockSkills, {
        categories: ['database'], // no skills with this category
      });
      expect(results).toHaveLength(0);
    });

    it('should apply all three filter types together', () => {
      const results = filterSkills(mockSkills, {
        categories: ['cloud'],
        types: ['native'],
        badges: ['official'],
      });
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('kubernetes');
    });
  });

  describe('filterByCategory', () => {
    it('should filter skills by category', () => {
      const results = filterByCategory(mockSkills, 'cloud');
      expect(results).toHaveLength(2);
    });

    it('should return empty array for non-existent category', () => {
      const results = filterByCategory(mockSkills, 'database');
      expect(results).toHaveLength(0);
    });
  });

  describe('filterByType', () => {
    it('should filter skills by type', () => {
      const results = filterByType(mockSkills, 'native');
      expect(results).toHaveLength(2);
    });

    it('should return only docker skills', () => {
      const results = filterByType(mockSkills, 'docker');
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('ffmpeg');
    });
  });

  describe('filterByBadge', () => {
    it('should filter skills by badge', () => {
      const results = filterByBadge(mockSkills, 'official');
      expect(results).toHaveLength(2);
    });

    it('should return empty array for non-existent badge', () => {
      const results = filterByBadge(mockSkills, 'verified');
      expect(results).toHaveLength(0);
    });

    it('should handle skills without badges', () => {
      const results = filterByBadge(mockSkills, 'featured');
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('kubernetes');
    });
  });

  describe('getAvailableCategories', () => {
    it('should return all unique categories', () => {
      const categories = getAvailableCategories(mockSkills);
      expect(categories).toContain('devops');
      expect(categories).toContain('cloud');
      expect(categories).toContain('api');
      expect(categories).toContain('development');
      expect(categories).toContain('media');
      expect(categories).toContain('utilities');
    });

    it('should not duplicate categories', () => {
      const categories = getAvailableCategories(mockSkills);
      const uniqueCategories = [...new Set(categories)];
      expect(categories.length).toBe(uniqueCategories.length);
    });

    it('should return empty array for empty skills', () => {
      const categories = getAvailableCategories([]);
      expect(categories).toHaveLength(0);
    });
  });

  describe('getAvailableTypes', () => {
    it('should return all unique types', () => {
      const types = getAvailableTypes(mockSkills);
      expect(types).toContain('native');
      expect(types).toContain('wasm');
      expect(types).toContain('docker');
    });

    it('should return correct count of types', () => {
      const types = getAvailableTypes(mockSkills);
      expect(types).toHaveLength(3);
    });

    it('should return empty array for empty skills', () => {
      const types = getAvailableTypes([]);
      expect(types).toHaveLength(0);
    });
  });

  describe('getAvailableBadges', () => {
    it('should return all unique badges', () => {
      const badges = getAvailableBadges(mockSkills);
      expect(badges).toContain('official');
      expect(badges).toContain('featured');
      expect(badges).toContain('community');
    });

    it('should handle skills without badges', () => {
      const badges = getAvailableBadges(mockSkills);
      expect(badges.length).toBeGreaterThan(0);
    });

    it('should return empty array for empty skills', () => {
      const badges = getAvailableBadges([]);
      expect(badges).toHaveLength(0);
    });
  });

  describe('countByCategory', () => {
    it('should count skills by category', () => {
      const counts = countByCategory(mockSkills);
      expect(counts.get('cloud')).toBe(2);
      expect(counts.get('devops')).toBe(1);
      expect(counts.get('api')).toBe(1);
    });

    it('should return empty map for empty skills', () => {
      const counts = countByCategory([]);
      expect(counts.size).toBe(0);
    });

    it('should handle skills with multiple categories', () => {
      const counts = countByCategory(mockSkills);
      // kubernetes has both 'devops' and 'cloud'
      expect(counts.get('devops')).toBe(1);
      expect(counts.get('cloud')).toBe(2); // kubernetes + aws
    });
  });

  describe('countByType', () => {
    it('should count skills by type', () => {
      const counts = countByType(mockSkills);
      expect(counts.get('native')).toBe(2);
      expect(counts.get('wasm')).toBe(1);
      expect(counts.get('docker')).toBe(1);
    });

    it('should return empty map for empty skills', () => {
      const counts = countByType([]);
      expect(counts.size).toBe(0);
    });
  });
});
