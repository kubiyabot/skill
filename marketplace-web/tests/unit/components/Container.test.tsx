/**
 * Unit tests for Container component
 */

import { render, screen } from '@testing-library/react';
import { Container } from '@/components/layout/Container';

describe('Container', () => {
  describe('Basic Rendering', () => {
    it('should render children content', () => {
      render(<Container>Container Content</Container>);
      expect(screen.getByText('Container Content')).toBeInTheDocument();
    });

    it('should render complex children', () => {
      render(
        <Container>
          <h2>Title</h2>
          <p>Description</p>
        </Container>
      );
      expect(screen.getByText('Title')).toBeInTheDocument();
      expect(screen.getByText('Description')).toBeInTheDocument();
    });
  });

  describe('Styling', () => {
    it('should have container class', () => {
      const { container } = render(<Container>Content</Container>);
      const div = container.firstChild as HTMLElement;
      expect(div).toHaveClass('container');
    });

    it('should have mx-auto for centering', () => {
      const { container } = render(<Container>Content</Container>);
      const div = container.firstChild as HTMLElement;
      expect(div).toHaveClass('mx-auto');
    });

    it('should have horizontal padding', () => {
      const { container } = render(<Container>Content</Container>);
      const div = container.firstChild as HTMLElement;
      expect(div).toHaveClass('px-4');
    });

    it('should apply custom className', () => {
      const { container } = render(<Container className="custom-class">Content</Container>);
      const div = container.firstChild as HTMLElement;
      expect(div).toHaveClass('custom-class');
    });

    it('should maintain base classes with custom className', () => {
      const { container } = render(<Container className="custom-class">Content</Container>);
      const div = container.firstChild as HTMLElement;
      expect(div).toHaveClass('custom-class', 'container', 'mx-auto', 'px-4');
    });
  });

  describe('Children Types', () => {
    it('should render string children', () => {
      render(<Container>Simple text</Container>);
      expect(screen.getByText('Simple text')).toBeInTheDocument();
    });

    it('should render JSX children', () => {
      render(
        <Container>
          <div data-testid="child">JSX Content</div>
        </Container>
      );
      expect(screen.getByTestId('child')).toBeInTheDocument();
    });

    it('should render multiple children', () => {
      render(
        <Container>
          <p>First</p>
          <p>Second</p>
          <p>Third</p>
        </Container>
      );
      expect(screen.getByText('First')).toBeInTheDocument();
      expect(screen.getByText('Second')).toBeInTheDocument();
      expect(screen.getByText('Third')).toBeInTheDocument();
    });
  });
});
