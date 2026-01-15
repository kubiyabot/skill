import { SkillManifest } from '@/lib/types';

export function generateCLI(skill: SkillManifest): string {
  const lines: string[] = [];

  lines.push('# Install the skill');

  if (skill.type === 'native' || skill.type === 'wasm') {
    lines.push(
      `skill install https://github.com/kubiyabot/skill/tree/main/${skill.installation.source}`
    );
  } else if (skill.type === 'docker') {
    lines.push('# Configure in .skill-engine.toml (see TOML tab), then run:');
    lines.push('skill list');
  }

  lines.push('');
  lines.push('# Configure (if authentication required)');
  if (skill.installation.requiresAuth) {
    lines.push(`skill config ${skill.id}`);
    lines.push('');
  }

  lines.push('# List available tools');
  lines.push(`skill info ${skill.id}`);

  lines.push('');
  lines.push('# Example usage');
  if (skill.tools && skill.tools.length > 0) {
    const firstTool = skill.tools[0];
    lines.push(`skill run ${skill.id}:${firstTool.name}`);
  }

  return lines.join('\n');
}
