import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'LLM Smart Router',
  description: '次世代智能 AI 路由引擎 — 高性能、多协议统一、智能熔断',
  lang: 'zh-CN',
  lastUpdated: true,
  themeConfig: {
    logo: '/logo.svg',
    nav: [
      { text: '首页', link: '/' },
      { text: '开发计划', link: '/development/plan' },
      { text: '架构指南', link: '/architecture/overview' },
      { text: 'API 参考', link: '/api/reference' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: '指南',
          items: [
            { text: '快速开始', link: '/guide/getting-started' },
            { text: '配置文件', link: '/guide/configuration' },
            { text: '提供商管理', link: '/guide/provider-management' },
            { text: '路由策略', link: '/guide/routing-strategies' },
          ]
        }
      ],
      '/architecture/': [
        {
          text: '架构',
          items: [
            { text: '系统总览', link: '/architecture/overview' },
            { text: '协议转换引擎', link: '/architecture/protocol-conversion' },
            { text: '智能熔断系统', link: '/architecture/circuit-breaker' },
            { text: '路由策略引擎', link: '/architecture/routing-strategies' },
            { text: '数据存储层', link: '/architecture/data-storage' },
          ]
        }
      ],
      '/development/': [
        {
          text: '开发计划',
          items: [
            { text: '总览', link: '/development/plan' },
            { text: 'Phase 1: 核心引擎', link: '/development/phase-1-core' },
            { text: 'Phase 2: 智能路由', link: '/development/phase-2-smart-routing' },
            { text: 'Phase 3: 企业级', link: '/development/phase-3-enterprise' },
          ]
        }
      ],
      '/api/': [
        {
          text: 'API 参考',
          items: [
            { text: '接口总览', link: '/api/reference' },
            { text: 'Chat Completions', link: '/api/chat-completions' },
            { text: 'Messages API', link: '/api/messages' },
            { text: 'Responses API', link: '/api/responses' },
            { text: '管理接口', link: '/api/admin' },
          ]
        }
      ],
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/your-org/llm-smart-routers' }
    ],
    footer: {
      message: 'AGPL-3.0 License',
      copyright: 'Copyright © 2026'
    }
  }
})