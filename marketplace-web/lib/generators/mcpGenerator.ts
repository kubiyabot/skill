import { SkillManifest } from '@/lib/types';

export function generateMCP(skill: SkillManifest): string {
  const config: any = {
    mcpServers: {
      'skill-engine': {
        command: 'skill',
        args: ['serve'],
        env: {},
      },
    },
  };

  // Add required environment variables for this skill
  if (skill.installation.envVars && skill.installation.envVars.length > 0) {
    skill.installation.envVars.forEach((envVar) => {
      const placeholder = envVar.required
        ? `<your-${envVar.name.toLowerCase().replace(/_/g, '-')}>`
        : envVar.default || '';
      config.mcpServers['skill-engine'].env[envVar.name] = placeholder;
    });
  }

  // Add comment about skill installation
  const output = [
    '// Step 1: Install the skill first using TOML or CLI method',
    '// Step 2: Add this to your .mcp.json configuration',
    '',
    JSON.stringify(config, null, 2),
  ];

  // Add environment variable descriptions as comments
  if (skill.installation.envVars && skill.installation.envVars.length > 0) {
    output.push('');
    output.push('// Environment variables:');
    skill.installation.envVars.forEach((envVar) => {
      const req = envVar.required ? '(required)' : '(optional)';
      output.push(`// - ${envVar.name} ${req}: ${envVar.description}`);
    });
  }

  return output.join('\n');
}
