/**
 * Setup verification test
 * This test ensures Jest and React Testing Library are configured correctly
 */

describe('Testing Infrastructure', () => {
  it('should run Jest successfully', () => {
    expect(true).toBe(true);
  });

  it('should have access to jest-dom matchers', () => {
    const element = document.createElement('div');
    element.textContent = 'Hello World';
    document.body.appendChild(element);

    expect(element).toBeInTheDocument();
    expect(element).toHaveTextContent('Hello World');

    document.body.removeChild(element);
  });

  it('should support TypeScript', () => {
    const message: string = 'TypeScript works';
    expect(message).toBe('TypeScript works');
  });
});
