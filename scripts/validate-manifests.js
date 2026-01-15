#!/usr/bin/env node

/**
 * Skill Manifest Validation Script
 *
 * Validates all skill manifests in marketplace/skills/ against the JSON schema.
 * Checks for:
 * - Schema compliance
 * - Duplicate IDs
 * - Valid category references
 * - Required fields
 *
 * Usage: node scripts/validate-manifests.js
 */

const fs = require('fs');
const path = require('path');

// Simple JSON schema validator (no dependencies needed)
class Validator {
  constructor(schema) {
    this.schema = schema;
    this.errors = [];
  }

  validate(data, schema = this.schema, path = '') {
    this.errors = [];
    this._validate(data, schema, path);
    return this.errors.length === 0;
  }

  _validate(data, schema, currentPath) {
    // Check required fields
    if (schema.required) {
      for (const field of schema.required) {
        if (!(field in data)) {
          this.errors.push(`${currentPath}: Missing required field "${field}"`);
        }
      }
    }

    // Check type
    if (schema.type) {
      const actualType = Array.isArray(data) ? 'array' : typeof data;
      if (actualType !== schema.type && data !== null) {
        this.errors.push(`${currentPath}: Expected type "${schema.type}", got "${actualType}"`);
        return;
      }
    }

    // Check enum
    if (schema.enum && !schema.enum.includes(data)) {
      this.errors.push(`${currentPath}: Value "${data}" not in allowed values: ${schema.enum.join(', ')}`);
    }

    // Check pattern
    if (schema.pattern && typeof data === 'string') {
      const regex = new RegExp(schema.pattern);
      if (!regex.test(data)) {
        this.errors.push(`${currentPath}: Value "${data}" does not match pattern ${schema.pattern}`);
      }
    }

    // Check minLength/maxLength
    if (schema.minLength && typeof data === 'string' && data.length < schema.minLength) {
      this.errors.push(`${currentPath}: String length ${data.length} is less than minimum ${schema.minLength}`);
    }
    if (schema.maxLength && typeof data === 'string' && data.length > schema.maxLength) {
      this.errors.push(`${currentPath}: String length ${data.length} exceeds maximum ${schema.maxLength}`);
    }

    // Check minItems/maxItems
    if (schema.minItems && Array.isArray(data) && data.length < schema.minItems) {
      this.errors.push(`${currentPath}: Array length ${data.length} is less than minimum ${schema.minItems}`);
    }
    if (schema.maxItems && Array.isArray(data) && data.length > schema.maxItems) {
      this.errors.push(`${currentPath}: Array length ${data.length} exceeds maximum ${schema.maxItems}`);
    }

    // Validate object properties
    if (schema.type === 'object' && schema.properties) {
      for (const [key, value] of Object.entries(data)) {
        if (schema.properties[key]) {
          this._validate(value, schema.properties[key], `${currentPath}.${key}`);
        }
      }
    }

    // Validate array items
    if (schema.type === 'array' && schema.items && Array.isArray(data)) {
      data.forEach((item, index) => {
        this._validate(item, schema.items, `${currentPath}[${index}]`);
      });

      // Check uniqueItems
      if (schema.uniqueItems) {
        const uniqueSet = new Set(data.map(JSON.stringify));
        if (uniqueSet.size !== data.length) {
          this.errors.push(`${currentPath}: Array contains duplicate items`);
        }
      }
    }
  }
}

function main() {
  console.log('🔍 Validating skill manifests...\n');

  const rootDir = path.join(__dirname, '..');
  const schemaPath = path.join(rootDir, 'marketplace', 'schema.json');
  const categoriesPath = path.join(rootDir, 'marketplace', 'categories.json');
  const skillsDir = path.join(rootDir, 'marketplace', 'skills');

  // Load schema
  if (!fs.existsSync(schemaPath)) {
    console.error('❌ Schema file not found:', schemaPath);
    process.exit(1);
  }

  const schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
  console.log('✓ Loaded schema.json');

  // Load categories
  if (!fs.existsSync(categoriesPath)) {
    console.error('❌ Categories file not found:', categoriesPath);
    process.exit(1);
  }

  const categoriesData = JSON.parse(fs.readFileSync(categoriesPath, 'utf8'));
  const validCategories = categoriesData.categories.map(c => c.id);
  console.log(`✓ Loaded ${validCategories.length} valid categories\n`);

  // Check skills directory
  if (!fs.existsSync(skillsDir)) {
    console.error('❌ Skills directory not found:', skillsDir);
    process.exit(1);
  }

  // Get all JSON files
  const files = fs.readdirSync(skillsDir).filter(f => f.endsWith('.json'));

  if (files.length === 0) {
    console.warn('⚠️  No skill manifests found in', skillsDir);
    process.exit(0);
  }

  console.log(`Found ${files.length} skill manifest(s) to validate:\n`);

  let totalErrors = 0;
  const ids = new Set();
  const validator = new Validator(schema);

  // Validate each file
  for (const file of files) {
    const filePath = path.join(skillsDir, file);
    const skillId = path.basename(file, '.json');

    process.stdout.write(`  ${file.padEnd(30)} `);

    try {
      const content = fs.readFileSync(filePath, 'utf8');
      const skill = JSON.parse(content);

      let fileErrors = 0;

      // Validate against schema
      if (!validator.validate(skill, schema, skill.id || 'root')) {
        console.log('❌ Schema validation failed');
        validator.errors.forEach(error => {
          console.log(`     └─ ${error}`);
          fileErrors++;
        });
      }

      // Check if ID matches filename
      if (skill.id !== skillId) {
        console.log(`     └─ ID "${skill.id}" doesn't match filename "${skillId}"`);
        fileErrors++;
      }

      // Check for duplicate IDs
      if (ids.has(skill.id)) {
        console.log(`     └─ Duplicate ID "${skill.id}"`);
        fileErrors++;
      }
      ids.add(skill.id);

      // Validate category references
      if (skill.categories) {
        for (const category of skill.categories) {
          if (!validCategories.includes(category)) {
            console.log(`     └─ Invalid category "${category}". Valid categories: ${validCategories.join(', ')}`);
            fileErrors++;
          }
        }
      }

      // Check for required description length
      if (skill.description && (skill.description.length < 20 || skill.description.length > 200)) {
        console.log(`     └─ Description length ${skill.description.length} outside range 20-200 characters`);
        fileErrors++;
      }

      // Validate semver
      if (skill.version && !/^\d+\.\d+\.\d+$/.test(skill.version)) {
        console.log(`     └─ Invalid version format "${skill.version}". Expected semver (e.g., 1.0.0)`);
        fileErrors++;
      }

      if (fileErrors === 0) {
        console.log('✅');
      }

      totalErrors += fileErrors;

    } catch (error) {
      console.log('❌ Parse error');
      console.log(`     └─ ${error.message}`);
      totalErrors++;
    }
  }

  // Summary
  console.log('\n' + '─'.repeat(60));
  if (totalErrors === 0) {
    console.log(`\n✅ All ${files.length} skill manifests validated successfully!\n`);
    process.exit(0);
  } else {
    console.log(`\n❌ Validation failed with ${totalErrors} error(s)\n`);
    process.exit(1);
  }
}

// Run if executed directly
if (require.main === module) {
  main();
}

module.exports = { Validator };
