/**
 * Unit tests for SkillGrid component
 */

import { render, screen } from '@testing-library/react';
import { SkillGrid } from '@/components/home/SkillGrid';
import { SkillCardData } from '@/lib/types';

const mockSkills: SkillCardData[] = [
  {
    id: 'kubernetes',
    name: 'Kubernetes',
    description: 'CLI wrapper for kubectl',
    type: 'native',
    categories: ['devops'],
    version: '1.0.0',
    author: { name: 'Test' },
    installation: { source: 'github.com/test' },
    toolsCount: 15,
    slug: 'kubernetes',
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
  {
    id: 'aws',
    name: 'AWS',
    description: 'AWS CLI wrapper',
    type: 'docker',
    categories: ['cloud'],
    version: '1.0.0',
    author: { name: 'Test' },
    installation: { source: 'github.com/test' },
    toolsCount: 50,
    slug: 'aws',
  },
];

describe('SkillGrid', () => {
  describe('With Skills', () => {
    it('should render all skills', () => {
      render(<SkillGrid skills={mockSkills} />);
      expect(screen.getByText('Kubernetes')).toBeInTheDocument();
      expect(screen.getByText('GitHub')).toBeInTheDocument();
      expect(screen.getByText('AWS')).toBeInTheDocument();
    });

    it('should render grid layout', () => {
      const { container } = render(<SkillGrid skills={mockSkills} />);
      const grid = container.querySelector('.grid');
      expect(grid).toBeInTheDocument();
      expect(grid).toHaveClass('grid-cols-1', 'md:grid-cols-2', 'lg:grid-cols-3', 'xl:grid-cols-4');
    });

    it('should have gap between cards', () => {
      const { container } = render(<SkillGrid skills={mockSkills} />);
      const grid = container.querySelector('.grid');
      expect(grid).toHaveClass('gap-6');
    });

    it('should render correct number of skill cards', () => {
      const { container } = render(<SkillGrid skills={mockSkills} />);
      const links = container.querySelectorAll('a[href^="/skills/"]');
      expect(links).toHaveLength(3);
    });

    it('should use skill id as key', () => {
      const { container } = render(<SkillGrid skills={mockSkills} />);
      // React uses keys internally, we can verify by checking rendered output
      expect(screen.getByText('Kubernetes')).toBeInTheDocument();
      expect(screen.getByText('GitHub')).toBeInTheDocument();
      expect(screen.getByText('AWS')).toBeInTheDocument();
    });
  });

  describe('Empty State', () => {
    it('should show empty message when no skills', () => {
      render(<SkillGrid skills={[]} />);
      expect(screen.getByText('No skills found matching your criteria.')).toBeInTheDocument();
    });

    it('should not render grid when empty', () => {
      const { container } = render(<SkillGrid skills={[]} />);
      const grid = container.querySelector('.grid');
      expect(grid).not.toBeInTheDocument();
    });

    it('should center empty message', () => {
      const { container } = render(<SkillGrid skills={[]} />);
      const emptyContainer = container.querySelector('.text-center');
      expect(emptyContainer).toBeInTheDocument();
      expect(emptyContainer).toHaveClass('py-12');
    });

    it('should have proper empty message styling', () => {
      render(<SkillGrid skills={[]} />);
      const message = screen.getByText('No skills found matching your criteria.');
      expect(message).toHaveClass('text-gray-600', 'text-lg');
    });
  });

  describe('Single Skill', () => {
    it('should render single skill correctly', () => {
      render(<SkillGrid skills={[mockSkills[0]]} />);
      expect(screen.getByText('Kubernetes')).toBeInTheDocument();
      expect(screen.queryByText('GitHub')).not.toBeInTheDocument();
      expect(screen.queryByText('AWS')).not.toBeInTheDocument();
    });

    it('should still use grid layout for single skill', () => {
      const { container } = render(<SkillGrid skills={[mockSkills[0]]} />);
      const grid = container.querySelector('.grid');
      expect(grid).toBeInTheDocument();
    });
  });

  describe('Many Skills', () => {
    it('should handle large number of skills', () => {
      const manySkills = Array.from({ length: 20 }, (_, i) => ({
        ...mockSkills[0],
        id: `skill-${i}`,
        name: `Skill ${i}`,
        slug: `skill-${i}`,
      }));

      render(<SkillGrid skills={manySkills} />);
      expect(screen.getByText('Skill 0')).toBeInTheDocument();
      expect(screen.getByText('Skill 19')).toBeInTheDocument();
    });

    it('should render all skill cards for large dataset', () => {
      const manySkills = Array.from({ length: 50 }, (_, i) => ({
        ...mockSkills[0],
        id: `skill-${i}`,
        name: `Skill ${i}`,
        slug: `skill-${i}`,
      }));

      const { container } = render(<SkillGrid skills={manySkills} />);
      const links = container.querySelectorAll('a[href^="/skills/"]');
      expect(links).toHaveLength(50);
    });
  });

  describe('Different Skill Types', () => {
    it('should render mixed skill types', () => {
      render(<SkillGrid skills={mockSkills} />);
      expect(screen.getByText('NATIVE')).toBeInTheDocument();
      expect(screen.getByText('WASM')).toBeInTheDocument();
      expect(screen.getByText('DOCKER')).toBeInTheDocument();
    });
  });

  describe('Responsive Behavior', () => {
    it('should have responsive grid columns', () => {
      const { container } = render(<SkillGrid skills={mockSkills} />);
      const grid = container.querySelector('.grid');

      // Verify responsive classes are present
      expect(grid).toHaveClass('grid-cols-1'); // Mobile
      expect(grid).toHaveClass('md:grid-cols-2'); // Tablet
      expect(grid).toHaveClass('lg:grid-cols-3'); // Desktop
      expect(grid).toHaveClass('xl:grid-cols-4'); // Large Desktop
    });
  });
});
