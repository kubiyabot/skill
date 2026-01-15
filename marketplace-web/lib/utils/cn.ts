/**
 * Utility for merging Tailwind CSS classes
 * Simple implementation without external dependencies
 */

export function cn(...classes: (string | undefined | null | false)[]): string {
  return classes.filter(Boolean).join(' ');
}
