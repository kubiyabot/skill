/**
 * Unit tests for data loading functions
 */

import {
  loadAllSkills,
  loadSkillById,
  getAllSkillIds,
  loadCategories,
  getSkillCountsByCategory,
  getRelatedSkills,
  getMarketplaceStats,
} from '@/lib/data/loadSkills';
import fs from 'fs';
import path from 'path';

jest.mock('fs');
jest.mock('path');

const mockFs = fs as jest.Mocked<typeof fs>;
const mockPath = path as jest.Mocked<typeof path>;

const mockSkillManifest = {
  id: 'test-skill',
  name: 'Test Skill',
  description: 'A test skill for unit testing',
  type: 'native' as const,
  version: '1.0.0',
  author: { name: 'Test Author' },
  categories: ['devops', 'cloud'],
  installation: { source: 'github.com/test/skill' },
  tools: [
    { name: 'test-tool', description: 'A test tool' },
    { name: 'another-tool', description: 'Another test tool' },
  ],
};

const mockSkillManifest2 = {
  ...mockSkillManifest,
  id: 'kubernetes',
  name: 'Kubernetes',
  categories: ['devops'],
  badges: ['official', 'featured'],
};

const mockSkillManifest3 = {
  ...mockSkillManifest,
  id: 'github',
  name: 'GitHub',
  categories: ['devops'],
  badges: ['official'],
};

const mockCategoriesData = {
  categories: [
    { id: 'devops', name: 'DevOps', description: 'DevOps tools' },
    { id: 'cloud', name: 'Cloud', description: 'Cloud services' },
  ],
};

describe('loadAllSkills', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockPath.join.mockImplementation((...args) => args.join('/'));
  });

  it('should return empty array when skills directory does not exist', async () => {
    mockFs.existsSync.mockReturnValue(false);

    const skills = await loadAllSkills();

    expect(skills).toEqual([]);
    expect(mockFs.existsSync).toHaveBeenCalled();
  });

  it('should load and enrich skills from JSON files', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue(['test-skill.json'] as any);
    mockFs.readFileSync.mockReturnValue(JSON.stringify(mockSkillManifest));

    const skills = await loadAllSkills();

    expect(skills).toHaveLength(1);
    expect(skills[0].id).toBe('test-skill');
    expect(skills[0].name).toBe('Test Skill');
    expect(skills[0].toolsCount).toBe(2);
    expect(skills[0].slug).toBe('test-skill');
  });

  it('should filter out non-JSON files', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'readme.md',
      'image.png',
    ] as any);
    mockFs.readFileSync.mockReturnValue(JSON.stringify(mockSkillManifest));

    const skills = await loadAllSkills();

    expect(skills).toHaveLength(1);
    expect(mockFs.readFileSync).toHaveBeenCalledTimes(1);
  });

  it('should sort featured skills first', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'kubernetes.json',
    ] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest2));

    const skills = await loadAllSkills();

    expect(skills).toHaveLength(2);
    expect(skills[0].id).toBe('kubernetes'); // Featured first
    expect(skills[1].id).toBe('test-skill');
  });

  it('should sort official skills after featured', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'kubernetes.json',
      'github.json',
    ] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest2))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest3));

    const skills = await loadAllSkills();

    expect(skills).toHaveLength(3);
    expect(skills[0].id).toBe('kubernetes'); // Featured + Official
    expect(skills[1].id).toBe('github'); // Official only
    expect(skills[2].id).toBe('test-skill'); // Neither
  });

  it('should sort alphabetically within same badge tier', async () => {
    const skillZ = { ...mockSkillManifest, id: 'zebra', name: 'Zebra' };
    const skillA = { ...mockSkillManifest, id: 'apple', name: 'Apple' };

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue(['zebra.json', 'apple.json'] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(skillZ))
      .mockReturnValueOnce(JSON.stringify(skillA));

    const skills = await loadAllSkills();

    expect(skills[0].name).toBe('Apple');
    expect(skills[1].name).toBe('Zebra');
  });

  it('should handle skills without tools', async () => {
    const skillNoTools = { ...mockSkillManifest, tools: undefined };

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue(['skill.json'] as any);
    mockFs.readFileSync.mockReturnValue(JSON.stringify(skillNoTools));

    const skills = await loadAllSkills();

    expect(skills[0].toolsCount).toBe(0);
  });

  it('should handle empty skills directory', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([] as any);

    const skills = await loadAllSkills();

    expect(skills).toEqual([]);
  });
});

