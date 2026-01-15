import { SkillManifest } from '@/lib/types';

export function generateTOML(skill: SkillManifest): string {
  const lines: string[] = [];

  lines.push(`[skills.${skill.id}]`);
  lines.push(`source = "${skill.installation.source}"`);
  lines.push(`description = "${skill.description}"`);

  if (skill.type === 'docker') {
    lines.push(`runtime = "docker"`);
    lines.push('');
    lines.push(`[skills.${skill.id}.docker]`);

    const dockerConfig = skill.installation.dockerConfig;
    if (dockerConfig) {
      if (dockerConfig.image) lines.push(`image = "${dockerConfig.image}"`);
      if (dockerConfig.entrypoint)
        lines.push(`entrypoint = "${dockerConfig.entrypoint}"`);
      if (dockerConfig.volumes)
        lines.push(`volumes = ${JSON.stringify(dockerConfig.volumes)}`);
      if (dockerConfig.workingDir)
        lines.push(`working_dir = "${dockerConfig.workingDir}"`);
      if (dockerConfig.memory) lines.push(`memory = "${dockerConfig.memory}"`);
      if (dockerConfig.network)
        lines.push(`network = "${dockerConfig.network}"`);
    }
  }

  lines.push('');
  lines.push(`[skills.${skill.id}.instances.default]`);

  if (skill.installation.envVars && skill.installation.envVars.length > 0) {
    const requiredVars = skill.installation.envVars
      .filter((v) => v.required)
      .map((v) => v.name);
    if (requiredVars.length > 0) {
      lines.push(`# Required environment variables: ${requiredVars.join(', ')}`);
    }
  }

  return lines.join('\n');
}
