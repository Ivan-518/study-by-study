<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { LearningItem } from '../stores/learning'
import { lessons } from '../data/learningContent'

type SearchResult = {
  id: string
  type: 'course' | 'learning' | 'note'
  title: string
  summary: string
  source: string
  topics: string[]
  meta: string
  lessonId?: string
}

const props = defineProps<{ query: string; notes: string[]; learningItems: LearningItem[]; lessonProgress: Record<string, number> }>()
const emit = defineEmits<{ navigate: [page: 'assistant' | 'notes']; search: [query: string]; 'open-lesson': [id: string] }>()
const searchText = ref(props.query)
const activeType = ref<'all' | SearchResult['type']>('all')
const selectedResult = ref<SearchResult | null>(null)
watch(() => props.query, (value) => { searchText.value = value })

const allResults = computed<SearchResult[]>(() => [
  ...lessons.map((lesson) => ({
    id: `course:${lesson.id}`, type: 'course' as const, title: lesson.title, summary: lesson.description,
    source: `${lesson.track} · 内置课程`, topics: lesson.concepts, meta: `${lesson.duration} 分钟 · ${props.lessonProgress[lesson.id] || 0}% 已学习`, lessonId: lesson.id,
  })),
  ...props.learningItems.map((item) => ({
    id: `learning:${item.id}`, type: 'learning' as const, title: item.title, summary: item.summary,
    source: `${item.source} · ${item.goal}`, topics: item.topics, meta: item.status === 'done' ? '已完成学习' : item.status === 'doing' ? '学习中' : '待学习',
  })),
  ...props.notes.map((note, index) => ({
    id: `note:${index}`, type: 'note' as const, title: note, summary: '你的学习笔记。可在“收藏”中继续编辑和整理。',
    source: '我的笔记', topics: ['学习记录'], meta: '本地笔记',
  })),
])
const normalizedQuery = computed(() => searchText.value.trim().toLowerCase())
const results = computed(() => allResults.value
  .filter((item) => activeType.value === 'all' || item.type === activeType.value)
  .filter((item) => {
    if (!normalizedQuery.value) return true
    const text = [item.title, item.summary, item.source, ...item.topics].join(' ').toLowerCase()
    return normalizedQuery.value.split(/\s+/).every((term) => text.includes(term))
  }))
const resultCount = computed(() => results.value.length)

function submit() { emit('search', searchText.value) }
function typeLabel(type: SearchResult['type']) { return type === 'course' ? '内置课程' : type === 'learning' ? '学习清单' : '我的笔记' }
function openResult(item: SearchResult) { selectedResult.value = item }
function primaryAction(item: SearchResult) {
  if (item.type === 'course' && item.lessonId) emit('open-lesson', item.lessonId)
  else emit('navigate', 'notes')
  selectedResult.value = null
}
</script>

<template>
  <main class="page search-page">
    <section class="page-heading grid-heading"><h1>本地学习搜索</h1><p>同时检索内置课程、你的学习清单与笔记；不需要联网，也不会把内容发到外部服务。</p></section>
    <form class="global-search compact-search" @submit.prevent="submit"><i class="pi pi-search" /><input v-model="searchText" placeholder="搜索 RAG、Attention、工具调用或自己的笔记…" /><button>搜索</button></form>
    <div class="search-layout local-search-layout"><aside class="panel filters"><h3>本地范围</h3><p>内容类型</p><div class="search-type-list"><button v-for="type in ([['all', '全部内容', 'pi pi-search'], ['course', '内置课程', 'pi pi-book'], ['learning', '学习清单', 'pi pi-bookmark'], ['note', '我的笔记', 'pi pi-pencil']] as const)" :key="type[0]" type="button" :class="{ active: activeType === type[0] }" @click="activeType = type[0]"><i :class="type[2]" />{{ type[1] }}<span>{{ type[0] === 'all' ? allResults.length : allResults.filter((item) => item.type === type[0]).length }}</span></button></div><p>搜索说明</p><div class="search-help"><i class="pi pi-shield" /><span>搜索只在本机完成。外部最新资讯仍在“热榜”中查看。</span></div></aside>
      <section class="panel results-panel"><header class="panel-title"><strong>匹配结果</strong><span>{{ resultCount }} 条</span></header><article v-for="item in results" :key="item.id" class="search-result local-search-result clickable" @click="openResult(item)"><div class="tech-thumb" :class="item.type === 'course' ? 'blocks' : item.type === 'learning' ? 'llama' : 'orb'"><i :class="item.type === 'course' ? 'pi pi-book' : item.type === 'learning' ? 'pi pi-bookmark' : 'pi pi-pencil'" /></div><div><span>{{ typeLabel(item.type) }} · {{ item.source }}</span><h3>{{ item.title }}</h3><p>{{ item.summary }}</p><div class="tag-list"><span v-for="topic in item.topics.slice(0, 3)" :key="topic">{{ topic }}</span></div><small>{{ item.meta }}</small></div><i class="pi pi-angle-right" /></article><div v-if="!results.length" class="search-empty"><i class="pi pi-search" /><h3>没有匹配的本地内容</h3><p>尝试更短的关键词，或到“热榜”发现新资料并加入学习清单。</p></div></section>
      <aside class="panel search-context"><h3>搜索后怎么继续</h3><article><i class="pi pi-book" /><div><strong>内置课程</strong><p>进入分层阅读，保存进度和本节笔记。</p></div></article><article><i class="pi pi-bookmark" /><div><strong>学习清单</strong><p>开始、完成或重新安排一项行动。</p></div></article><button type="button" class="primary-outline" @click="emit('navigate', 'assistant')"><i class="pi pi-sparkles" />配置 AI 导师后提问</button></aside>
    </div>
    <div v-if="selectedResult" class="insight-backdrop" @click.self="selectedResult = null"><aside class="insight-drawer learning-card"><header><div><span>{{ typeLabel(selectedResult.type) }} · {{ selectedResult.source }}</span><h2>{{ selectedResult.title }}</h2></div><button type="button" aria-label="关闭资料详情" @click="selectedResult = null"><i class="pi pi-times" /></button></header><section><h3>快速理解</h3><p>{{ selectedResult.summary }}</p></section><section><h3>相关概念</h3><div class="tag-list"><span v-for="topic in selectedResult.topics" :key="topic">{{ topic }}</span></div></section><section><h3>下一步</h3><p>{{ selectedResult.type === 'course' ? '进入课程后可按“概念 → 代码 → 练习”分层完成学习。' : '打开学习清单，继续推进状态或补充自己的笔记。' }}</p><button type="button" class="primary-button" @click="primaryAction(selectedResult)">{{ selectedResult.type === 'course' ? '开始本节学习' : '打开学习清单' }} <i class="pi pi-arrow-right" /></button></section></aside></div>
  </main>
</template>
