/**
 * Unit tests for SkillFilters component
 */

import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SkillFilters } from '@/components/home/SkillFilters';
import { useRouter, useSearchParams } from 'next/navigation';

// Mock next/navigation
jest.mock('next/navigation', () => ({
  useRouter: jest.fn(),
  useSearchParams: jest.fn(),
}));

const mockRouter = {
  push: jest.fn(),
};

const mockSearchParams = {
  getAll: jest.fn(() => []),
  toString: jest.fn(() => ''),
};

describe('SkillFilters', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (useRouter as jest.Mock).mockReturnValue(mockRouter);
    (useSearchParams as jest.Mock).mockReturnValue(mockSearchParams);
  });

  describe('Basic Rendering', () => {
    it('should render filters heading', () => {
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );
      expect(screen.getByText('Filters')).toBeInTheDocument();
    });

    it('should render type filters', () => {
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );
      expect(screen.getByText('NATIVE')).toBeInTheDocument();
      expect(screen.getByText('WASM')).toBeInTheDocument();
      expect(screen.getByText('DOCKER')).toBeInTheDocument();
    });

    it('should render badge filters', () => {
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );
      expect(screen.getByText(/official/i)).toBeInTheDocument();
      expect(screen.getByText(/verified/i)).toBeInTheDocument();
      expect(screen.getByText(/featured/i)).toBeInTheDocument();
      expect(screen.getByText(/community/i)).toBeInTheDocument();
    });

    it('should render category filters when provided', () => {
      render(
        <SkillFilters
          availableCategories={['devops', 'cloud']}
          categoryCounts={{ devops: 5, cloud: 3 }}
        />
      );
      expect(screen.getByText('devops')).toBeInTheDocument();
      expect(screen.getByText('cloud')).toBeInTheDocument();
    });

    it('should display category counts when provided', () => {
      render(
        <SkillFilters
          availableCategories={['devops', 'cloud']}
          categoryCounts={{ devops: 5, cloud: 3 }}
        />
      );
      // Text is split across elements, so check separately
      expect(screen.getByText('devops')).toBeInTheDocument();
      expect(screen.getByText('(5)')).toBeInTheDocument();
      expect(screen.getByText('cloud')).toBeInTheDocument();
      expect(screen.getByText('(3)')).toBeInTheDocument();
    });

    it('should not show clear button when no filters active', () => {
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );
      expect(screen.queryByText('Clear all')).not.toBeInTheDocument();
    });

    it('should not show active filter count when no filters', () => {
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );
      expect(screen.queryByText(/filters active/)).not.toBeInTheDocument();
    });
  });

  describe('Filter Selection', () => {
    it('should toggle type filter on click', async () => {
      const user = userEvent.setup();
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      const nativeButton = screen.getByText('NATIVE');
      await user.click(nativeButton);

      expect(mockRouter.push).toHaveBeenCalledWith('?type=native', {
        scroll: false,
      });
    });

    it('should toggle category filter on click', async () => {
      const user = userEvent.setup();
      render(
        <SkillFilters
          availableCategories={['devops']}
          categoryCounts={{}}
        />
      );

      const devopsButton = screen.getByText('devops');
      await user.click(devopsButton);

      expect(mockRouter.push).toHaveBeenCalledWith('?category=devops', {
        scroll: false,
      });
    });

    it('should toggle badge filter on click', async () => {
      const user = userEvent.setup();
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      const officialButton = screen.getByText(/official/i);
      await user.click(officialButton);

      expect(mockRouter.push).toHaveBeenCalledWith('?badge=official', {
        scroll: false,
      });
    });

    it('should highlight active type filter', () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        return [];
      });

      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      const nativeButton = screen.getByText('NATIVE');
      expect(nativeButton).toHaveClass('bg-blue-600', 'text-white');
    });

    it('should highlight active category filter', () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'category') return ['devops'];
        return [];
      });

      render(
        <SkillFilters
          availableCategories={['devops', 'cloud']}
          categoryCounts={{}}
        />
      );

      const devopsButton = screen.getByText('devops');
      expect(devopsButton).toHaveClass('bg-blue-600', 'text-white');
    });

    it('should highlight multiple active filters', () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native', 'wasm'];
        return [];
      });

      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      expect(screen.getByText('NATIVE')).toHaveClass('bg-blue-600');
      expect(screen.getByText('WASM')).toHaveClass('bg-blue-600');
      expect(screen.getByText('DOCKER')).not.toHaveClass('bg-blue-600');
    });
  });

  describe('URL State Management', () => {
    it('should preserve existing params when adding filter', async () => {
      mockSearchParams.toString.mockReturnValue('type=native');
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        return [];
      });

      const user = userEvent.setup();
      render(
        <SkillFilters
          availableCategories={['devops']}
          categoryCounts={{}}
        />
      );

      const devopsButton = screen.getByText('devops');
      await user.click(devopsButton);

      expect(mockRouter.push).toHaveBeenCalledWith(
        expect.stringContaining('type=native'),
        { scroll: false }
      );
      expect(mockRouter.push).toHaveBeenCalledWith(
        expect.stringContaining('category=devops'),
        { scroll: false }
      );
    });

    it('should remove filter when clicking active filter', async () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        return [];
      });

      const user = userEvent.setup();
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      const nativeButton = screen.getByText('NATIVE');
      await user.click(nativeButton);

      expect(mockRouter.push).toHaveBeenCalledWith(
        expect.not.stringContaining('type=native'),
        { scroll: false }
      );
    });
  });

  describe('Clear Filters', () => {
    it('should show clear button when filters active', () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        return [];
      });

      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      expect(screen.getByText('Clear all')).toBeInTheDocument();
    });

    it('should clear all filters on clear button click', async () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        if (key === 'category') return ['devops'];
        return [];
      });

      const user = userEvent.setup();
      render(
        <SkillFilters
          availableCategories={['devops']}
          categoryCounts={{}}
        />
      );

      const clearButton = screen.getByText('Clear all');
      await user.click(clearButton);

      expect(mockRouter.push).toHaveBeenCalledWith('/', { scroll: false });
    });

    it('should call onFiltersChange when clearing', async () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        return [];
      });

      const onFiltersChange = jest.fn();
      const user = userEvent.setup();
      render(
        <SkillFilters
          availableCategories={[]}
          categoryCounts={{}}
          onFiltersChange={onFiltersChange}
        />
      );

      const clearButton = screen.getByText('Clear all');
      await user.click(clearButton);

      expect(onFiltersChange).toHaveBeenCalledWith({
        types: [],
        categories: [],
        badges: [],
      });
    });
  });

  describe('Active Filter Count', () => {
    it('should display active filter count', () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native', 'wasm'];
        if (key === 'category') return ['devops'];
        return [];
      });

      render(
        <SkillFilters
          availableCategories={['devops']}
          categoryCounts={{}}
        />
      );

      expect(screen.getByText('3 filters active')).toBeInTheDocument();
    });

    it('should use singular form for single filter', () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        return [];
      });

      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      expect(screen.getByText('1 filter active')).toBeInTheDocument();
    });
  });

  describe('Callbacks', () => {
    it('should call onFiltersChange when filter toggled', async () => {
      // Mock URLSearchParams to return the new filter
      const mockURLSearchParams = jest.fn().mockImplementation(() => ({
        getAll: jest.fn((key: string) => {
          if (key === 'type') return ['native'];
          return [];
        }),
        append: jest.fn(),
        delete: jest.fn(),
        toString: jest.fn(() => 'type=native'),
      }));
      (global as any).URLSearchParams = mockURLSearchParams;

      const onFiltersChange = jest.fn();
      const user = userEvent.setup();
      render(
        <SkillFilters
          availableCategories={[]}
          categoryCounts={{}}
          onFiltersChange={onFiltersChange}
        />
      );

      const nativeButton = screen.getByText('NATIVE');
      await user.click(nativeButton);

      expect(onFiltersChange).toHaveBeenCalledWith({
        types: ['native'],
        categories: [],
        badges: [],
      });
    });
  });

  describe('Accessibility', () => {
    it('should have proper aria-pressed attributes', () => {
      mockSearchParams.getAll.mockImplementation((key: string) => {
        if (key === 'type') return ['native'];
        return [];
      });

      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );

      const nativeButton = screen.getByText('NATIVE');
      const wasmButton = screen.getByText('WASM');

      expect(nativeButton).toHaveAttribute('aria-pressed', 'true');
      expect(wasmButton).toHaveAttribute('aria-pressed', 'false');
    });

    it('should have type="button" on all buttons', () => {
      render(
        <SkillFilters
          availableCategories={['devops']}
          categoryCounts={{}}
        />
      );

      const buttons = screen.getAllByRole('button');
      buttons.forEach((button) => {
        expect(button).toHaveAttribute('type', 'button');
      });
    });
  });

  describe('Badge Icons', () => {
    it('should display star icon for official badge', () => {
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );
      const officialButton = screen.getByText(/⭐.*official/i);
      expect(officialButton).toBeInTheDocument();
    });

    it('should display checkmark icon for verified badge', () => {
      render(
        <SkillFilters availableCategories={[]} categoryCounts={{}} />
      );
      const verifiedButton = screen.getByText(/✓.*verified/i);
      expect(verifiedButton).toBeInTheDocument();
    });
  });
});
