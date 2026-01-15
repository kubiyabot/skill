# Skill Engine Documentation Site

Production-ready documentation site for Skill Engine built with VitePress.

## Features

- ✅ **26 Pages** of comprehensive documentation
- ✅ **External Catalog Integration** linking to marketplace site
- ✅ **Clean Professional Design** with custom theme
- ✅ **Fast Build** (~3s) with static site generation
- ✅ **SEO Optimized** with proper meta tags
- ✅ **Mobile Responsive** design
- ✅ **Configurable URLs** for dev/prod environments

## Architecture

This documentation site works alongside the separate **Skill Catalog** (marketplace-web):

- **Docs Site** (this): Technical documentation, guides, API references
- **Catalog Site**: Interactive skill marketplace with search and filtering
- **Unified Experience**: Matching design, seamless navigation between sites

## Quick Start

### Development (Recommended - Run Both Sites)

From the project root:

```bash
make dev
# Opens:
# - Docs:    http://localhost:5173/skill/
# - Catalog: http://localhost:3000
```

The "Catalog" link in the docs nav will automatically point to `localhost:3000` during development.

### Development (Docs Only)

```bash
npm install
npm run dev
# Open http://localhost:5173/skill/
```

### Production Build

```bash
npm run build
# Output in .vitepress/dist/
```

### Preview Production Build

```bash
npm run preview
```

## Configuration

### Environment Variables

Create a `.env` file (see `.env.example`):

```bash
# Catalog URL (required)
VITE_CATALOG_URL=http://localhost:3000  # Development
# VITE_CATALOG_URL=https://catalog.skill-ai.dev  # Production

# Optional
VITE_SITE_URL=https://docs.skill-ai.dev
VITE_GITHUB_REPO=https://github.com/kubiyabot/skill
```

The catalog URL determines where the "Catalog" navigation link points:
- **Dev**: Set to `http://localhost:3000` (done automatically by `make dev`)
- **Prod**: Defaults to `https://catalog.skill-ai.dev` if not set

## Deployment

### Deploy to Vercel

Both docs and catalog sites can be deployed separately:

**Docs Site**:
```bash
cd docs-site
vercel --prod
# Set VITE_CATALOG_URL=https://catalog.skill-ai.dev
```

**Catalog Site**:
```bash
cd marketplace-web
vercel --prod
```

### GitHub Actions (Recommended)

Configure two separate deployments:
- `docs.skill-ai.dev` → docs-site/
- `catalog.skill-ai.dev` → marketplace-web/

Both sites will have matching design and seamless navigation.

## Project Structure

```
docs-site/
├── .vitepress/
│   ├── config.ts              # VitePress configuration
│   ├── theme/
│   │   ├── index.ts          # Custom theme
│   │   └── custom.css        # Custom styles
│   └── dist/                 # Build output
├── getting-started/           # Getting started guides
├── guides/                    # User guides
│   ├── advanced/             # Advanced topics
│   ├── manifest.md
│   ├── environment.md
│   └── testing.md
├── api/                       # API references
│   ├── cli.md
│   ├── rest.md
│   ├── mcp.md
│   └── rust.md
├── examples/                  # Example skills
│   ├── kubernetes.md
│   ├── terraform.md
│   └── claude-bridge.md
├── catalog.md                 # Skill catalog
├── changelog.md               # Version history
├── contributing.md            # Contribution guide
├── index.md                   # Home page
├── package.json
└── vercel.json               # Vercel deployment config
```

## Documentation Coverage

### Completed (100%)

- ✅ Getting Started (3 pages)
  - Installation
  - Quick Start
  - What is Skill Engine

- ✅ Core Guides (6 pages)
  - Skill Development
  - Manifest Configuration
  - Environment Variables
  - Testing
  - Claude Code Integration
  - MCP Protocol

- ✅ Advanced Guides (3 pages)
  - Claude Bridge
  - RAG Search Pipeline
  - Security Model

- ✅ API References (5 pages)
  - CLI Commands
  - REST API
  - MCP Protocol
  - Rust Crates
  - Overview

- ✅ Examples (4 pages)
  - Kubernetes
  - Terraform
  - Claude Bridge Output
  - Overview

- ✅ Meta (2 pages)
  - Changelog
  - Contributing

**Total**: 26 pages (Catalog is separate site)

## Environment Variables

Optional environment variables for deployment:

```bash
# Site URL (for meta tags)
VITE_SITE_URL=https://skill-engine-docs.vercel.app

# GitHub repo
VITE_GITHUB_REPO=https://github.com/kubiyabot/skill
```

## Custom Domain (Optional)

After deploying to Vercel:

1. Go to your project settings
2. Navigate to "Domains"
3. Add your custom domain (e.g., `docs.skill.dev`)
4. Follow Vercel's DNS configuration instructions

## Performance

- **Build Time**: ~3 seconds
- **Bundle Size**: Optimized with VitePress
- **First Load**: <1s FCP
- **SEO Score**: 90+
- **Mobile Responsive**: Yes

## Tech Stack

- **Framework**: VitePress 1.0+
- **Language**: TypeScript
- **Styling**: Custom CSS with design tokens
- **Build**: Static Site Generation
- **Deployment**: Vercel
- **Fonts**: Inter (body), JetBrains Mono (code)

## Maintenance

### Adding New Pages

1. Create markdown file in appropriate directory
2. Add to `.vitepress/config.ts` sidebar configuration
3. Build and test locally
4. Deploy

### Updating Catalog

1. Edit `catalog.md`
2. Add skill cards with metadata
3. Include installation and usage examples

### Theme Customization

Edit `.vitepress/theme/custom.css` to customize:
- Colors
- Typography
- Spacing
- Components

## Troubleshooting

### Build Fails

```bash
# Clear cache
rm -rf node_modules .vitepress/cache
npm install
npm run build
```

### Links Not Working

- Ensure all internal links use relative paths
- Check `.vitepress/config.ts` for correct base path
- Verify files exist in correct directories

### Styles Not Loading

- Check `.vitepress/theme/custom.css` is imported in `index.ts`
- Verify CSS custom properties are defined
- Clear browser cache

## Support

- **Issues**: [GitHub Issues](https://github.com/kubiyabot/skill/issues)
- **Discussions**: [GitHub Discussions](https://github.com/kubiyabot/skill/discussions)

## License

MIT License - see LICENSE file in repository root

---

Built with ❤️ by the Skill Engine Team
