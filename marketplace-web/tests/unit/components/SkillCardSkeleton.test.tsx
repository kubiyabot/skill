/**
 * Unit tests for SkillCardSkeleton component
 */

import { render, screen } from '@testing-library/react';
import { SkillCardSkeleton, SkillCardSkeletonGrid } from '@/components/home/SkillCardSkeleton';

describe('SkillCardSkeleton', () => {
  it('should render skeleton structure', () => {
    render(<SkillCardSkeleton />);
    // Should have multiple skeleton elements
    const skeletons = screen.getAllByRole('status');
    expect(skeletons.length).toBeGreaterThan(3); // Icon, title, type, description lines, footer
  });

  it('should have card styling', () => {
    const { container } = render(<SkillCardSkeleton />);
    const card = container.firstChild as HTMLElement;
    expect(card).toHaveClass('card', 'h-full', 'flex', 'flex-col');
  });

  it('should render icon skeleton', () => {
    render(<SkillCardSkeleton />);
    const skeletons = screen.getAllByRole('status');
    // First skeleton should be the icon (48x48)
    expect(skeletons[0]).toHaveStyle({ width: '48px', height: '48px' });
  });

  it('should have footer with border', () => {
    const { container } = render(<SkillCardSkeleton />);
    const footer = container.querySelector('.border-t');
    expect(footer).toBeInTheDocument();
    expect(footer).toHaveClass('border-gray-100');
  });

  it('should match SkillCard layout', () => {
    const { container } = render(<SkillCardSkeleton />);

    // Should have header section
    const header = container.querySelector('.flex.items-start.gap-3');
    expect(header).toBeInTheDocument();

    // Should have description area
    const description = container.querySelector('.space-y-2');
    expect(description).toBeInTheDocument();

    // Should have footer
    const footer = container.querySelector('.border-t');
    expect(footer).toBeInTheDocument();
  });
});

describe('SkillCardSkeletonGrid', () => {
  it('should render 6 cards by default', () => {
    const { container } = render(<SkillCardSkeletonGrid />);
    const cards = container.querySelectorAll('.card');
    expect(cards).toHaveLength(6);
  });

  it('should render specified number of cards', () => {
    const { container } = render(<SkillCardSkeletonGrid count={3} />);
    const cards = container.querySelectorAll('.card');
    expect(cards).toHaveLength(3);
  });

  it('should render 12 cards when specified', () => {
    const { container } = render(<SkillCardSkeletonGrid count={12} />);
    const cards = container.querySelectorAll('.card');
    expect(cards).toHaveLength(12);
  });

  it('should render no cards when count is 0', () => {
    const { container } = render(<SkillCardSkeletonGrid count={0} />);
    const cards = container.querySelectorAll('.card');
    expect(cards).toHaveLength(0);
  });

  it('should render each card with unique key', () => {
    const { container } = render(<SkillCardSkeletonGrid count={3} />);
    const cards = container.querySelectorAll('.card');

    // All cards should render
    expect(cards).toHaveLength(3);

    // Each should have the same structure
    cards.forEach((card) => {
      expect(card).toHaveClass('card', 'h-full', 'flex', 'flex-col');
    });
  });
});
