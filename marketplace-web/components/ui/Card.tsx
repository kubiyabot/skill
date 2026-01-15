import { ReactNode } from 'react';
import { cn } from '@/lib/utils/cn';

interface CardProps {
  children: ReactNode;
  className?: string;
  hover?: boolean;
}

export function Card({ children, className, hover = false }: CardProps) {
  return (
    <div
      className={cn(
        'rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6',
        hover &&
          'transition-all duration-200 hover:shadow-card-hover hover:-translate-y-1 cursor-pointer',
        !hover && 'shadow-card',
        className
      )}
    >
      {children}
    </div>
  );
}
