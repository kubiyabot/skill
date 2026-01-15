/**
 * Unit tests for Badge component
 */

import { render, screen } from '@testing-library/react';
import { Badge } from '@/components/ui/Badge';

describe('Badge', () => {
  describe('Basic Rendering', () => {
    it('should render badge with children', () => {
      render(<Badge variant="official">Official</Badge>);
      expect(screen.getByText('Official')).toBeInTheDocument();
    });

    it('should apply custom className', () => {
      render(<Badge variant="official" className="custom-class">Test</Badge>);
      const badge = screen.getByText('Test');
      expect(badge).toHaveClass('custom-class');
    });

    it('should have base badge styling', () => {
      render(<Badge variant="official">Test</Badge>);
      const badge = screen.getByText('Test');
      expect(badge).toHaveClass('inline-flex', 'items-center', 'px-2.5', 'py-0.5', 'rounded-full', 'text-xs', 'font-medium');
    });
  });

  describe('Variants', () => {
    it('should apply official variant styling', () => {
      render(<Badge variant="official">Official</Badge>);
      const badge = screen.getByText('Official');
      expect(badge).toHaveClass('bg-badge-official', 'text-white');
    });

    it('should apply verified variant styling', () => {
      render(<Badge variant="verified">Verified</Badge>);
      const badge = screen.getByText('Verified');
      expect(badge).toHaveClass('bg-badge-verified', 'text-white');
    });

    it('should apply featured variant styling', () => {
      render(<Badge variant="featured">Featured</Badge>);
      const badge = screen.getByText('Featured');
      expect(badge).toHaveClass('bg-badge-featured', 'text-white');
    });

    it('should apply community variant styling', () => {
      render(<Badge variant="community">Community</Badge>);
      const badge = screen.getByText('Community');
      expect(badge).toHaveClass('bg-badge-community', 'text-white');
    });
  });

  describe('Children', () => {
    it('should render text children', () => {
      render(<Badge variant="official">Badge Text</Badge>);
      expect(screen.getByText('Badge Text')).toBeInTheDocument();
    });

    it('should render with emoji', () => {
      render(<Badge variant="official">⭐ Official</Badge>);
      expect(screen.getByText('⭐ Official')).toBeInTheDocument();
    });

    it('should render with mixed content', () => {
      render(<Badge variant="verified">✓ Verified Badge</Badge>);
      expect(screen.getByText('✓ Verified Badge')).toBeInTheDocument();
    });
  });

  describe('Multiple Badges', () => {
    it('should render multiple badges independently', () => {
      const { container } = render(
        <div>
          <Badge variant="official">Official</Badge>
          <Badge variant="verified">Verified</Badge>
          <Badge variant="trending">Trending</Badge>
        </div>
      );

      expect(screen.getByText('Official')).toBeInTheDocument();
      expect(screen.getByText('Verified')).toBeInTheDocument();
      expect(screen.getByText('Trending')).toBeInTheDocument();

      const badges = container.querySelectorAll('.inline-flex');
      expect(badges).toHaveLength(3);
    });
  });

  describe('Styling', () => {
    it('should maintain consistent sizing', () => {
      render(<Badge variant="official">Test</Badge>);
      const badge = screen.getByText('Test');
      expect(badge).toHaveClass('text-xs', 'px-2.5', 'py-0.5');
    });

    it('should be inline element', () => {
      render(<Badge variant="official">Test</Badge>);
      const badge = screen.getByText('Test');
      expect(badge).toHaveClass('inline-flex');
    });

    it('should be rounded', () => {
      render(<Badge variant="official">Test</Badge>);
      const badge = screen.getByText('Test');
      expect(badge).toHaveClass('rounded-full');
    });
  });
});
