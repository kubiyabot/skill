/**
 * Unit tests for SkillCard component
 */

import { render, screen } from '@testing-library/react';
import { SkillCard } from '@/components/home/SkillCard';
import { SkillCardData } from '@/lib/types';

const mockSkill: SkillCardData = {
  id: 'kubernetes',
  name: 'Kubernetes',
  description: 'CLI wrapper for kubectl with simplified commands',
  type: 'native',
  categories: ['devops', 'cloud'],
  version: '1.0.0',
  author: { name: 'Test Author' },
  installation: { source: 'github.com/test/kubernetes' },
  toolsCount: 15,
  slug: 'kubernetes',
  icon: '/icons/kubernetes.svg',
};

describe('SkillCard', () => {
  describe('Basic Rendering', () => {
    it('should render skill name', () => {
      render(<SkillCard skill={mockSkill} />);
      expect(screen.getByText('Kubernetes')).toBeInTheDocument();
    });

    it('should render skill description', () => {
      render(<SkillCard skill={mockSkill} />);
      expect(screen.getByText('CLI wrapper for kubectl with simplified commands')).toBeInTheDocument();
    });

    it('should render tools count', () => {
      render(<SkillCard skill={mockSkill} />);
      expect(screen.getByText('15 tools')).toBeInTheDocument();
    });

    it('should render author name', () => {
      render(<SkillCard skill={mockSkill} />);
      expect(screen.getByText('by Test Author')).toBeInTheDocument();
    });

    it('should render skill without icon', () => {
      const skillWithoutIcon = { ...mockSkill, icon: undefined };
      render(<SkillCard skill={skillWithoutIcon} />);
      expect(screen.queryByAltText('Kubernetes icon')).not.toBeInTheDocument();
    });
  });

  describe('Icon', () => {
    it('should render icon when provided', () => {
      render(<SkillCard skill={mockSkill} />);
      const icon = screen.getByAltText('Kubernetes icon');
      expect(icon).toBeInTheDocument();
      expect(icon).toHaveAttribute('src', '/icons/kubernetes.svg');
    });

    it('should have proper icon container styling', () => {
      const { container } = render(<SkillCard skill={mockSkill} />);
      const iconContainer = container.querySelector('.w-12.h-12');
      expect(iconContainer).toBeInTheDocument();
      expect(iconContainer).toHaveClass('rounded-lg', 'bg-gray-50', 'border');
    });
  });

  describe('Type Badge', () => {
    it('should display type badge in uppercase', () => {
      render(<SkillCard skill={mockSkill} />);
      expect(screen.getByText('NATIVE')).toBeInTheDocument();
    });

    it('should display WASM type correctly', () => {
      const wasmSkill = { ...mockSkill, type: 'wasm' as const };
      render(<SkillCard skill={wasmSkill} />);
      expect(screen.getByText('WASM')).toBeInTheDocument();
    });

    it('should display Docker type correctly', () => {
      const dockerSkill = { ...mockSkill, type: 'docker' as const };
      render(<SkillCard skill={dockerSkill} />);
      expect(screen.getByText('DOCKER')).toBeInTheDocument();
    });
  });

  describe('Official Badge', () => {
    it('should show official badge when skill has official badge', () => {
      const officialSkill = { ...mockSkill, badges: ['official'] };
      render(<SkillCard skill={officialSkill} />);
      expect(screen.getByText('⭐ Official')).toBeInTheDocument();
    });

    it('should not show official badge when skill does not have it', () => {
      render(<SkillCard skill={mockSkill} />);
      expect(screen.queryByText('⭐ Official')).not.toBeInTheDocument();
    });

    it('should show official badge with other badges', () => {
      const skillWithBadges = { ...mockSkill, badges: ['official', 'verified'] };
      render(<SkillCard skill={skillWithBadges} />);
      expect(screen.getByText('⭐ Official')).toBeInTheDocument();
    });

    it('should have correct official badge styling', () => {
      const officialSkill = { ...mockSkill, badges: ['official'] };
      render(<SkillCard skill={officialSkill} />);
      const badge = screen.getByText('⭐ Official');
      expect(badge).toHaveClass('bg-blue-100', 'text-blue-800');
    });
  });

  describe('Link', () => {
    it('should link to skill detail page', () => {
      render(<SkillCard skill={mockSkill} />);
      const link = screen.getByRole('link');
      expect(link).toHaveAttribute('href', '/skills/kubernetes');
    });

    it('should use slug for link URL', () => {
      const skillWithDifferentSlug = { ...mockSkill, slug: 'k8s-cli' };
      render(<SkillCard skill={skillWithDifferentSlug} />);
      const link = screen.getByRole('link');
      expect(link).toHaveAttribute('href', '/skills/k8s-cli');
    });

    it('should make entire card clickable', () => {
      const { container } = render(<SkillCard skill={mockSkill} />);
      const link = screen.getByRole('link');
      const card = link.querySelector('.card');
      expect(card).toHaveClass('cursor-pointer');
    });
  });

  describe('Layout', () => {
    it('should have flex column layout', () => {
      const { container } = render(<SkillCard skill={mockSkill} />);
      const card = container.querySelector('.card');
      expect(card).toHaveClass('flex', 'flex-col');
    });

    it('should have hover effect class', () => {
      const { container } = render(<SkillCard skill={mockSkill} />);
      const card = container.querySelector('.card');
      expect(card).toHaveClass('card-hover');
    });

    it('should have footer with border', () => {
      const { container } = render(<SkillCard skill={mockSkill} />);
      const footer = container.querySelector('.border-t');
      expect(footer).toBeInTheDocument();
      expect(footer).toHaveClass('border-gray-100');
    });

    it('should clamp description to 2 lines', () => {
      const { container } = render(<SkillCard skill={mockSkill} />);
      const description = container.querySelector('.line-clamp-2');
      expect(description).toBeInTheDocument();
    });
  });

  describe('Multiple Skills', () => {
    it('should render different skills correctly', () => {
      const skill1 = mockSkill;
      const skill2: SkillCardData = {
        ...mockSkill,
        id: 'github',
        name: 'GitHub',
        description: 'GitHub CLI wrapper',
        slug: 'github',
        toolsCount: 25,
        author: { name: 'GitHub Team' },
      };

      const { rerender } = render(<SkillCard skill={skill1} />);
      expect(screen.getByText('Kubernetes')).toBeInTheDocument();
      expect(screen.getByText('15 tools')).toBeInTheDocument();

      rerender(<SkillCard skill={skill2} />);
      expect(screen.getByText('GitHub')).toBeInTheDocument();
      expect(screen.getByText('25 tools')).toBeInTheDocument();
      expect(screen.getByText('by GitHub Team')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have descriptive alt text for icon', () => {
      render(<SkillCard skill={mockSkill} />);
      const icon = screen.getByAltText('Kubernetes icon');
      expect(icon).toBeInTheDocument();
    });

    it('should be keyboard accessible via link', () => {
      render(<SkillCard skill={mockSkill} />);
      const link = screen.getByRole('link');
      expect(link).toBeInTheDocument();
    });
  });
});
