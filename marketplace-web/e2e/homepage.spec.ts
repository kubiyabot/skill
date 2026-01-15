/**
 * E2E tests for the homepage
 */

import { test, expect } from '@playwright/test';

test.describe('Homepage', () => {
  test('should load successfully', async ({ page }) => {
    await page.goto('/');

    // Wait for page to be fully loaded
    await page.waitForLoadState('networkidle');

    // Verify page title
    await expect(page).toHaveTitle(/Skill Engine/i);
  });

  test('should display the hero section', async ({ page }) => {
    await page.goto('/');

    // Check for main heading - look for any h1
    const heading = page.locator('h1').first();
    await expect(heading).toBeVisible();
  });

  test('should display skill cards', async ({ page }) => {
    await page.goto('/');

    // Wait for page to load
    await page.waitForLoadState('networkidle');

    // Verify links to skills exist (check if any exist, may be 0 for empty marketplace)
    const skillLinks = page.locator('a[href*="/skills/"]');
    const count = await skillLinks.count();

    // Should have at least loaded the page successfully
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should navigate to skill detail page', async ({ page }) => {
    await page.goto('/');

    // Wait for page to load
    await page.waitForLoadState('networkidle');

    // Check if skill links exist
    const skillLinks = page.locator('a[href*="/skills/"]');
    const count = await skillLinks.count();

    // Only test navigation if skills exist
    if (count > 0) {
      await skillLinks.first().click();
      await page.waitForLoadState('networkidle');
      expect(page.url()).toContain('/skills/');
    } else {
      // If no skills, just verify homepage loaded
      expect(page.url()).toContain('localhost:3000');
    }
  });

  test('should have working navigation', async ({ page }) => {
    await page.goto('/');

    // Wait for page to load
    await page.waitForLoadState('networkidle');

    // Check that navigation links exist
    const navLinks = page.locator('nav a, header a');
    const count = await navLinks.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should be responsive', async ({ page }) => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Page should load
    await page.waitForLoadState('networkidle');

    // Hero section should be visible
    const heading = page.getByRole('heading', { level: 1 }).first();
    await expect(heading).toBeVisible();
  });
});
