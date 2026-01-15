/**
 * Unit tests for Card component
 */

import { render, screen } from '@testing-library/react';
import { Card } from '@/components/ui/Card';

describe('Card', () => {
  describe('Basic Rendering', () => {
    it('should render children content', () => {
      render(<Card>Card Content</Card>);
      expect(screen.getByText('Card Content')).toBeInTheDocument();
    });

    it('should render complex children', () => {
      render(
        <Card>
          <h2>Title</h2>
          <p>Description</p>
        </Card>
      );
      expect(screen.getByText('Title')).toBeInTheDocument();
      expect(screen.getByText('Description')).toBeInTheDocument();
    });

    it('should apply custom className', () => {
      const { container } = render(<Card className="custom-class">Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('custom-class');
    });
  });

  describe('Base Styling', () => {
    it('should have border and rounded corners', () => {
      const { container } = render(<Card>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('rounded-lg', 'border', 'border-gray-200');
    });

    it('should have white background', () => {
      const { container } = render(<Card>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('bg-white');
    });

    it('should have padding', () => {
      const { container } = render(<Card>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('p-6');
    });

    it('should have dark mode classes', () => {
      const { container } = render(<Card>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('dark:border-gray-800', 'dark:bg-gray-900');
    });
  });

  describe('Hover Variant', () => {
    it('should not have hover effects by default', () => {
      const { container } = render(<Card>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('shadow-card');
      expect(card).not.toHaveClass('cursor-pointer');
    });

    it('should have hover effects when hover prop is true', () => {
      const { container } = render(<Card hover>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('transition-all', 'duration-200', 'cursor-pointer');
      expect(card).toHaveClass('hover:shadow-card-hover', 'hover:-translate-y-1');
    });

    it('should have shadow-card when hover is false', () => {
      const { container } = render(<Card hover={false}>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('shadow-card');
    });

    it('should not have shadow-card when hover is true', () => {
      const { container } = render(<Card hover={true}>Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).not.toHaveClass('shadow-card');
    });
  });

  describe('Props Combinations', () => {
    it('should work with hover and custom className', () => {
      const { container } = render(
        <Card hover className="custom-class">
          Content
        </Card>
      );
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('custom-class', 'cursor-pointer');
    });

    it('should maintain base styles with custom className', () => {
      const { container } = render(<Card className="custom-class">Content</Card>);
      const card = container.firstChild as HTMLElement;
      expect(card).toHaveClass('custom-class', 'rounded-lg', 'border', 'p-6');
    });
  });

  describe('Children Types', () => {
    it('should render string children', () => {
      render(<Card>Simple text</Card>);
      expect(screen.getByText('Simple text')).toBeInTheDocument();
    });

    it('should render JSX children', () => {
      render(
        <Card>
          <div data-testid="child">JSX Content</div>
        </Card>
      );
      expect(screen.getByTestId('child')).toBeInTheDocument();
    });

    it('should render multiple children', () => {
      render(
        <Card>
          <p>First</p>
          <p>Second</p>
          <p>Third</p>
        </Card>
      );
      expect(screen.getByText('First')).toBeInTheDocument();
      expect(screen.getByText('Second')).toBeInTheDocument();
      expect(screen.getByText('Third')).toBeInTheDocument();
    });

    it('should render nested components', () => {
      render(
        <Card>
          <Card>Nested Card</Card>
        </Card>
      );
      expect(screen.getByText('Nested Card')).toBeInTheDocument();
    });
  });

  describe('Multiple Cards', () => {
    it('should render multiple cards independently', () => {
      const { container } = render(
        <div>
          <Card>Card 1</Card>
          <Card hover>Card 2</Card>
          <Card className="custom">Card 3</Card>
        </div>
      );

      expect(screen.getByText('Card 1')).toBeInTheDocument();
      expect(screen.getByText('Card 2')).toBeInTheDocument();
      expect(screen.getByText('Card 3')).toBeInTheDocument();

      const cards = container.querySelectorAll('.rounded-lg');
      expect(cards).toHaveLength(3);
    });
  });
});
