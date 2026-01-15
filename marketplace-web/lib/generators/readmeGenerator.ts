import { SkillManifest } from '@/lib/types';

export function generateGitHubInstructions(skill: SkillManifest): string {
  const lines: string[] = [];

  lines.push('# Download from GitHub');
  lines.push('');
  lines.push('1. Clone the repository:');
  lines.push('   ```bash');
  lines.push('   git clone https://github.com/kubiyabot/skill.git');
  lines.push('   cd skill');
  lines.push('   ```');
  lines.push('');

  lines.push('2. Install the skill:');
  lines.push('   ```bash');
  lines.push(`   skill install ./${skill.installation.source}`);
  lines.push('   ```');
  lines.push('');

  if (skill.installation.requiresAuth) {
    lines.push('3. Configure credentials:');
    lines.push('   ```bash');
    lines.push(`   skill config ${skill.id}`);
    lines.push('   ```');
    lines.push('');

    if (skill.installation.envVars && skill.installation.envVars.length > 0) {
      lines.push('   Required environment variables:');
      skill.installation.envVars
        .filter((v) => v.required)
        .forEach((envVar) => {
          lines.push(`   - \`${envVar.name}\`: ${envVar.description || ''}`);
        });
      lines.push('');
    }

    lines.push('4. Verify installation:');
  } else {
    lines.push("3. You're ready to use it!");
    lines.push('');
    lines.push('4. Verify installation:');
  }

  lines.push('   ```bash');
  lines.push('   skill list');
  lines.push(`   skill info ${skill.id}`);
  lines.push('   ```');

  return lines.join('\n');
}
