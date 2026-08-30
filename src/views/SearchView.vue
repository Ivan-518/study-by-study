<script setup lang="ts">
import { ref } from 'vue'
import AssistantPanel from '../components/AssistantPanel.vue'

const props = defineProps<{ query: string; selected: string[] }>()
const emit = defineEmits<{ navigate: [page: 'path' | 'assistant']; search: [query: string] }>()
const searchText = ref(props.query)
const results = [
  ['官方文档', 'Agentic RAG 最佳实践指南', '系统阐述 Agentic RAG 的核心概念、关键组件、实践模式与落地要点，并提供详细的架构建议与案例参考。', '2 天前更新 · 10 分钟阅读', 'orb'],
  ['GitHub', 'agentic-rag-best-practices', '开源项目：Agentic RAG 最佳实践示例代码，包含多步推理、工具调用与评估脚本，支持快速开始和二次开发。', '5 天前更新 · 8 分钟阅读', 'llama'],
  ['论文', '面向复杂任务的 Agentic RAG：设计、挑战与评估', '提出 Agentic RAG 的统一框架，分析关键挑战与在多个任务上的实际效果，为设计智能应用提供参考。', '1 周前更新 · 15 分钟阅读', 'paper'],
  ['技术博客', '从 0 到 1 搭建企业级 Agentic RAG 系统', '结合实际项目经验，分享完整的系统架构、检索增强与优化策略，帮助构建可靠的 Agentic RAG 应用。', '1 周前更新 · 12 分钟阅读', 'blocks'],
]
function submit() { emit('search', searchText.value) }
</script>

<template>
  <main class="page search-page"><section class="page-heading grid-heading"><h1>搜索结果</h1></section><form class="global-search compact-search" @submit.prevent="submit"><i class="pi pi-search" /><input v-model="searchText" /><button>搜索</button></form>
    <div class="search-layout"><aside class="panel filters"><h3>筛选与来源</h3><p>来源</p><div class="filter-sources"><button><i class="pi pi-book" />官方文档</button><button><i class="pi pi-github" />GitHub</button><button><i class="pi pi-file" />论文</button><button><i class="pi pi-file-edit" />技术博客</button></div><p>相关度 <i class="pi pi-question-circle" /></p><input type="range" value="85" /><div class="range-label"><span>较低</span><span>较高</span></div><p>主题筛选</p><button v-for="label in ['架构设计  12', '检索增强  18', '工具调用  14', '评估与优化  9', '工程实践  16']" :key="label" class="topic-filter">{{ label }}</button><button class="link-button">展开更多 <i class="pi pi-angle-down" /></button></aside>
      <section class="panel results-panel"><header class="panel-title"><strong>关于 {{ query }} 的最新资料</strong><button>最新 <i class="pi pi-angle-down" /></button></header><article v-for="([source, title, description, meta, art], index) in results" :key="title" class="search-result" :class="{ selected: index === 0 }"><div class="tech-thumb" :class="art"><i class="pi pi-circle-fill" /></div><div><span>{{ source }}</span><h3>{{ title }}</h3><p>{{ description }}</p><small>{{ meta }}</small></div><button class="bookmark"><i class="pi" :class="index === 0 ? 'pi-bookmark-fill' : 'pi-bookmark'" /></button></article><div class="pagination"><i class="pi pi-angle-left" /><b>1</b><span>2</span><span>3</span><span>4</span><span>5</span><i class="pi pi-angle-right" /></div></section>
      <AssistantPanel @open="emit('navigate', 'assistant')" @ask="emit('navigate', 'assistant')" />
    </div>
    <footer class="selected-bar panel"><strong>已选资料（{{ selected.length }}）</strong><button>清空</button><div class="selected-sources"><span v-for="source in selected" :key="source"><i class="tech-dot" />{{ source }} <i class="pi pi-times" /></span></div><button class="primary-outline" @click="emit('navigate', 'path')"><i class="pi pi-play-circle" />开始学习</button></footer>
  </main>
</template>
