/**
 * Unit tests for HeroSection component
 */

import { render, screen } from '@testing-library/react';
import { HeroSection } from '@/components/home/HeroSection';

const mockStats = {
  totalSkills: 3,
  totalTools: 85,
  byType: {
    wasm: 1,
    native: 1,
    docker: 1,
  },
};

describe('HeroSection', () => {
  describe('Basic Rendering', () => {
    it('should render main heading', () => {
      render(<HeroSection stats={mockStats} />);
      expect(screen.getByText('Agentic Skills Library')).toBeInTheDocument();
    });

    it('should render heading as h1', () => {
      render(<HeroSection stats={mockStats} />);
      const heading = screen.getByText('Agentic Skills Library');
      expect(heading.tagName).toBe('H1');
    });

    it('should render description with skill count', () => {
      render(<HeroSection stats={mockStats} />);
      expect(screen.getByText(/A collection of 3 production-ready/)).toBeInTheDocument();
    });

    it('should mention runtime types', () => {
      render(<HeroSection stats={mockStats} />);
      expect(screen.getByText(/WASM, Native, and Docker runtime skills/)).toBeInTheDocument();
    });
  });

  describe('Stats Display', () => {
    it('should display total skills count', () => {
      render(<HeroSection stats={mockStats} />);
      const statsSection = screen.getByText('3').closest('.flex');
      expect(statsSection).toHaveTextContent('3');
      expect(statsSection).toHaveTextContent('skills');
    });

    it('should display total tools count', () => {
      render(<HeroSection stats={mockStats} />);
      const statsSection = screen.getByText('85').closest('.flex');
      expect(statsSection).toHaveTextContent('85');
      expect(statsSection).toHaveTextContent('tools');
    });

    it('should display WASM count', () => {
      render(<HeroSection stats={mockStats} />);
      const wasmStat = screen.getByText('WASM').closest('.flex');
      expect(wasmStat).toHaveTextContent('1');
      expect(wasmStat).toHaveTextContent('WASM');
    });

    it('should display Native count', () => {
      render(<HeroSection stats={mockStats} />);
      expect(screen.getByText('Native')).toBeInTheDocument();
    });

    it('should display Docker count', () => {
      render(<HeroSection stats={mockStats} />);
      expect(screen.getByText('Docker')).toBeInTheDocument();
    });

    it('should handle zero WASM skills', () => {
      const statsNoWasm = { ...mockStats, byType: { ...mockStats.byType, wasm: 0 } };
      render(<HeroSection stats={statsNoWasm} />);
      expect(screen.getByText('WASM')).toBeInTheDocument();
      const wasmStat = screen.getByText('WASM').closest('.flex');
      expect(wasmStat).toHaveTextContent('0');
    });

    it('should handle missing type counts', () => {
      const statsIncomplete = {
        totalSkills: 5,
        totalTools: 100,
        byType: {} as Record<string, number>,
      };
      render(<HeroSection stats={statsIncomplete} />);
      // Should default to 0 for missing types
      expect(screen.getByText('WASM')).toBeInTheDocument();
      expect(screen.getByText('Native')).toBeInTheDocument();
      expect(screen.getByText('Docker')).toBeInTheDocument();
    });
  });

  describe('Layout', () => {
    it('should have proper container styling', () => {
      const { container } = render(<HeroSection stats={mockStats} />);
      const heroContainer = container.querySelector('.py-12');
      expect(heroContainer).toBeInTheDocument();
      expect(heroContainer).toHaveClass('md:py-16');
    });

    it('should have max-width constraint', () => {
      const { container } = render(<HeroSection stats={mockStats} />);
      const content = container.querySelector('.max-w-4xl');
      expect(content).toBeInTheDocument();
    });

    it('should have stats in horizontal layout', () => {
      const { container } = render(<HeroSection stats={mockStats} />);
      const statsContainer = container.querySelector('.flex.items-center.gap-6');
      expect(statsContainer).toBeInTheDocument();
    });
  });

  describe('Typography', () => {
    it('should have large heading text', () => {
      render(<HeroSection stats={mockStats} />);
      const heading = screen.getByText('Agentic Skills Library');
      expect(heading).toHaveClass('text-3xl', 'md:text-4xl', 'font-bold');
    });

    it('should have readable description text', () => {
      const { container } = render(<HeroSection stats={mockStats} />);
      const description = container.querySelector('.text-lg');
      expect(description).toHaveClass('text-gray-600', 'leading-relaxed');
    });

    it('should emphasize stat numbers', () => {
      const { container } = render(<HeroSection stats={mockStats} />);
      const statNumbers = container.querySelectorAll('.font-semibold.text-gray-900');
      expect(statNumbers.length).toBeGreaterThan(0);
    });
  });

  describe('Different Stat Values', () => {
    it('should handle large numbers', () => {
      const largeStats = {
        totalSkills: 150,
        totalTools: 5000,
        byType: { wasm: 50, native: 75, docker: 25 },
      };
      render(<HeroSection stats={largeStats} />);
      expect(screen.getByText('150')).toBeInTheDocument();
      expect(screen.getByText('5000')).toBeInTheDocument();
      expect(screen.getByText('50')).toBeInTheDocument();
      expect(screen.getByText('75')).toBeInTheDocument();
      expect(screen.getByText('25')).toBeInTheDocument();
    });

    it('should handle single skill', () => {
      const singleStats = {
        totalSkills: 1,
        totalTools: 10,
        byType: { wasm: 1, native: 0, docker: 0 },
      };
      render(<HeroSection stats={singleStats} />);
      expect(screen.getByText(/A collection of 1 production-ready/)).toBeInTheDocument();
    });

    it('should update when stats change', () => {
      const { rerender } = render(<HeroSection stats={mockStats} />);
      expect(screen.getByText('3')).toBeInTheDocument();

      const newStats = { ...mockStats, totalSkills: 10 };
      rerender(<HeroSection stats={newStats} />);
      expect(screen.getByText('10')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have semantic heading structure', () => {
      render(<HeroSection stats={mockStats} />);
      const heading = screen.getByRole('heading', { level: 1 });
      expect(heading).toBeInTheDocument();
    });

    it('should have readable color contrast', () => {
      const { container } = render(<HeroSection stats={mockStats} />);
      const heading = container.querySelector('h1');
      expect(heading).toHaveClass('text-gray-900');
    });
  });

  describe('Responsive Design', () => {
    it('should have responsive padding', () => {
      const { container } = render(<HeroSection stats={mockStats} />);
      const section = container.firstChild as HTMLElement;
      expect(section).toHaveClass('py-12', 'md:py-16');
    });

    it('should have responsive heading size', () => {
      render(<HeroSection stats={mockStats} />);
      const heading = screen.getByText('Agentic Skills Library');
      expect(heading).toHaveClass('text-3xl', 'md:text-4xl');
    });
  });
});
