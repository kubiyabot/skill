import { defineConfig } from 'vitepress'

// Catalog URL - configurable for dev vs prod
const CATALOG_URL = process.env.VITE_CATALOG_URL || 'https://catalog.skill-ai.dev'

export default defineConfig({
  title: 'Skill',
  description: 'Give your AI agent superpowers through the terminal. Universal skill runtime.',
  titleTemplate: ':title | Skill',

  base: '/',
  cleanUrls: true,

  themeConfig: {
    siteTitle: 'Skill',

    nav: [
      {
        text: 'Getting Started',
        items: [
          { text: 'What is Skill Engine?', link: '/getting-started/' },
          { text: 'Quick Start', link: '/getting-started/quick-start' },
          { text: 'Installation', link: '/getting-started/installation' }
        ]
      },
      { text: 'Tutorials', link: '/tutorials/', activeMatch: '/tutorials/' },
      { text: 'Guides', link: '/guides/', activeMatch: '/guides/' },
      { text: 'API', link: '/api/', activeMatch: '/api/' },
      { text: 'Examples', link: '/examples/', activeMatch: '/examples/' },
      { text: 'Catalog', link: CATALOG_URL, target: '_blank', rel: 'noopener' },
      {
        text: 'v1.0.0',
        items: [
          { text: 'Changelog', link: '/changelog' },
          { text: 'Contributing', link: '/contributing' },
          { text: 'FAQ', link: '/faq' }
        ]
      }
    ],

    sidebar: {
      '/getting-started/': [
        {
          text: 'Introduction',
          collapsed: false,
          items: [
            { text: 'What is Skill Engine?', link: '/getting-started/' },
            { text: 'Quick Start', link: '/getting-started/quick-start' },
            { text: 'Installation', link: '/getting-started/installation' }
          ]
        }
      ],

      '/tutorials/': [
        {
          text: 'Tutorials',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/tutorials/' },
            { text: 'Your First WASM Skill', link: '/tutorials/first-wasm-skill' },
            { text: 'Your First Native Skill', link: '/tutorials/first-native-skill' },
            { text: 'API Integration', link: '/tutorials/api-integration' },
            { text: 'Testing Skills', link: '/tutorials/testing-skills' }
          ]
        }
      ],

      '/guides/': [
        {
          text: 'Essentials',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/guides/' },
            { text: 'Developing Skills', link: '/guides/developing-skills' },
            { text: 'Skill Instances', link: '/guides/skill-instances' },
            { text: 'Manifest Configuration', link: '/guides/manifest' },
            { text: 'Environment Variables', link: '/guides/environment' },
            { text: 'Testing', link: '/guides/testing' }
          ]
        },
        {
          text: 'Features',
          collapsed: false,
          items: [
            { text: 'Web Interface', link: '/guides/web-interface' },
            { text: 'Semantic Search', link: '/guides/semantic-search' }
          ]
        },
        {
          text: 'Integration',
          collapsed: false,
          items: [
            { text: 'Claude Code', link: '/guides/claude-code' },
            { text: 'MCP Protocol', link: '/guides/mcp' }
          ]
        },
        {
          text: 'Advanced',
          collapsed: false,
          items: [
            { text: 'Claude Bridge', link: '/guides/advanced/claude-bridge' },
            { text: 'RAG Search Pipeline', link: '/guides/advanced/rag-search' },
            { text: 'Security Model', link: '/guides/advanced/security' }
          ]
        },
        {
          text: 'Operations',
          collapsed: false,
          items: [
            { text: 'CI/CD Pipeline', link: '/guides/ci-cd' },
            { text: 'Troubleshooting', link: '/guides/troubleshooting' }
          ]
        }
      ],

      '/api/': [
        {
          text: 'Reference',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/api/' },
            { text: 'CLI Commands', link: '/api/cli' },
            { text: 'REST API', link: '/api/rest' },
            { text: 'MCP Protocol', link: '/api/mcp' },
            { text: 'Rust Crates', link: '/api/rust' }
          ]
        }
      ],

      '/examples/': [
        {
          text: 'Skill Gallery',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/examples/' },
            { text: 'Azure', link: '/examples/azure' },
            { text: 'Claude Bridge Output', link: '/examples/claude-bridge' },
            { text: 'Datadog', link: '/examples/datadog' },
            { text: 'DigitalOcean', link: '/examples/digitalocean' },
            { text: 'Discord', link: '/examples/discord' },
            { text: 'GCP', link: '/examples/gcp' },
            { text: 'Grafana', link: '/examples/grafana' },
            { text: 'Kubernetes', link: '/examples/kubernetes' },
            { text: 'Linear', link: '/examples/linear' },
            { text: 'MongoDB', link: '/examples/mongodb' },
            { text: 'MySQL', link: '/examples/mysql' },
            { text: 'PagerDuty', link: '/examples/pagerduty' },
            { text: 'Terraform', link: '/examples/terraform' }
          ]
        }
      ],

      // Default sidebar for root pages
      '/': [
        {
          text: 'Getting Started',
          collapsed: false,
          items: [
            { text: 'What is Skill Engine?', link: '/getting-started/' },
            { text: 'Quick Start', link: '/getting-started/quick-start' },
            { text: 'Installation', link: '/getting-started/installation' }
          ]
        },
        {
          text: 'Tutorials',
          collapsed: false,
          items: [
            { text: 'Your First WASM Skill', link: '/tutorials/first-wasm-skill' },
            { text: 'Your First Native Skill', link: '/tutorials/first-native-skill' },
            { text: 'API Integration', link: '/tutorials/api-integration' },
            { text: 'Testing Skills', link: '/tutorials/testing-skills' }
          ]
        },
        {
          text: 'Guides',
          collapsed: false,
          items: [
            { text: 'Developing Skills', link: '/guides/developing-skills' },
            { text: 'Skill Instances', link: '/guides/skill-instances' },
            { text: 'Web Interface', link: '/guides/web-interface' },
            { text: 'Semantic Search', link: '/guides/semantic-search' },
            { text: 'Manifest', link: '/guides/manifest' },
            { text: 'Environment', link: '/guides/environment' },
            { text: 'Testing', link: '/guides/testing' },
            { text: 'CI/CD Pipeline', link: '/guides/ci-cd' },
            { text: 'Troubleshooting', link: '/guides/troubleshooting' },
            { text: 'Claude Code', link: '/guides/claude-code' },
            { text: 'MCP Protocol', link: '/guides/mcp' }
          ]
        },
        {
          text: 'API Reference',
          collapsed: false,
          items: [
            { text: 'CLI Commands', link: '/api/cli' },
            { text: 'REST API', link: '/api/rest' },
            { text: 'MCP Protocol', link: '/api/mcp' },
            { text: 'Rust Crates', link: '/api/rust' }
          ]
        },
        {
          text: 'Examples',
          collapsed: false,
          items: [
            { text: 'Azure', link: '/examples/azure' },
            { text: 'Claude Bridge', link: '/examples/claude-bridge' },
            { text: 'Datadog', link: '/examples/datadog' },
            { text: 'DigitalOcean', link: '/examples/digitalocean' },
            { text: 'Discord', link: '/examples/discord' },
            { text: 'GCP', link: '/examples/gcp' },
            { text: 'Grafana', link: '/examples/grafana' },
            { text: 'Kubernetes', link: '/examples/kubernetes' },
            { text: 'Linear', link: '/examples/linear' },
            { text: 'MongoDB', link: '/examples/mongodb' },
            { text: 'MySQL', link: '/examples/mysql' },
            { text: 'PagerDuty', link: '/examples/pagerduty' },
            { text: 'Terraform', link: '/examples/terraform' }
          ]
        },
        {
          text: 'Resources',
          collapsed: false,
          items: [
            { text: 'Changelog', link: '/changelog' },
            { text: 'Contributing', link: '/contributing' },
            { text: 'FAQ', link: '/faq' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/kubiyabot/skill' }
    ],

    search: {
      provider: 'local',
      options: {
        placeholder: 'Search docs...',
        translations: {
          button: {
            buttonText: 'Search',
            buttonAriaLabel: 'Search documentation'
          }
        }
      }
    },

    editLink: {
      pattern: 'https://github.com/kubiyabot/skill/edit/main/docs-site/:path',
      text: 'Edit this page on GitHub'
    },

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present Kubiya Inc.'
    },

    outline: {
      level: [2, 3],
      label: 'On this page'
    }
  },

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    },
    lineNumbers: false,
    codeTransformers: [
      {
        postprocess(code) {
          return code.replace(/\[!code (\w+)\]/g, '')
        }
      }
    ]
  },

  outDir: '.vitepress/dist',
  cacheDir: '.vitepress/cache',
  ignoreDeadLinks: true,

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/skill/logo.svg' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', { href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap', rel: 'stylesheet' }],
    ['meta', { name: 'theme-color', content: '#000000' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Skill Engine - Universal Runtime for AI Agent Skills' }],
    ['meta', { property: 'og:description', content: 'Secure, portable, and intelligent tool runtime for AI agents. Build once, run everywhere.' }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title', content: 'Skill Engine' }],
    ['meta', { name: 'twitter:description', content: 'Universal runtime for AI agent skills' }]
  ]
})
