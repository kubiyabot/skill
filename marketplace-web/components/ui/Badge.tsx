import { SkillBadge } from '@/lib/types';
import { BADGE_COLORS } from '@/lib/utils/constants';
import { cn } from '@/lib/utils/cn';

interface BadgeProps {
  variant: SkillBadge;
  children: React.ReactNode;
  className?: string;
}

export function Badge({ variant, children, className }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium',
        BADGE_COLORS[variant],
        className
      )}
    >
      {children}
    </span>
  );
}
