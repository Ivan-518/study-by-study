<script setup lang="ts">
import { computed, ref } from 'vue'
import StatCard from '../components/StatCard.vue'
import type { LearningItem, LearningStatus } from '../stores/learning'

const props = defineProps<{ notes: string[]; learningItems: LearningItem[] }>()
const emit = defineEmits<{
  add: [note: string]
  remove: [index: number]
  navigate: [page: 'library' | 'assistant' | 'radar']
  'set-learning-status': [id: string, status: LearningStatus]
  'remove-learning': [id: string]
}>()

const note = ref('')
const activeTab = ref<'all' | LearningStatus>('all')
const statusCopy: Record<LearningStatus, string> = { todo: '待学习', doing: '学习中', done: '已完成' }
const filteredItems = computed(() => activeTab.value === 'all' ? props.learningItems : props.learningItems.filter((item) => item.status === activeTab.value))
const todoCount = computed(() => props.learningItems.filter((item) => item.status === 'todo').length)
const doingCount = computed(() => props.learningItems.filter((item) => item.status === 'doing').length)
const doneCount = computed(() => props.learningItems.filter((item) => item.status === 'done').length)

function save() { emit('add', note.value); note.value = '' }
function nextStatus(status: LearningStatus): LearningStatus { return status === 'todo' ? 'doing' : status === 'doing' ? 'done' : 'todo' }
function nextLabel(status: LearningStatus) { return status === 'todo' ? '开始学习' : status === 'doing' ? '标记完成' : '再次学习' }
function relativeDate(value: string) { const days = Math.max(0, Math.floor((Date.now() - new Date(value).getTime()) / 86400000)); return days === 0 ? '今天加入' : `${days} 天前加入` }
</script>

<template>
  <main class="page notes-page">
    <section class="page-heading grid-heading"><h1>学习清单与笔记</h1><p>将值得关注的 AI 洞察变成明确的学习行动，并保留自己的理解。</p></section>
    <div class="learning-overview panel"><div><i class="pi pi-flag" /><span><strong>本周学习焦点</strong><small>从热榜或探索页将内容加入清单，再按目标推进。</small></span></div><button type="button" class="primary-outline" @click="emit('navigate', 'radar')">浏览热榜 <i class="pi pi-arrow-right" /></button></div>
    <div class="stats-row three"><StatCard icon="pi pi-bookmark" label="待学习" :value="todoCount" /><StatCard icon="pi pi-play-circle" label="学习中" :value="doingCount" tone="blue" /><StatCard icon="pi pi-check-circle" label="已完成" :value="doneCount" tone="blue" /></div>
    <div class="tab-row learning-tabs"><button v-for="tab in ([['all', '全部'], ['todo', '待学习'], ['doing', '学习中'], ['done', '已完成']] as const)" :key="tab[0]" type="button" :class="{ active: activeTab === tab[0] }" @click="activeTab = tab[0]">{{ tab[1] }}<b v-if="tab[0] !== 'all'">{{ tab[0] === 'todo' ? todoCount : tab[0] === 'doing' ? doingCount : doneCount }}</b></button></div>
    <div class="notes-layout learning-layout">
      <aside class="panel note-filter"><h3>学习方法</h3><p>按目标学习</p><div class="goal-guide"><article><i class="pi pi-lightbulb" /><strong>了解概念</strong><small>先建立术语、问题和边界。</small></article><article><i class="pi pi-wrench" /><strong>动手试用</strong><small>跟着原始项目完成一次体验。</small></article><article><i class="pi pi-book" /><strong>深入学习</strong><small>形成笔记、复盘或可复用作品。</small></article></div><p>当前节奏</p><div class="learning-progress"><span><i :style="{ width: `${learningItems.length ? Math.round(doneCount / learningItems.length * 100) : 0}%` }" /></span><small>{{ learningItems.length ? `${Math.round(doneCount / learningItems.length * 100)}% 已完成` : '从任意洞察开始' }}</small></div></aside>
      <section class="panel learning-list"><header class="panel-title"><strong>{{ activeTab === 'all' ? '我的学习清单' : statusCopy[activeTab] }}</strong><span>{{ filteredItems.length }} 条</span></header>
        <div v-if="filteredItems.length" class="learning-list-content"><article v-for="item in filteredItems" :key="item.id" class="learning-item" :class="item.status"><div class="learning-item-icon"><i :class="item.status === 'done' ? 'pi pi-check' : item.goal === '动手试用' ? 'pi pi-wrench' : item.goal === '深入学习' ? 'pi pi-book' : 'pi pi-lightbulb'" /></div><div class="learning-item-copy"><span>{{ item.source }} · {{ item.goal }}</span><h3>{{ item.title }}</h3><p>{{ item.summary || '来自 AI 洞察的学习条目。' }}</p><div class="tag-list"><span v-for="topic in item.topics.slice(0, 3)" :key="topic">{{ topic }}</span></div><small>{{ relativeDate(item.createdAt) }} · {{ statusCopy[item.status] }}</small></div><div class="learning-item-actions"><button type="button" class="status-button" @click="emit('set-learning-status', item.id, nextStatus(item.status))">{{ nextLabel(item.status) }} <i class="pi pi-arrow-right" /></button><button type="button" class="remove-learning" :aria-label="`移除 ${item.title}`" @click="emit('remove-learning', item.id)"><i class="pi pi-times" /></button></div></article></div>
        <div v-else class="learning-empty"><i class="pi pi-bookmark" /><h3>还没有学习任务</h3><p>在“探索”或“热榜”打开一条洞察，选择学习目标后，它会自动出现在这里。</p><button type="button" class="primary-button" @click="emit('navigate', 'radar')">去发现内容 <i class="pi pi-arrow-right" /></button></div>
      </section>
      <aside class="notes-side"><section class="panel scratchpad"><h3>我的随手笔记</h3><article v-for="(item,index) in notes" :key="`${item}-${index}`"><h3>{{ item }}</h3><p>记录关键结论、实践中的疑问，或下次需要继续验证的想法。</p><div class="tag-list"><span>学习记录</span></div><small>刚刚更新</small><button type="button" @click="emit('remove', index)"><i class="pi pi-times" /></button></article><form @submit.prevent="save"><textarea v-model="note" placeholder="记下一段想法或一个还没弄懂的问题..." /><button class="primary-outline">保存笔记</button></form><button type="button" class="ai-note" @click="emit('navigate', 'assistant')"><i class="pi pi-sparkles" />AI 学习助手<br><small>把这条笔记变成知识卡</small></button></section></aside>
    </div>
  </main>
</template>
