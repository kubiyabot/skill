/**
 * TypeScript types for Skills Marketplace
 * Based on marketplace/schema.json
 */

export type SkillType = 'wasm' | 'native' | 'docker';

export type SkillBadge = 'official' | 'featured' | 'community' | 'verified';

export type CategoryId =
  | 'featured'
  | 'devops'
  | 'cloud'
  | 'database'
  | 'api'
  | 'messaging'
  | 'monitoring'
  | 'media'
  | 'development'
  | 'utilities';

export type Platform = 'linux' | 'macos' | 'windows';

export interface SkillAuthor {
  name: string;
  email?: string;
  url?: string;
  github?: string;
}

export interface EnvVar {
  name: string;
  required?: boolean;
  description?: string;
  default?: string;
}

export interface Service {
  name: string;
  description?: string;
  port?: number;
  optional?: boolean;
}

export interface DockerConfig {
  image?: string;
  entrypoint?: string;
  volumes?: string[];
  workingDir?: string;
  memory?: string;
  network?: string;
}

export interface Installation {
  source: string;
  requiresAuth?: boolean;
  envVars?: EnvVar[];
  services?: Service[];
  dockerConfig?: DockerConfig;
}

export interface Requirements {
  cli?: string[];
  platform?: Platform[];
}

export interface ToolParameter {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object' | 'file';
  required?: boolean;
  description?: string;
  default?: any;
  enum?: string[];
}

export interface Tool {
  name: string;
  description: string;
  parameters?: ToolParameter[];
}

export interface Example {
  title: string;
  description?: string;
  code: string;
}

export interface Links {
  github?: string;
  documentation?: string;
  homepage?: string;
}

export interface Stats {
  downloads?: number;
  stars?: number;
  lastUpdated?: string;
}

export interface SkillManifest {
  id: string;
  name: string;
  icon?: string; // URL to SVG icon
  skillMdPath?: string; // Local path to SKILL.md file
  skillMdUrl?: string; // URL to SKILL.md file
  skillMdContent?: string; // Loaded SKILL.md content (populated at build time)
  type: SkillType;
  description: string;
  longDescription?: string;
  version: string;
  author: SkillAuthor;
  categories: CategoryId[];
  badges?: SkillBadge[];
  installation: Installation;
  requirements?: Requirements;
  tools?: Tool[];
  examples?: Example[];
  links?: Links;
  stats?: Stats;
}

export interface Category {
  id: CategoryId;
  name: string;
  icon: string;
  color: string;
  description: string;
}

export interface CategoriesData {
  categories: Category[];
}

// Extended types for UI (computed fields)
export interface SkillCardData extends SkillManifest {
  toolsCount: number;
  slug: string;
}

// Search and filter types
export interface SearchFilters {
  query: string;
  categories: CategoryId[];
  types: SkillType[];
  badges: SkillBadge[];
}
