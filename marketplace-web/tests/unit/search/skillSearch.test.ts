/**
 * Unit tests for skill search functionality
 */

import {
  createSkillSearchIndex,
  searchSkills,
  searchSkillsDirect,
  getSearchSuggestions,
} from '@/lib/search/skillSearch';
import { SkillCardData } from '@/lib/types';

// Mock skill data for testing
const mockSkills: SkillCardData[] = [
  {
    id: 'kubernetes',
    name: 'Kubernetes',
    type: 'native',
    description: 'Manage Kubernetes clusters with kubectl',
    version: '1.0.0',
    author: { name: 'Test Author' },
    categories: ['devops'],
    installation: { source: './test' },
    tools: [
      { name: 'get', description: 'Get Kubernetes resources like pods and deployments' },
      { name: 'apply', description: 'Apply configuration to cluster' },
    ],
    toolsCount: 2,
    slug: 'kubernetes',
  },
  {
    id: 'aws',
    name: 'AWS',
    type: 'wasm',
    description: 'Interact with Amazon Web Services',
    version: '1.0.0',
    author: { name: 'Test Author' },
    categories: ['cloud'],
    installation: { source: './test' },
    tools: [
      { name: 's3-list', description: 'List S3 buckets' },
      { name: 'ec2-list', description: 'List EC2 instances' },
    ],
    toolsCount: 2,
    slug: 'aws',
  },
  {
    id: 'github',
    name: 'GitHub',
    type: 'native',
    description: 'Manage GitHub repositories and issues',
    version: '1.0.0',
    author: { name: 'Test Author' },
    categories: ['api', 'development'],
    installation: { source: './test' },
    tools: [
      { name: 'list-repos', description: 'List repositories' },
      { name: 'create-issue', description: 'Create a new issue' },
    ],
    toolsCount: 2,
    slug: 'github',
  },
];

describe('skillSearch', () => {
  describe('createSkillSearchIndex', () => {
    it('should create a Fuse.js search index', () => {
      const index = createSkillSearchIndex(mockSkills);
      expect(index).toBeDefined();
      expect(index.search).toBeDefined();
    });

    it('should create an empty index for empty skills array', () => {
      const index = createSkillSearchIndex([]);
      expect(index).toBeDefined();
      const results = index.search('test');
      expect(results).toHaveLength(0);
    });
  });

  describe('searchSkills', () => {
    const searchIndex = createSkillSearchIndex(mockSkills);

    it('should find skills by name', () => {
      const results = searchSkills(searchIndex, 'Kubernetes');
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('kubernetes');
    });

    it('should find skills with partial name match', () => {
      const results = searchSkills(searchIndex, 'kube');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].id).toBe('kubernetes');
    });

    it('should find skills by description', () => {
      const results = searchSkills(searchIndex, 'Amazon Web Services');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].id).toBe('aws');
    });

    it('should find skills by tool name', () => {
      const results = searchSkills(searchIndex, 'pods');
      expect(results.length).toBeGreaterThan(0);
      // Should contain kubernetes since it has "pods" in tool description
      const ids = results.map(r => r.id);
      expect(ids).toContain('kubernetes');
    });

    it('should find skills by tool description', () => {
      const results = searchSkills(searchIndex, 'S3 buckets');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].id).toBe('aws');
    });

    it('should return empty array for empty query', () => {
      const results = searchSkills(searchIndex, '');
      expect(results).toHaveLength(0);
    });

    it('should return empty array for whitespace-only query', () => {
      const results = searchSkills(searchIndex, '   ');
      expect(results).toHaveLength(0);
    });

    it('should return empty array for no matches', () => {
      const results = searchSkills(searchIndex, 'nonexistentskill12345');
      expect(results).toHaveLength(0);
    });

    it('should be case-insensitive', () => {
      const results1 = searchSkills(searchIndex, 'KUBERNETES');
      const results2 = searchSkills(searchIndex, 'kubernetes');
      const results3 = searchSkills(searchIndex, 'KuBeRnEtEs');

      expect(results1).toHaveLength(1);
      expect(results2).toHaveLength(1);
      expect(results3).toHaveLength(1);
      expect(results1[0].id).toBe('kubernetes');
      expect(results2[0].id).toBe('kubernetes');
      expect(results3[0].id).toBe('kubernetes');
    });
  });

  describe('searchSkillsDirect', () => {
    it('should search skills without creating index first', () => {
      const results = searchSkillsDirect(mockSkills, 'GitHub');
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('github');
    });

    it('should return empty array for empty query', () => {
      const results = searchSkillsDirect(mockSkills, '');
      expect(results).toHaveLength(0);
    });
  });

  describe('getSearchSuggestions', () => {
    it('should return skill name suggestions', () => {
      const suggestions = getSearchSuggestions(mockSkills, 'kube');
      expect(suggestions).toContain('Kubernetes');
    });

    it('should limit suggestions to specified count', () => {
      const suggestions = getSearchSuggestions(mockSkills, 'e', 2);
      expect(suggestions.length).toBeLessThanOrEqual(2);
    });

    it('should return empty array for short query (< 2 chars)', () => {
      const suggestions = getSearchSuggestions(mockSkills, 'k');
      expect(suggestions).toHaveLength(0);
    });

    it('should return empty array for empty query', () => {
      const suggestions = getSearchSuggestions(mockSkills, '');
      expect(suggestions).toHaveLength(0);
    });

    it('should default to 5 suggestions when limit not specified', () => {
      const largeMockSkills = Array.from({ length: 10 }, (_, i) => ({
        ...mockSkills[0],
        id: `skill-${i}`,
        name: `Skill ${i}`,
        slug: `skill-${i}`,
      }));

      const suggestions = getSearchSuggestions(largeMockSkills, 'skill');
      expect(suggestions.length).toBeLessThanOrEqual(5);
    });
  });
});
