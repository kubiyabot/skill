/**
 * Unit tests for Skeleton components
 */

import { render, screen } from '@testing-library/react';
import { Skeleton, SkeletonText } from '@/components/ui/Skeleton';

describe('Skeleton', () => {
  describe('Basic Rendering', () => {
    it('should render with default props', () => {
      render(<Skeleton />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toBeInTheDocument();
      expect(skeleton).toHaveAttribute('aria-label', 'Loading...');
    });

    it('should render with custom className', () => {
      render(<Skeleton className="custom-class" />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveClass('custom-class');
    });

    it('should have animate-pulse class by default', () => {
      render(<Skeleton />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveClass('animate-pulse');
    });

    it('should not animate when animate is false', () => {
      render(<Skeleton animate={false} />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).not.toHaveClass('animate-pulse');
    });
  });

  describe('Variants', () => {
    it('should render text variant by default', () => {
      render(<Skeleton />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveClass('h-4', 'rounded');
    });

    it('should render text variant', () => {
      render(<Skeleton variant="text" />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveClass('h-4', 'rounded');
    });

    it('should render circular variant', () => {
      render(<Skeleton variant="circular" />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveClass('rounded-full');
    });

    it('should render rectangular variant', () => {
      render(<Skeleton variant="rectangular" />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveClass('rounded-lg');
    });
  });

  describe('Dimensions', () => {
    it('should apply width as number', () => {
      render(<Skeleton width={100} />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveStyle({ width: '100px' });
    });

    it('should apply width as string', () => {
      render(<Skeleton width="50%" />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveStyle({ width: '50%' });
    });

    it('should apply height as number', () => {
      render(<Skeleton height={50} />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveStyle({ height: '50px' });
    });

    it('should apply height as string', () => {
      render(<Skeleton height="2rem" />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveStyle({ height: '2rem' });
    });

    it('should apply both width and height', () => {
      render(<Skeleton width={100} height={50} />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveStyle({ width: '100px', height: '50px' });
    });
  });

  describe('Accessibility', () => {
    it('should have role="status"', () => {
      render(<Skeleton />);
      expect(screen.getByRole('status')).toBeInTheDocument();
    });

    it('should have aria-label', () => {
      render(<Skeleton />);
      const skeleton = screen.getByRole('status');
      expect(skeleton).toHaveAttribute('aria-label', 'Loading...');
    });
  });
});

describe('SkeletonText', () => {
  it('should render 3 lines by default', () => {
    render(<SkeletonText />);
    const skeletons = screen.getAllByRole('status');
    expect(skeletons).toHaveLength(3);
  });

  it('should render specified number of lines', () => {
    render(<SkeletonText lines={5} />);
    const skeletons = screen.getAllByRole('status');
    expect(skeletons).toHaveLength(5);
  });

  it('should apply custom className to container', () => {
    const { container } = render(<SkeletonText className="custom-class" />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveClass('custom-class');
    expect(wrapper).toHaveClass('space-y-2');
  });

  it('should render last line with 70% width', () => {
    render(<SkeletonText lines={3} />);
    const skeletons = screen.getAllByRole('status');
    // Last skeleton should have different width
    expect(skeletons[2]).toHaveStyle({ width: '70%' });
  });

  it('should render other lines with 100% width', () => {
    render(<SkeletonText lines={3} />);
    const skeletons = screen.getAllByRole('status');
    expect(skeletons[0]).toHaveStyle({ width: '100%' });
    expect(skeletons[1]).toHaveStyle({ width: '100%' });
  });
});
