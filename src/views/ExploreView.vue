<script setup lang="ts">
import { ref } from 'vue'

defineProps<{ query: string }>()
const emit = defineEmits<{ search: [query: string]; navigate: [page: 'path' | 'assistant'] }>()
const searchText = ref('')

const signals = [
  ['GPT-4o 原生图像理解的 8 个关键能力解析', '技术趋势', '2 小时前 · 1.2k 阅读', 'eye'],
  ['LlamaIndex 0.11 发布：更强大的 RAG 和 Agent 能力', '开源项目', '5 小时前 · 968 阅读', 'llama'],
  ['使用 Next.js 14 构建全栈 AI 网页应用', '教程', '昨天 · 2.3k 阅读', 'next'],
  ['微软发布 Phi-3-mini：小模型，大能量', '研究速递', '昨天 · 1.1k 阅读', 'orb'],
]
function submit() { emit('search', searchText.value) }
</script>

<template>
  <main class="page explore-page">
    <section class="page-heading grid-heading"><h1>早上好，Ivan</h1></section>
    <form class="global-search" @submit.prevent="submit"><i class="pi pi-search" /><input v-model="searchText" :placeholder="`搜索最新 AI、编程技术或问题...`" /><button>搜索</button></form>
    <div class="explore-grid">
      <section class="panel happening-panel"><header class="panel-title"><strong>正在发生</strong><button>查看全部 <i class="pi pi-angle-right" /></button></header>
        <article v-for="([title, label, meta, art]) in signals" :key="title" class="signal-row"><div class="tech-thumb" :class="art"><i class="pi pi-circle-fill" /></div><div><span>{{ label }}</span><h3>{{ title }}</h3><small>{{ meta }}</small></div></article>
        <button class="link-button">查看更多趋势 <i class="pi pi-angle-down" /></button>
      </section>
      <section class="panel path-card"><header class="panel-title"><strong>推荐学习路径 <em>进阶</em></strong><span><i class="pi pi-bookmark" /> <i class="pi pi-ellipsis-h" /></span></header>
        <div class="path-top"><div><h2>构建 Agentic RAG 应用</h2><p>从零掌握 Agentic RAG 的核心原理与工程实践，构建可落地、可扩展的智能应用。</p><div class="meta-pills"><span><i class="pi pi-clock" />预计 12 小时</span><span><i class="pi pi-chart-bar" />中级</span><span><i class="pi pi-book" />4 个里程碑</span></div></div><div class="progress-ring"><b>64%</b><small>学习进度</small></div></div>
        <div class="milestones"><div><b>1</b><p><strong>理解 RAG 与 Agentic 模式</strong><small>概念、架构与核心流程</small></p><span>已完成 <i class="pi pi-check-circle" /></span></div><div><b>2</b><p><strong>构建知识检索系统</strong><small>文档处理、向量化与混合检索</small></p><span>已完成 <i class="pi pi-check-circle" /></span></div><div class="current"><b>3</b><p><strong>设计与实现 Agent</strong><small>工具使用、规划、记忆与执行</small></p><span>进行中 <i class="pi pi-spinner" /></span></div><div><b>4</b><p><strong>评估、部署与优化</strong><small>评估体系、可观测性与上线实践</small></p><span>未开始 <i class="pi pi-circle" /></span></div></div>
        <button class="link-button" @click="emit('navigate', 'path')">继续学习 <i class="pi pi-arrow-right" /></button>
      </section>
    </div>
  </main>
</template>
