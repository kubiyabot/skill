/**
 * Unit tests for HomePageClient component
 */

import { render, screen } from '@testing-library/react';
import { HomePageClient } from '@/components/home/HomePageClient';
import { SkillCardData } from '@/lib/types';
import { useSearchParams, useRouter } from 'next/navigation';

// Mock next/navigation
jest.mock('next/navigation', () => ({
  useSearchParams: jest.fn(),
  useRouter: jest.fn(),
}));

const mockRouter = {
  push: jest.fn(),
};

const mockSearchParams = {
  get: jest.fn(() => null),
  getAll: jest.fn(() => []),
};

(useRouter as jest.Mock).mockReturnValue(mockRouter);
(useSearchParams as jest.Mock).mockReturnValue(mockSearchParams);

const mockSkills: SkillCardData[] = [
  {
    id: 'kubernetes',
    name: 'Kubernetes',
    description: 'CLI wrapper for kubectl commands',
    type: 'native',
    categories: ['devops', 'cloud'],
    version: '1.0.0',
    author: { name: 'Test' },
    installation: { source: 'github.com/test' },
    toolsCount: 15,
    slug: 'kubernetes',
    icon: '/icons/kubernetes.svg',
  },
  {
    id: 'github',
    name: 'GitHub',
    description: 'GitHub CLI wrapper',
    type: 'wasm',
    categories: ['devops'],
    version: '1.0.0',
    author: { name: 'Test' },
    installation: { source: 'github.com/test' },
    toolsCount: 20,
    slug: 'github',
  },
];

describe('HomePageClient', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockSearchParams.get.mockReturnValue(null);
    mockSearchParams.getAll.mockReturnValue([]);
  });

  it('should render search component', () => {
    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops', 'cloud']}
        categoryCounts={{ devops: 2, cloud: 1 }}
      />
    );

    expect(screen.getByPlaceholderText(/Search skills/)).toBeInTheDocument();
  });

  it('should render filter controls', () => {
    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops', 'cloud']}
        categoryCounts={{ devops: 2, cloud: 1 }}
      />
    );

    expect(screen.getByText('Filters')).toBeInTheDocument();
  });

  it('should display "All Skills" when no filters active', () => {
    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    expect(screen.getByText('All Skills')).toBeInTheDocument();
  });

  it('should display skill count', () => {
    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    expect(screen.getByText('2 skills')).toBeInTheDocument();
  });

  it('should display "Filtered Skills" when search query is active', () => {
    mockSearchParams.get.mockImplementation((key: string) => {
      if (key === 'q') return 'kubernetes';
      return null;
    });

    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    expect(screen.getByText('Filtered Skills')).toBeInTheDocument();
  });

  it('should display "Filtered Skills" when type filter is active', () => {
    mockSearchParams.getAll.mockImplementation((key: string) => {
      if (key === 'type') return ['native'];
      return [];
    });

    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    expect(screen.getByText('Filtered Skills')).toBeInTheDocument();
  });

  it('should filter skills by search query', () => {
    mockSearchParams.get.mockImplementation((key: string) => {
      if (key === 'q') return 'kubernetes';
      return null;
    });

    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    // Should show 1 skill that matches "kubernetes"
    expect(screen.getByText('1 skill')).toBeInTheDocument();
  });

  it('should filter skills by type', () => {
    mockSearchParams.getAll.mockImplementation((key: string) => {
      if (key === 'type') return ['native'];
      return [];
    });

    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    // Should show 1 native skill
    expect(screen.getByText('1 skill')).toBeInTheDocument();
  });

  it('should combine search and filters', () => {
    mockSearchParams.get.mockImplementation((key: string) => {
      if (key === 'q') return 'github';
      return null;
    });
    mockSearchParams.getAll.mockImplementation((key: string) => {
      if (key === 'type') return ['wasm'];
      return [];
    });

    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    // Should show 1 skill that matches both search and type filter
    expect(screen.getByText('1 skill')).toBeInTheDocument();
  });

  it('should show singular "skill" for single result', () => {
    mockSearchParams.get.mockImplementation((key: string) => {
      if (key === 'q') return 'kubernetes';
      return null;
    });

    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    expect(screen.getByText('1 skill')).toBeInTheDocument();
  });

  it('should show plural "skills" for multiple results', () => {
    render(
      <HomePageClient
        skills={mockSkills}
        categories={['devops']}
        categoryCounts={{}}
      />
    );

    expect(screen.getByText('2 skills')).toBeInTheDocument();
  });
});
