# Agentic Skills Marketplace

A production-ready web application for browsing and discovering self-contained agentic skills. Built with Next.js 14, TypeScript, and Tailwind CSS.

## 🌟 Features

- **Clean, Professional UI** - Inspired by mcpservers.org aesthetic
- **Static Site Generation** - Lightning-fast page loads (<1s FCP)
- **3 Skill Types** - WASM, Native, and Docker runtime skills
- **4 Installation Methods** - TOML, CLI, MCP, and GitHub instructions
- **Fully Responsive** - 1-4 column grid adapts to all screen sizes
- **Type-Safe** - Full TypeScript coverage
- **SEO Optimized** - Meta tags, OpenGraph, structured data
- **Related Skills** - Automatic recommendations
- **Validated** - Pre-build manifest validation

## 🚀 Quick Start

### Development

```bash
cd marketplace-web
npm install
npm run dev
# Open http://localhost:3000
```

### Production Build

```bash
npm run build
# Validates manifests → Builds static site
npm start
```

### Deploy to Vercel

```bash
# Connect your GitHub repo to Vercel
# It will auto-detect Next.js and deploy!
```

Or use the Vercel CLI:

```bash
npm install -g vercel
vercel
```

## 📁 Project Structure

```
marketplace-web/
├── app/                        # Next.js 14 App Router
│   ├── layout.tsx             # Root layout with metadata
│   ├── page.tsx               # Homepage (skills grid)
│   ├── skills/[id]/page.tsx   # Dynamic skill detail pages (SSG)
│   └── globals.css            # Tailwind + custom styles
│
├── components/                # React components
│   ├── ui/                    # Badge, Card primitives
│   ├── layout/                # Header, Footer, Container
│   ├── home/                  # SkillCard, SkillGrid, HeroSection
│   └── detail/                # (Future: InstallationTabs, etc.)
│
├── lib/                       # Core logic
│   ├── data/
│   │   └── loadSkills.ts      # Parse JSON manifests (build-time)
│   ├── generators/            # Generate installation snippets
│   │   ├── tomlGenerator.ts   # .skill-engine.toml
│   │   ├── cliGenerator.ts    # CLI commands
│   │   ├── mcpGenerator.ts    # .mcp.json
│   │   └── readmeGenerator.ts # GitHub instructions
│   ├── search/                # (Future: Fuse.js search)
│   ├── utils/
│   │   ├── cn.ts              # Class name merger
│   │   └── constants.ts       # Site constants
│   └── types.ts               # TypeScript types
│
├── public/                    # Static assets
├── next.config.js            # Next.js config (SSG)
├── tailwind.config.js        # Design tokens
├── vercel.json               # Deployment config
└── package.json
```

## 🎨 Design System

### Colors

- **Primary:** `#2563eb` (blue)
- **Gray Scale:** `#f9fafb` → `#111827`
- **Borders:** `#e5e7eb`

### Typography

- **Font:** Inter (system fallback)
- **Sizes:** `text-xs` → `text-4xl`
- **Weights:** `font-medium`, `font-semibold`, `font-bold`

### Components

- **Cards:** `rounded-xl`, `border-gray-200`, `hover:shadow-lg`
- **Spacing:** 6-unit system (24px)
- **Grid:** 1-4 columns responsive

## 📝 Adding New Skills

1. Create a JSON manifest in `../marketplace/skills/`:

```json
{
  "id": "my-skill",
  "name": "My Skill",
  "type": "native",
  "description": "Self-contained skill for...",
  "version": "1.0.0",
  "author": { "name": "Your Name" },
  "categories": ["development"],
  "installation": {
    "source": "./examples/native-skills/my-skill"
  },
  "tools": [...],
  "examples": [...]
}
```

2. Validate:

```bash
npm run validate
```

3. Rebuild:

```bash
npm run build
```

## 🔍 Tech Stack

- **Framework:** Next.js 14 (App Router)
- **Language:** TypeScript
- **Styling:** Tailwind CSS
- **Build:** Static Site Generation (SSG)
- **Deployment:** Vercel
- **Validation:** Custom JSON schema validator

## 📊 Performance

- **Bundle Size:** 96.2 kB first load
- **Build Time:** ~10-15 seconds
- **Pages Generated:** Homepage + detail pages for each skill (SSG)
- **Lighthouse Score:** >90 (target)

## 🛠️ Development

### Available Scripts

```bash
npm run dev        # Start dev server
npm run build      # Production build (with validation)
npm run start      # Start production server
npm run lint       # Lint code
npm run validate   # Validate skill manifests
```

### Environment Variables

Create `.env.local`:

```bash
NEXT_PUBLIC_SITE_URL=https://marketplace.skill.dev
NEXT_PUBLIC_GITHUB_REPO=https://github.com/kubiyabot/skill
```

## 🚢 Deployment

### Vercel (Recommended)

1. Push to GitHub
2. Connect repo to Vercel
3. Auto-deploy on push to `main`

### Manual Static Export

```bash
npm run build
# Files in .next/ are ready for static hosting
```

## 📄 License

MIT License - see LICENSE file

## 🤝 Contributing

1. Fork the repo
2. Create a feature branch
3. Add your skill manifest to `marketplace/skills/`
4. Submit a PR

See `../marketplace/README.md` for skill manifest guidelines.

---

Built with ❤️ by the Skill Engine Team