describe('loadSkillById', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockPath.join.mockImplementation((...args) => args.join('/'));
  });

  it('should return null when skill file does not exist', async () => {
    mockFs.existsSync.mockReturnValue(false);

    const skill = await loadSkillById('non-existent');

    expect(skill).toBeNull();
  });

  it('should load skill manifest by ID', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readFileSync.mockReturnValue(JSON.stringify(mockSkillManifest));

    const skill = await loadSkillById('test-skill');

    expect(skill).not.toBeNull();
    expect(skill?.id).toBe('test-skill');
    expect(skill?.name).toBe('Test Skill');
  });

  it('should load SKILL.md content when skillMdPath is provided', async () => {
    const skillWithMd = {
      ...mockSkillManifest,
      skillMdPath: 'skills/test-skill/SKILL.md',
    };
    const mockMarkdown = '# Test Skill\n\nThis is a test skill.';

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(skillWithMd))
      .mockReturnValueOnce(mockMarkdown);

    const skill = await loadSkillById('test-skill');

    expect(skill?.skillMdContent).toBe(mockMarkdown);
  });

  it('should remove frontmatter from SKILL.md content', async () => {
    const skillWithMd = {
      ...mockSkillManifest,
      skillMdPath: 'skills/test-skill/SKILL.md',
    };
    const mockMarkdownWithFrontmatter =
      '---\ntitle: Test\nauthor: Test\n---\n# Test Skill';

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(skillWithMd))
      .mockReturnValueOnce(mockMarkdownWithFrontmatter);

    const skill = await loadSkillById('test-skill');

    expect(skill?.skillMdContent).toBe('# Test Skill');
    expect(skill?.skillMdContent).not.toContain('---');
  });

  it('should handle missing SKILL.md file gracefully', async () => {
    const skillWithMd = {
      ...mockSkillManifest,
      skillMdPath: 'skills/test-skill/SKILL.md',
    };

    mockFs.existsSync
      .mockReturnValueOnce(true) // Skill file exists
      .mockReturnValueOnce(false); // SKILL.md does not exist
    mockFs.readFileSync.mockReturnValueOnce(JSON.stringify(skillWithMd));

    const skill = await loadSkillById('test-skill');

    expect(skill).not.toBeNull();
    expect(skill?.skillMdContent).toBeUndefined();
  });

  it('should handle SKILL.md read errors', async () => {
    const skillWithMd = {
      ...mockSkillManifest,
      skillMdPath: 'skills/test-skill/SKILL.md',
    };

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(skillWithMd))
      .mockImplementationOnce(() => {
        throw new Error('Read error');
      });

    const skill = await loadSkillById('test-skill');

    expect(skill).not.toBeNull();
    expect(skill?.skillMdContent).toBeUndefined();
  });
});

describe('getAllSkillIds', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockPath.join.mockImplementation((...args) => args.join('/'));
    mockPath.basename.mockImplementation((file, ext) =>
      file.replace(ext || '', '')
    );
  });

  it('should return empty array when skills directory does not exist', async () => {
    mockFs.existsSync.mockReturnValue(false);

    const ids = await getAllSkillIds();

    expect(ids).toEqual([]);
  });

  it('should return all skill IDs', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'kubernetes.json',
      'github.json',
      'aws.json',
    ] as any);

    const ids = await getAllSkillIds();

    expect(ids).toHaveLength(3);
    expect(ids).toContain('kubernetes');
    expect(ids).toContain('github');
    expect(ids).toContain('aws');
  });

  it('should filter out non-JSON files', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'kubernetes.json',
      'readme.md',
      'image.png',
    ] as any);

    const ids = await getAllSkillIds();

    expect(ids).toHaveLength(1);
    expect(ids).toContain('kubernetes');
  });
});

describe('loadCategories', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockPath.join.mockImplementation((...args) => args.join('/'));
  });

  it('should return empty categories when file does not exist', async () => {
    mockFs.existsSync.mockReturnValue(false);

    const categories = await loadCategories();

    expect(categories).toEqual({ categories: [] });
  });

  it('should load categories from JSON file', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readFileSync.mockReturnValue(JSON.stringify(mockCategoriesData));

    const categories = await loadCategories();

    expect(categories.categories).toHaveLength(2);
    expect(categories.categories[0].id).toBe('devops');
    expect(categories.categories[1].id).toBe('cloud');
  });
});

