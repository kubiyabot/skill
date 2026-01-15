/**
 * Application constants
 */

export const SITE_NAME = 'Skills Marketplace';
export const SITE_DESCRIPTION =
  'Browse production-ready agentic skills. Self-contained WASM, Native, and Docker runtime skills for DevOps, cloud, APIs, and more.';
export const SITE_URL = 'https://marketplace.skill.dev';
export const GITHUB_REPO = 'https://github.com/kubiyabot/skill';

export const TYPE_COLORS = {
  wasm: 'text-skill-wasm',
  native: 'text-skill-native',
  docker: 'text-skill-docker',
} as const;

export const TYPE_BG_COLORS = {
  wasm: 'bg-skill-wasm/10',
  native: 'bg-skill-native/10',
  docker: 'bg-skill-docker/10',
} as const;

export const BADGE_COLORS = {
  official: 'bg-badge-official text-white',
  featured: 'bg-badge-featured text-white',
  verified: 'bg-badge-verified text-white',
  community: 'bg-badge-community text-white',
} as const;
