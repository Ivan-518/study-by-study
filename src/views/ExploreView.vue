<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'

defineProps<{ query: string }>()
const emit = defineEmits<{ search: [query: string]; navigate: [page: 'path' | 'assistant'] }>()
const searchText = ref('')

type Happening = {
  title: string
  source: string
  url: string
  publishedAt: string
  summary: string
}
type HappeningsPayload = { items: Happening[]; refreshedAt: string; isStale: boolean; failedSources: number }

const fallbackSignals = [
  ['GPT-4o 原生图像理解的 8 个关键能力解析', '技术趋势', '2 小时前 · 1.2k 阅读', 'eye'],
  ['LlamaIndex 0.11 发布：更强大的 RAG 和 Agent 能力', '开源项目', '5 小时前 · 968 阅读', 'llama'],
  ['使用 Next.js 14 构建全栈 AI 网页应用', '教程', '昨天 · 2.3k 阅读', 'next'],
  ['微软发布 Phi-3-mini：小模型，大能量', '研究速递', '昨天 · 1.1k 阅读', 'orb'],
]
const liveItems = ref<Happening[]>([])
const loadingFeed = ref(false)
const feedNotice = ref('')
const expanded = ref(false)
const artStyles = ['eye', 'llama', 'next', 'orb']
const signals = computed(() => {
  const source = liveItems.value.length ? liveItems.value : fallbackSignals
  const limit = expanded.value ? 12 : 4
  return source.slice(0, limit).map((item, index) => {
    if (Array.isArray(item)) return item
    return [item.title, item.source, relativeTime(item.publishedAt), artStyles[index % artStyles.length]]
  })
})

function relativeTime(value: string) {
  const timestamp = new Date(value).getTime()
  if (Number.isNaN(timestamp)) return '刚刚更新'
  const minutes = Math.max(0, Math.floor((Date.now() - timestamp) / 60000))
  if (minutes < 2) return '刚刚更新'
  if (minutes < 60) return `${minutes} 分钟前`
  if (minutes < 24 * 60) return `${Math.floor(minutes / 60)} 小时前`
  if (minutes < 48 * 60) return '昨天'
  return `${Math.floor(minutes / (24 * 60))} 天前`
}

async function refreshFeed(force = false) {
  if (!isTauri()) {
    feedNotice.value = '桌面应用启动后将自动获取最新资讯'
    return
  }
  loadingFeed.value = true
  try {
    const payload = await invoke<HappeningsPayload>('refresh_happenings', { force })
    liveItems.value = payload.items
    feedNotice.value = payload.isStale
      ? '网络暂不可用，正在展示上次成功获取的内容'
      : `已更新 ${payload.items.length} 条资讯${payload.failedSources ? `，${payload.failedSources} 个来源暂不可用` : ''}`
  } catch (error) {
    feedNotice.value = typeof error === 'string' ? error : '暂时无法获取资讯，请稍后重试'
  } finally {
    loadingFeed.value = false
  }
}

function toggleMore() { expanded.value = !expanded.value }
function submit() { emit('search', searchText.value) }
onMounted(() => { void refreshFeed() })
</script>

<template>
  <main class="page explore-page">
    <section class="page-heading grid-heading"><h1>早上好，Ivan</h1></section>
    <form class="global-search" @submit.prevent="submit"><i class="pi pi-search" /><input v-model="searchText" :placeholder="`搜索最新 AI、编程技术或问题...`" /><button>搜索</button></form>
    <div class="explore-grid">
      <section class="panel happening-panel"><header class="panel-title"><strong>正在发生</strong><button type="button" :disabled="loadingFeed" @click="refreshFeed(true)">{{ loadingFeed ? '更新中…' : '刷新' }} <i class="pi pi-refresh" /></button></header>
        <article v-for="([title, label, meta, art]) in signals" :key="title" class="signal-row"><div class="tech-thumb" :class="art"><i class="pi pi-circle-fill" /></div><div><span>{{ label }}</span><h3>{{ title }}</h3><small>{{ meta }}</small></div></article>
        <small class="feed-notice">{{ feedNotice }}</small>
        <button class="link-button" type="button" @click="toggleMore">{{ expanded ? '收起趋势' : '查看更多趋势' }} <i :class="expanded ? 'pi pi-angle-up' : 'pi pi-angle-down'" /></button>
      </section>
      <section class="panel path-card"><header class="panel-title"><strong>推荐学习路径 <em>进阶</em></strong><span><i class="pi pi-bookmark" /> <i class="pi pi-ellipsis-h" /></span></header>
        <div class="path-top"><div><h2>构建 Agentic RAG 应用</h2><p>从零掌握 Agentic RAG 的核心原理与工程实践，构建可落地、可扩展的智能应用。</p><div class="meta-pills"><span><i class="pi pi-clock" />预计 12 小时</span><span><i class="pi pi-chart-bar" />中级</span><span><i class="pi pi-book" />4 个里程碑</span></div></div><div class="progress-ring"><b>64%</b><small>学习进度</small></div></div>
        <div class="milestones"><div><b>1</b><p><strong>理解 RAG 与 Agentic 模式</strong><small>概念、架构与核心流程</small></p><span>已完成 <i class="pi pi-check-circle" /></span></div><div><b>2</b><p><strong>构建知识检索系统</strong><small>文档处理、向量化与混合检索</small></p><span>已完成 <i class="pi pi-check-circle" /></span></div><div class="current"><b>3</b><p><strong>设计与实现 Agent</strong><small>工具使用、规划、记忆与执行</small></p><span>进行中 <i class="pi pi-spinner" /></span></div><div><b>4</b><p><strong>评估、部署与优化</strong><small>评估体系、可观测性与上线实践</small></p><span>未开始 <i class="pi pi-circle" /></span></div></div>
        <button class="link-button" @click="emit('navigate', 'path')">继续学习 <i class="pi pi-arrow-right" /></button>
      </section>
    </div>
  </main>
</template>