describe('getSkillCountsByCategory', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockPath.join.mockImplementation((...args) => args.join('/'));
  });

  it('should count skills by category', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'kubernetes.json',
    ] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest2));

    const counts = await getSkillCountsByCategory();

    expect(counts.devops).toBe(2); // Both skills have devops
    expect(counts.cloud).toBe(1); // Only test-skill has cloud
  });

  it('should return empty object when no skills', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([] as any);

    const counts = await getSkillCountsByCategory();

    expect(counts).toEqual({});
  });

  it('should handle skills with multiple categories', async () => {
    const multiCatSkill = {
      ...mockSkillManifest,
      categories: ['devops', 'cloud', 'api'],
    };

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue(['skill.json'] as any);
    mockFs.readFileSync.mockReturnValue(JSON.stringify(multiCatSkill));

    const counts = await getSkillCountsByCategory();

    expect(counts.devops).toBe(1);
    expect(counts.cloud).toBe(1);
    expect(counts.api).toBe(1);
  });
});

describe('getRelatedSkills', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockPath.join.mockImplementation((...args) => args.join('/'));
  });

  it('should return empty array when skill not found', async () => {
    mockFs.existsSync.mockReturnValue(false);

    const related = await getRelatedSkills('non-existent');

    expect(related).toEqual([]);
  });

  it('should return skills with shared categories', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'kubernetes.json',
      'github.json',
    ] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest)) // For loadSkillById
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest)) // For loadAllSkills
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest2))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest3));

    const related = await getRelatedSkills('test-skill');

    expect(related.length).toBeGreaterThan(0);
    expect(related.some((s) => s.id === 'test-skill')).toBe(false); // Should exclude current skill
    expect(related.every((s) => s.categories.includes('devops'))).toBe(true); // All share devops category
  });

  it('should respect limit parameter', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'kubernetes.json',
      'github.json',
    ] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest2))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest3));

    const related = await getRelatedSkills('test-skill', 1);

    expect(related).toHaveLength(1);
  });

  it('should exclude current skill from results', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue(['test-skill.json'] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest));

    const related = await getRelatedSkills('test-skill');

    expect(related).toEqual([]);
  });

  it('should default to limit of 4', async () => {
    const manySkills = Array.from({ length: 10 }, (_, i) => ({
      ...mockSkillManifest,
      id: `skill-${i}`,
      name: `Skill ${i}`,
    }));

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue(
      manySkills.map((s) => `${s.id}.json`) as any
    );
    mockFs.readFileSync.mockImplementation((path: any) => {
      const filename = path.split('/').pop();
      const id = filename.replace('.json', '');
      const skill = manySkills.find((s) => s.id === id) || mockSkillManifest;
      return JSON.stringify(skill);
    });

    const related = await getRelatedSkills('skill-0');

    expect(related.length).toBeLessThanOrEqual(4);
  });
});

describe('getMarketplaceStats', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockPath.join.mockImplementation((...args) => args.join('/'));
  });

  it('should calculate marketplace statistics', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'kubernetes.json',
    ] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest2))
      .mockReturnValueOnce(JSON.stringify(mockCategoriesData));

    const stats = await getMarketplaceStats();

    expect(stats.totalSkills).toBe(2);
    expect(stats.totalCategories).toBe(2);
    expect(stats.totalTools).toBe(4); // 2 tools per skill
    expect(stats.byType.native).toBe(2);
    expect(stats.featuredCount).toBe(1); // kubernetes is featured
    expect(stats.officialCount).toBe(1); // kubernetes is official
  });

  it('should handle empty marketplace', async () => {
    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([] as any);
    mockFs.readFileSync.mockReturnValue(
      JSON.stringify({ categories: [] })
    );

    const stats = await getMarketplaceStats();

    expect(stats.totalSkills).toBe(0);
    expect(stats.totalCategories).toBe(0);
    expect(stats.totalTools).toBe(0);
    expect(stats.featuredCount).toBe(0);
    expect(stats.officialCount).toBe(0);
  });

  it('should count different skill types', async () => {
    const wasmSkill = { ...mockSkillManifest, id: 'wasm-skill', type: 'wasm' as const };
    const dockerSkill = { ...mockSkillManifest, id: 'docker-skill', type: 'docker' as const };

    mockFs.existsSync.mockReturnValue(true);
    mockFs.readdirSync.mockReturnValue([
      'test-skill.json',
      'wasm-skill.json',
      'docker-skill.json',
    ] as any);
    mockFs.readFileSync
      .mockReturnValueOnce(JSON.stringify(mockSkillManifest))
      .mockReturnValueOnce(JSON.stringify(wasmSkill))
      .mockReturnValueOnce(JSON.stringify(dockerSkill))
      .mockReturnValueOnce(JSON.stringify(mockCategoriesData));

    const stats = await getMarketplaceStats();

    expect(stats.byType.native).toBe(1);
    expect(stats.byType.wasm).toBe(1);
    expect(stats.byType.docker).toBe(1);
  });
});
