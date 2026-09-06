---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "LLM Smart Router"
  text: "次世代智能 AI 路由引擎"
  tagline: 高性能 · 多协议统一 · 智能熔断 · 零外部依赖
  actions:
    - theme: brand
      text: 开发计划
      link: /development/plan
    - theme: alt
      text: 架构总览
      link: /architecture/overview
    - theme: alt
      text: GitHub
      link: https://github.com/Maicarons/llm-smart-routers

features:
  - title: 🚀 零外部依赖
    details: 所有数据层（缓存、状态、日志、配置）均在进程内管理，单二进制部署，即开即用。
  - title: 🔄 多协议统一
    details: 兼容 OpenAI Chat Completions、Anthropic Messages、Responses API 三大协议，自動转换。
  - title: 🧠 智能路由
    details: 任务感知 + 多维度评分 + 自适应学习，实时为用户选择最优模型。
  - title: ⚡ 智能熔断
    details: 五维渐进式熔断系统，预测性、多维度、渐进式流量控制，超越传统二值熔断。
  - title: 🔌 零内置提供商
    details: 用户完全掌控自己的 API 接口信息，系统只做路由和管理。
  - title: 🖥️ 后端优先
    details: 核心引擎是独立后台服务，WebUI 是分离的可选组件。
--- 
