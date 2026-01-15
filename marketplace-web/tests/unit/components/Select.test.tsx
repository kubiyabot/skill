/**
 * Unit tests for Select component
 */

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Select } from '@/components/ui/Select';

describe('Select', () => {
  const mockOptions = [
    { value: 'us', label: 'United States' },
    { value: 'uk', label: 'United Kingdom' },
    { value: 'ca', label: 'Canada' },
  ];

  describe('Basic Rendering', () => {
    it('should render select element', () => {
      render(<Select />);
      expect(screen.getByRole('combobox')).toBeInTheDocument();
    });

    it('should apply custom className', () => {
      render(<Select className="custom-class" />);
      expect(screen.getByRole('combobox')).toHaveClass('custom-class');
    });

    it('should render custom chevron icon', () => {
      const { container } = render(<Select />);
      const chevron = container.querySelector('svg');
      expect(chevron).toBeInTheDocument();
    });

    it('should have appearance-none class to hide default arrow', () => {
      render(<Select />);
      expect(screen.getByRole('combobox')).toHaveClass('appearance-none');
    });
  });

  describe('Options Rendering', () => {
    it('should render options from options prop', () => {
      render(<Select options={mockOptions} />);
      const select = screen.getByRole('combobox');
      const options = Array.from(select.querySelectorAll('option'));

      expect(options).toHaveLength(3);
      expect(options[0]).toHaveTextContent('United States');
      expect(options[0]).toHaveValue('us');
      expect(options[1]).toHaveTextContent('United Kingdom');
      expect(options[1]).toHaveValue('uk');
      expect(options[2]).toHaveTextContent('Canada');
      expect(options[2]).toHaveValue('ca');
    });

    it('should render children when options prop not provided', () => {
      render(
        <Select>
          <option value="opt1">Option 1</option>
          <option value="opt2">Option 2</option>
        </Select>
      );

      const select = screen.getByRole('combobox');
      const options = Array.from(select.querySelectorAll('option'));

      expect(options).toHaveLength(2);
      expect(options[0]).toHaveTextContent('Option 1');
      expect(options[1]).toHaveTextContent('Option 2');
    });

    it('should support disabled options', () => {
      const optionsWithDisabled = [
        { value: 'us', label: 'United States' },
        { value: 'uk', label: 'United Kingdom', disabled: true },
        { value: 'ca', label: 'Canada' },
      ];

      render(<Select options={optionsWithDisabled} />);
      const select = screen.getByRole('combobox');
      const options = Array.from(select.querySelectorAll('option'));

      expect(options[1]).toBeDisabled();
      expect(options[0]).not.toBeDisabled();
      expect(options[2]).not.toBeDisabled();
    });
  });

  describe('Label', () => {
    it('should render with label', () => {
      render(<Select label="Country" options={mockOptions} />);
      expect(screen.getByLabelText('Country')).toBeInTheDocument();
    });

    it('should associate label with select', () => {
      render(<Select label="Country" id="country" options={mockOptions} />);
      const select = screen.getByRole('combobox');
      const label = screen.getByText('Country');
      expect(label).toHaveAttribute('for', 'country');
      expect(select).toHaveAttribute('id', 'country');
    });

    it('should show asterisk for required fields', () => {
      render(<Select label="Country" required options={mockOptions} />);
      expect(screen.getByText('*')).toBeInTheDocument();
    });

    it('should not show asterisk for non-required fields', () => {
      render(<Select label="Country" options={mockOptions} />);
      expect(screen.queryByText('*')).not.toBeInTheDocument();
    });
  });

  describe('Error State', () => {
    it('should display error message', () => {
      render(<Select label="Country" error="Country is required" options={mockOptions} />);
      expect(screen.getByText('Country is required')).toBeInTheDocument();
    });

    it('should have error styling', () => {
      render(<Select error="Error message" options={mockOptions} />);
      const select = screen.getByRole('combobox');
      expect(select).toHaveClass('border-red-300');
    });

    it('should set aria-invalid when error exists', () => {
      render(<Select error="Error" options={mockOptions} />);
      expect(screen.getByRole('combobox')).toHaveAttribute('aria-invalid', 'true');
    });

    it('should associate error with select via aria-describedby', () => {
      render(<Select id="country" error="Error message" options={mockOptions} />);
      const select = screen.getByRole('combobox');
      const errorId = select.getAttribute('aria-describedby');
      expect(errorId).toBeTruthy();
      expect(screen.getByText('Error message')).toHaveAttribute('id', errorId!);
    });

    it('should have role="alert" on error message', () => {
      render(<Select error="Error message" options={mockOptions} />);
      const errorElement = screen.getByText('Error message');
      expect(errorElement).toHaveAttribute('role', 'alert');
    });
  });

  describe('Hint Text', () => {
    it('should display hint message', () => {
      render(<Select hint="Choose your country" options={mockOptions} />);
      expect(screen.getByText('Choose your country')).toBeInTheDocument();
    });

    it('should not show hint when error exists', () => {
      render(<Select hint="Hint text" error="Error text" options={mockOptions} />);
      expect(screen.queryByText('Hint text')).not.toBeInTheDocument();
      expect(screen.getByText('Error text')).toBeInTheDocument();
    });

    it('should associate hint with select via aria-describedby', () => {
      render(<Select id="country" hint="Hint message" options={mockOptions} />);
      const select = screen.getByRole('combobox');
      const hintId = select.getAttribute('aria-describedby');
      expect(hintId).toBeTruthy();
      expect(screen.getByText('Hint message')).toHaveAttribute('id', hintId!);
    });
  });

  describe('Disabled State', () => {
    it('should be disabled when disabled prop is true', () => {
      render(<Select disabled options={mockOptions} />);
      expect(screen.getByRole('combobox')).toBeDisabled();
    });

    it('should have disabled styling', () => {
      render(<Select disabled options={mockOptions} />);
      expect(screen.getByRole('combobox')).toHaveClass('disabled:bg-gray-50');
    });

    it('should not change value when disabled', async () => {
      const user = userEvent.setup();
      render(<Select disabled value="us" options={mockOptions} />);
      const select = screen.getByRole('combobox');

      // Attempt to change selection (should not work)
      await user.selectOptions(select, 'uk');
      expect(select).toHaveValue('us');
    });
  });

  describe('User Interaction', () => {
    it('should change value on selection', async () => {
      const user = userEvent.setup();
      render(<Select options={mockOptions} />);
      const select = screen.getByRole('combobox');

      await user.selectOptions(select, 'uk');
      expect(select).toHaveValue('uk');
    });

    it('should call onChange handler', async () => {
      const handleChange = jest.fn();
      const user = userEvent.setup();
      render(<Select onChange={handleChange} options={mockOptions} />);
      const select = screen.getByRole('combobox');

      await user.selectOptions(select, 'ca');
      expect(handleChange).toHaveBeenCalled();
    });

    it('should call onBlur handler', async () => {
      const handleBlur = jest.fn();
      const user = userEvent.setup();
      render(<Select onBlur={handleBlur} options={mockOptions} />);

      const select = screen.getByRole('combobox');
      select.focus();
      await user.tab();

      expect(handleBlur).toHaveBeenCalled();
    });

    it('should respect controlled value', () => {
      const { rerender } = render(<Select value="us" onChange={() => {}} options={mockOptions} />);
      expect(screen.getByRole('combobox')).toHaveValue('us');

      rerender(<Select value="ca" onChange={() => {}} options={mockOptions} />);
      expect(screen.getByRole('combobox')).toHaveValue('ca');
    });
  });

  describe('Ref Forwarding', () => {
    it('should forward ref to select element', () => {
      const ref = { current: null };
      render(<Select ref={ref as any} options={mockOptions} />);
      expect(ref.current).toBeInstanceOf(HTMLSelectElement);
    });

    it('should allow ref access to select methods', () => {
      const ref = { current: null as HTMLSelectElement | null };
      render(<Select ref={ref as any} options={mockOptions} />);
      expect(ref.current?.focus).toBeDefined();
      expect(ref.current?.blur).toBeDefined();
    });
  });

  describe('Accessibility', () => {
    it('should have proper focus styles', () => {
      render(<Select options={mockOptions} />);
      expect(screen.getByRole('combobox')).toHaveClass('focus:ring-2');
    });

    it('should support aria-label', () => {
      render(<Select aria-label="Country" options={mockOptions} />);
      expect(screen.getByLabelText('Country')).toBeInTheDocument();
    });

    it('should have aria-invalid false when no error', () => {
      render(<Select options={mockOptions} />);
      expect(screen.getByRole('combobox')).toHaveAttribute('aria-invalid', 'false');
    });
  });

  describe('Layout', () => {
    it('should have relative wrapper for chevron positioning', () => {
      const { container } = render(<Select options={mockOptions} />);
      const wrapper = container.querySelector('.relative');
      expect(wrapper).toBeInTheDocument();
    });

    it('should have pointer-events-none on chevron container', () => {
      const { container } = render(<Select options={mockOptions} />);
      const chevronContainer = container.querySelector('.pointer-events-none');
      expect(chevronContainer).toBeInTheDocument();
    });

    it('should have padding-right for chevron space', () => {
      render(<Select options={mockOptions} />);
      expect(screen.getByRole('combobox')).toHaveClass('pr-10');
    });
  });
});
