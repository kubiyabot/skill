/**
 * Unit tests for SkillSearch component
 */

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SkillSearch } from '@/components/home/SkillSearch';
import { SkillCardData } from '@/lib/types';
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
  get: jest.fn(() => null),
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
    description: 'GitHub CLI wrapper for repository management',
    type: 'wasm',
    categories: ['devops'],
    version: '1.0.0',
    author: { name: 'Test' },
    installation: { source: 'github.com/test' },
    toolsCount: 20,
    slug: 'github',
    icon: '/icons/github.svg',
  },
  {
    id: 'aws',
    name: 'AWS',
    description: 'AWS CLI wrapper for cloud services',
    type: 'docker',
    categories: ['cloud'],
    version: '1.0.0',
    author: { name: 'Test' },
    installation: { source: 'github.com/test' },
    toolsCount: 50,
    slug: 'aws',
  },
];

describe('SkillSearch', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Basic Rendering', () => {
    it('should render search input', () => {
      render(<SkillSearch skills={mockSkills} />);
      expect(screen.getByPlaceholderText(/Search skills/)).toBeInTheDocument();
    });

    it('should render with custom placeholder', () => {
      render(
        <SkillSearch skills={mockSkills} placeholder="Find a skill..." />
      );
      expect(screen.getByPlaceholderText('Find a skill...')).toBeInTheDocument();
    });

    it('should have search icon', () => {
      const { container } = render(<SkillSearch skills={mockSkills} />);
      const searchIcon = container.querySelector('svg');
      expect(searchIcon).toBeInTheDocument();
    });

    it('should not show results dropdown initially', () => {
      render(<SkillSearch skills={mockSkills} />);
      expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    });

    it('should not show clear button initially', () => {
      render(<SkillSearch skills={mockSkills} />);
      expect(screen.queryByLabelText('Clear search')).not.toBeInTheDocument();
    });
  });

  describe('Search Functionality', () => {
    it('should show results when typing', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
        expect(screen.getByText('Kubernetes')).toBeInTheDocument();
      });
    });

    it('should filter results based on query', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kube');

      await waitFor(() => {
        expect(screen.getByText('Kubernetes')).toBeInTheDocument();
        expect(screen.queryByText('GitHub')).not.toBeInTheDocument();
      });
    });

    it('should show no results message when no matches', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'nonexistent');

      await waitFor(() => {
        expect(screen.getByText(/No skills found for/)).toBeInTheDocument();
      });
    });

    it('should show clear button when typing', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'test');

      expect(screen.getByLabelText('Clear search')).toBeInTheDocument();
    });

    it('should clear input when clear button clicked', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(
        /Search skills/
      ) as HTMLInputElement;

      await user.type(input, 'kubernetes');
      expect(input.value).toBe('kubernetes');

      const clearButton = screen.getByLabelText('Clear search');
      await user.click(clearButton);

      expect(input.value).toBe('');
    });

    it('should respect maxResults prop', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} maxResults={1} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'github');

      await waitFor(() => {
        const results = screen.queryAllByRole('option');
        expect(results.length).toBeLessThanOrEqual(1);
      });
    });
  });

  describe('Results Display', () => {
    it('should display skill name and description', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByText('Kubernetes')).toBeInTheDocument();
        expect(
          screen.getByText('CLI wrapper for kubectl commands')
        ).toBeInTheDocument();
      });
    });

    it('should display type badge', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByText('NATIVE')).toBeInTheDocument();
      });
    });

    it('should display tools count', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByText('15 tools')).toBeInTheDocument();
      });
    });

    it('should display skill icon when available', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        const icon = screen.getByAltText('');
        expect(icon).toHaveAttribute('src', '/icons/kubernetes.svg');
      });
    });
  });

  describe('User Interactions', () => {
    it('should navigate to skill page on result click', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByText('Kubernetes')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Kubernetes'));

      expect(mockRouter.push).toHaveBeenCalledWith('/skills/kubernetes');
    });

    it('should call onResultClick when provided', async () => {
      const onResultClick = jest.fn();
      const user = userEvent.setup();
      render(
        <SkillSearch skills={mockSkills} onResultClick={onResultClick} />
      );
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByText('Kubernetes')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Kubernetes'));

      expect(onResultClick).toHaveBeenCalledWith(mockSkills[0]);
      expect(mockRouter.push).not.toHaveBeenCalled();
    });

    it('should close dropdown after result click', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');
      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Kubernetes'));

      await waitFor(() => {
        expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
      });
    });
  });

  describe('Keyboard Navigation', () => {
    it('should navigate down with arrow down key', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'github');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      const firstResult = screen.getAllByRole('option')[0];
      expect(firstResult).toHaveClass('bg-gray-50');

      fireEvent.keyDown(input, { key: 'ArrowDown' });

      // After arrow down, selection might move if multiple results
      const results = screen.getAllByRole('option');
      if (results.length > 1) {
        expect(results[1]).toHaveClass('bg-gray-50');
      }
    });

    it('should navigate up with arrow up key', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'cloud');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      fireEvent.keyDown(input, { key: 'ArrowDown' });
      fireEvent.keyDown(input, { key: 'ArrowUp' });

      const firstResult = screen.getAllByRole('option')[0];
      expect(firstResult).toHaveClass('bg-gray-50');
    });

    it('should select result with Enter key', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      fireEvent.keyDown(input, { key: 'Enter' });

      expect(mockRouter.push).toHaveBeenCalledWith('/skills/kubernetes');
    });

    it('should close dropdown with Escape key', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      fireEvent.keyDown(input, { key: 'Escape' });

      await waitFor(() => {
        expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
      });
    });

    it('should highlight result on mouse enter', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'cloud');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      const results = screen.getAllByRole('option');
      if (results.length > 1) {
        fireEvent.mouseEnter(results[1]);
        expect(results[1]).toHaveClass('bg-gray-50');
      }
    });
  });

  describe('Click Outside', () => {
    it('should close dropdown when clicking outside', async () => {
      const user = userEvent.setup();
      render(
        <div>
          <SkillSearch skills={mockSkills} />
          <div data-testid="outside">Outside</div>
        </div>
      );
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      const outside = screen.getByTestId('outside');
      fireEvent.mouseDown(outside);

      await waitFor(() => {
        expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
      });
    });

    it('should not close dropdown when clicking inside', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      fireEvent.mouseDown(input);

      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA attributes', () => {
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      expect(input).toHaveAttribute('aria-label', 'Search skills');
      expect(input).toHaveAttribute('aria-autocomplete', 'list');
      expect(input).toHaveAttribute('aria-controls', 'search-results');
    });

    it('should set aria-expanded when dropdown is open', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(input).toHaveAttribute('aria-expanded', 'true');
      });
    });

    it('should have role="listbox" on results', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });
    });

    it('should have role="option" on result items', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'kubernetes');

      await waitFor(() => {
        expect(screen.getByRole('option')).toBeInTheDocument();
      });
    });

    it('should set aria-selected on selected result', async () => {
      const user = userEvent.setup();
      render(<SkillSearch skills={mockSkills} />);
      const input = screen.getByPlaceholderText(/Search skills/);

      await user.type(input, 'cloud');

      await waitFor(() => {
        const results = screen.getAllByRole('option');
        expect(results[0]).toHaveAttribute('aria-selected', 'true');
      });
    });
  });
});
