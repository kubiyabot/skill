/**
 * Loading spinner component for inline loading states
 */

import { Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils/cn';

export type SpinnerSize = 'sm' | 'md' | 'lg';

export interface LoadingSpinnerProps {
  size?: SpinnerSize;
  className?: string;
  label?: string;
}

const sizeMap: Record<SpinnerSize, number> = {
  sm: 16,
  md: 24,
  lg: 32,
};

/**
 * Loading spinner with icon animation
 *
 * @example
 * ```tsx
 * <LoadingSpinner size="md" label="Loading skills..." />
 * ```
 */
export function LoadingSpinner({ size = 'md', className, label }: LoadingSpinnerProps) {
  return (
    <div
      className={cn('flex items-center justify-center gap-2', className)}
      role="status"
      aria-label={label || 'Loading'}
    >
      <Loader2 className="animate-spin text-blue-600" size={sizeMap[size]} />
      {label && <span className="text-sm text-gray-600">{label}</span>}
    </div>
  );
}

/**
 * Full page loading spinner
 */
export function LoadingSpinnerFullPage({ label = 'Loading...' }: { label?: string }) {
  return (
    <div className="fixed inset-0 flex items-center justify-center bg-white bg-opacity-75 z-50">
      <LoadingSpinner size="lg" label={label} />
    </div>
  );
}

/**
 * Centered loading spinner for content areas
 */
export function LoadingSpinnerCentered({ label }: { label?: string }) {
  return (
    <div className="flex items-center justify-center py-12">
      <LoadingSpinner size="md" label={label} />
    </div>
  );
}
