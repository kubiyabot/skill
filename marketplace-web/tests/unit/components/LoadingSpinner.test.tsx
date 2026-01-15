/**
 * Unit tests for LoadingSpinner components
 */

import { render, screen } from '@testing-library/react';
import {
  LoadingSpinner,
  LoadingSpinnerFullPage,
  LoadingSpinnerCentered,
} from '@/components/ui/LoadingSpinner';

describe('LoadingSpinner', () => {
  describe('Basic Rendering', () => {
    it('should render with default props', () => {
      render(<LoadingSpinner />);
      const spinner = screen.getByRole('status');
      expect(spinner).toBeInTheDocument();
    });

    it('should have default aria-label', () => {
      render(<LoadingSpinner />);
      const spinner = screen.getByRole('status');
      expect(spinner).toHaveAttribute('aria-label', 'Loading');
    });

    it('should render with custom label', () => {
      render(<LoadingSpinner label="Loading skills..." />);
      expect(screen.getByText('Loading skills...')).toBeInTheDocument();
    });

    it('should have custom aria-label when label provided', () => {
      render(<LoadingSpinner label="Loading data" />);
      const spinner = screen.getByRole('status');
      expect(spinner).toHaveAttribute('aria-label', 'Loading data');
    });

    it('should render spinner icon', () => {
      const { container } = render(<LoadingSpinner />);
      const svg = container.querySelector('svg');
      expect(svg).toBeInTheDocument();
      expect(svg).toHaveClass('animate-spin');
    });

    it('should apply custom className', () => {
      render(<LoadingSpinner className="custom-class" />);
      const spinner = screen.getByRole('status');
      expect(spinner).toHaveClass('custom-class');
    });
  });

  describe('Sizes', () => {
    it('should render medium size by default', () => {
      const { container } = render(<LoadingSpinner />);
      const svg = container.querySelector('svg');
      expect(svg).toHaveAttribute('width', '24');
      expect(svg).toHaveAttribute('height', '24');
    });

    it('should render small size', () => {
      const { container } = render(<LoadingSpinner size="sm" />);
      const svg = container.querySelector('svg');
      expect(svg).toHaveAttribute('width', '16');
      expect(svg).toHaveAttribute('height', '16');
    });

    it('should render large size', () => {
      const { container } = render(<LoadingSpinner size="lg" />);
      const svg = container.querySelector('svg');
      expect(svg).toHaveAttribute('width', '32');
      expect(svg).toHaveAttribute('height', '32');
    });
  });

  describe('With Label', () => {
    it('should not show label text when not provided', () => {
      render(<LoadingSpinner />);
      const spinner = screen.getByRole('status');
      expect(spinner.querySelector('span')).not.toBeInTheDocument();
    });

    it('should show label text when provided', () => {
      render(<LoadingSpinner label="Please wait" />);
      expect(screen.getByText('Please wait')).toBeInTheDocument();
    });

    it('should style label text appropriately', () => {
      render(<LoadingSpinner label="Loading" />);
      const labelElement = screen.getByText('Loading');
      expect(labelElement).toHaveClass('text-sm', 'text-gray-600');
    });
  });

  describe('Accessibility', () => {
    it('should have role="status"', () => {
      render(<LoadingSpinner />);
      expect(screen.getByRole('status')).toBeInTheDocument();
    });

    it('should have aria-label for screen readers', () => {
      render(<LoadingSpinner label="Loading content" />);
      const spinner = screen.getByRole('status');
      expect(spinner).toHaveAttribute('aria-label', 'Loading content');
    });
  });
});

describe('LoadingSpinnerFullPage', () => {
  it('should render with default label', () => {
    render(<LoadingSpinnerFullPage />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('should render with custom label', () => {
    render(<LoadingSpinnerFullPage label="Loading application..." />);
    expect(screen.getByText('Loading application...')).toBeInTheDocument();
  });

  it('should be fixed positioned', () => {
    const { container } = render(<LoadingSpinnerFullPage />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveClass('fixed', 'inset-0');
  });

  it('should have high z-index for overlay', () => {
    const { container } = render(<LoadingSpinnerFullPage />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveClass('z-50');
  });

  it('should center content', () => {
    const { container } = render(<LoadingSpinnerFullPage />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveClass('flex', 'items-center', 'justify-center');
  });

  it('should render large spinner', () => {
    const { container } = render(<LoadingSpinnerFullPage />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '32');
    expect(svg).toHaveAttribute('height', '32');
  });
});

describe('LoadingSpinnerCentered', () => {
  it('should render without label', () => {
    render(<LoadingSpinnerCentered />);
    const spinner = screen.getByRole('status');
    expect(spinner).toBeInTheDocument();
  });

  it('should render with custom label', () => {
    render(<LoadingSpinnerCentered label="Fetching data..." />);
    expect(screen.getByText('Fetching data...')).toBeInTheDocument();
  });

  it('should be centered with padding', () => {
    const { container } = render(<LoadingSpinnerCentered />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveClass('flex', 'items-center', 'justify-center', 'py-12');
  });

  it('should render medium spinner', () => {
    const { container } = render(<LoadingSpinnerCentered />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '24');
    expect(svg).toHaveAttribute('height', '24');
  });
});
