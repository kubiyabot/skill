/**
 * Unit tests for Textarea component
 */

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Textarea } from '@/components/ui/Textarea';

describe('Textarea', () => {
  describe('Basic Rendering', () => {
    it('should render textarea element', () => {
      render(<Textarea />);
      expect(screen.getByRole('textbox')).toBeInTheDocument();
    });

    it('should render with placeholder', () => {
      render(<Textarea placeholder="Enter your message..." />);
      expect(screen.getByPlaceholderText('Enter your message...')).toBeInTheDocument();
    });

    it('should render with value', () => {
      render(<Textarea value="Test message" readOnly />);
      expect(screen.getByRole('textbox')).toHaveValue('Test message');
    });

    it('should apply custom className', () => {
      render(<Textarea className="custom-class" />);
      expect(screen.getByRole('textbox')).toHaveClass('custom-class');
    });

    it('should use default rows of 4', () => {
      render(<Textarea />);
      expect(screen.getByRole('textbox')).toHaveAttribute('rows', '4');
    });

    it('should use custom rows value', () => {
      render(<Textarea rows={8} />);
      expect(screen.getByRole('textbox')).toHaveAttribute('rows', '8');
    });
  });

  describe('Label', () => {
    it('should render with label', () => {
      render(<Textarea label="Description" />);
      expect(screen.getByLabelText('Description')).toBeInTheDocument();
    });

    it('should associate label with textarea', () => {
      render(<Textarea label="Comments" id="comments" />);
      const textarea = screen.getByRole('textbox');
      const label = screen.getByText('Comments');
      expect(label).toHaveAttribute('for', 'comments');
      expect(textarea).toHaveAttribute('id', 'comments');
    });

    it('should show asterisk for required fields', () => {
      render(<Textarea label="Message" required />);
      expect(screen.getByText('*')).toBeInTheDocument();
    });

    it('should not show asterisk for non-required fields', () => {
      render(<Textarea label="Message" />);
      expect(screen.queryByText('*')).not.toBeInTheDocument();
    });
  });

  describe('Error State', () => {
    it('should display error message', () => {
      render(<Textarea label="Message" error="Message is required" />);
      expect(screen.getByText('Message is required')).toBeInTheDocument();
    });

    it('should have error styling', () => {
      render(<Textarea error="Error message" />);
      const textarea = screen.getByRole('textbox');
      expect(textarea).toHaveClass('border-red-300');
    });

    it('should set aria-invalid when error exists', () => {
      render(<Textarea error="Error" />);
      expect(screen.getByRole('textbox')).toHaveAttribute('aria-invalid', 'true');
    });

    it('should associate error with textarea via aria-describedby', () => {
      render(<Textarea id="message" error="Error message" />);
      const textarea = screen.getByRole('textbox');
      const errorId = textarea.getAttribute('aria-describedby');
      expect(errorId).toBeTruthy();
      expect(screen.getByText('Error message')).toHaveAttribute('id', errorId!);
    });

    it('should have role="alert" on error message', () => {
      render(<Textarea error="Error message" />);
      const errorElement = screen.getByText('Error message');
      expect(errorElement).toHaveAttribute('role', 'alert');
    });
  });

  describe('Hint Text', () => {
    it('should display hint message', () => {
      render(<Textarea hint="Max 500 characters" />);
      expect(screen.getByText('Max 500 characters')).toBeInTheDocument();
    });

    it('should not show hint when error exists', () => {
      render(<Textarea hint="Hint text" error="Error text" />);
      expect(screen.queryByText('Hint text')).not.toBeInTheDocument();
      expect(screen.getByText('Error text')).toBeInTheDocument();
    });

    it('should associate hint with textarea via aria-describedby', () => {
      render(<Textarea id="message" hint="Hint message" />);
      const textarea = screen.getByRole('textbox');
      const hintId = textarea.getAttribute('aria-describedby');
      expect(hintId).toBeTruthy();
      expect(screen.getByText('Hint message')).toHaveAttribute('id', hintId!);
    });
  });

  describe('Disabled State', () => {
    it('should be disabled when disabled prop is true', () => {
      render(<Textarea disabled />);
      expect(screen.getByRole('textbox')).toBeDisabled();
    });

    it('should have disabled styling', () => {
      render(<Textarea disabled />);
      expect(screen.getByRole('textbox')).toHaveClass('disabled:bg-gray-50');
    });

    it('should not accept input when disabled', async () => {
      const user = userEvent.setup();
      render(<Textarea disabled />);
      const textarea = screen.getByRole('textbox');

      await user.type(textarea, 'test');
      expect(textarea).toHaveValue('');
    });

    it('should disable resize when disabled', () => {
      render(<Textarea disabled />);
      expect(screen.getByRole('textbox')).toHaveClass('disabled:resize-none');
    });
  });

  describe('Resize Behavior', () => {
    it('should allow vertical resize by default', () => {
      render(<Textarea />);
      expect(screen.getByRole('textbox')).toHaveClass('resize-y');
    });

    it('should not have horizontal resize', () => {
      const { container } = render(<Textarea />);
      const textarea = screen.getByRole('textbox');
      const styles = window.getComputedStyle(textarea);
      // resize-y class prevents horizontal resizing
      expect(textarea).toHaveClass('resize-y');
    });
  });

  describe('User Interaction', () => {
    it('should accept user input', async () => {
      const user = userEvent.setup();
      render(<Textarea />);
      const textarea = screen.getByRole('textbox');

      await user.type(textarea, 'Hello\nWorld');
      expect(textarea).toHaveValue('Hello\nWorld');
    });

    it('should call onChange handler', async () => {
      const handleChange = jest.fn();
      const user = userEvent.setup();
      render(<Textarea onChange={handleChange} />);

      await user.type(screen.getByRole('textbox'), 'A');
      expect(handleChange).toHaveBeenCalled();
    });

    it('should call onBlur handler', async () => {
      const handleBlur = jest.fn();
      const user = userEvent.setup();
      render(<Textarea onBlur={handleBlur} />);

      const textarea = screen.getByRole('textbox');
      textarea.focus();
      await user.tab();

      expect(handleBlur).toHaveBeenCalled();
    });

    it('should support multi-line text', async () => {
      const user = userEvent.setup();
      render(<Textarea />);
      const textarea = screen.getByRole('textbox');

      await user.type(textarea, 'Line 1{Enter}Line 2{Enter}Line 3');
      expect(textarea).toHaveValue('Line 1\nLine 2\nLine 3');
    });
  });

  describe('Ref Forwarding', () => {
    it('should forward ref to textarea element', () => {
      const ref = { current: null };
      render(<Textarea ref={ref as any} />);
      expect(ref.current).toBeInstanceOf(HTMLTextAreaElement);
    });

    it('should allow ref access to textarea methods', () => {
      const ref = { current: null as HTMLTextAreaElement | null };
      render(<Textarea ref={ref as any} />);
      expect(ref.current?.focus).toBeDefined();
      expect(ref.current?.select).toBeDefined();
    });
  });

  describe('Accessibility', () => {
    it('should have proper focus styles', () => {
      render(<Textarea />);
      expect(screen.getByRole('textbox')).toHaveClass('focus:ring-2');
    });

    it('should support aria-label', () => {
      render(<Textarea aria-label="Message" />);
      expect(screen.getByLabelText('Message')).toBeInTheDocument();
    });

    it('should have aria-invalid false when no error', () => {
      render(<Textarea />);
      expect(screen.getByRole('textbox')).toHaveAttribute('aria-invalid', 'false');
    });
  });
});
