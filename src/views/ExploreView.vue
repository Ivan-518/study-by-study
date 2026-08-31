<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { DiscoveryLearningItem, LearningGoal, LearningItem } from '../stores/learning'
import { lessonById, lessons } from '../data/learningContent'

import type { LessonAssessment } from '../stores/learning'
const props = defineProps<{ query: string; lessonProgress: Record<string, number>; lessonAssessments: Record<string, LessonAssessment>; currentLessonId: string; learningItems: LearningItem[] }>()
const emit = defineEmits<{ search: [query: string]; navigate: [page: 'path' | 'assistant' | 'radar']; 'open-lesson': [id: string]; 'add-learning': [item: DiscoveryLearningItem, goal: LearningGoal] }>()
const searchText = ref('')

type DiscoveryEvidence = {
  name: string
  kind: string
  url: string
}
type DiscoveryCard = {
  id: number
  title: string
  kind: 'project' | 'discussion' | 'paper' | 'release' | 'technology' | 'trend'
  summary: string
  primaryUrl: string
  topics: string[]
  publishedAt: string
  hotScore: number
  status: 'hot' | 'watch' | 'candidate'
  weeklyStars: number
  whyNow: string
  learningValue: string
  sourceCount: number
  sources: string[]
  evidence: DiscoveryEvidence[]
}
type DiscoveryPayload = { events: DiscoveryCard[]; refreshedAt: string; isStale: boolean; failedSources: number }

const liveItems = ref<DiscoveryCard[]>([])
const loadingFeed = ref(false)
const feedNotice = ref('')
const selectedInsight = ref<DiscoveryCard | null>(null)
const artStyles = ['eye', 'llama', 'next', 'orb']
const happeningItems = computed(() => {
  const isCommunitySignal = (item: DiscoveryCard) => item.sources.includes('Hacker News 讨论')
  const isIndependentProjectSignal = (item: DiscoveryCard) => item.sources.includes('GitHub 项目趋势') && item.weeklyStars === 0
  const isIndependentSignal = (item: DiscoveryCard) => isCommunitySignal(item) || isIndependentProjectSignal(item)

  return [...liveItems.value]
    .filter(isIndependentSignal)
    .sort((left, right) => {
      const sourcePriority = Number(isCommunitySignal(right)) - Number(isCommunitySignal(left))
      return sourcePriority || right.hotScore - left.hotScore
    })
    .slice(0, 3)
})
const signals = computed(() => {
  return happeningItems.value.map((item, index) => {
    const category = item.kind === 'discussion' ? '社区讨论' : '技术趋势'
    const topics = item.topics.slice(0, 2).join(' / ') || 'AI 技术'
    return [
      item.title,
      `${category} · ${topics}`,
      `${item.kind === 'discussion' ? '社区热度' : '项目信号'} ${item.hotScore} · ${relativeTime(item.publishedAt)}`,
      artStyles[index % artStyles.length],
    ]
  })
})
const continueLesson = computed(() => lessonById(props.currentLessonId)
  || lessons.find((lesson) => (props.lessonProgress[lesson.id] || 0) > 0 && (props.lessonProgress[lesson.id] || 0) < 100)
  || lessons.find((lesson) => (props.lessonProgress[lesson.id] || 0) < 100)
  || lessons[0])
const continueProgress = computed(() => props.lessonProgress[continueLesson.value?.id || ''] || 0)
const continueAssessment = computed(() => props.lessonAssessments[continueLesson.value?.id || ''])
const pendingLearning = computed(() => props.learningItems.filter((item) => item.status !== 'done').slice(0, 2))

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
    const payload = await invoke<DiscoveryPayload>('refresh_discoveries', { force })
    liveItems.value = payload.events
    const hotCount = payload.events.filter((item) => item.status === 'hot').length
    feedNotice.value = payload.isStale
      ? '网络暂不可用，正在展示上次成功发现的趋势'
      : `已确认 ${hotCount} 个热点，另有 ${payload.events.length - hotCount} 个新发现等待后续验证${payload.failedSources ? `，${payload.failedSources} 个来源暂不可用` : ''}`
  } catch (error) {
    feedNotice.value = typeof error === 'string' ? error : '暂时无法获取资讯，请稍后重试'
  } finally {
    loadingFeed.value = false
  }
}

function submit() { emit('search', searchText.value) }
function openInsight(insight: DiscoveryCard) { selectedInsight.value = insight }
function closeInsight() { selectedInsight.value = null }
async function openExternal(url: string) {
  if (!url) return
  if (!isTauri()) {
    window.open(url, '_blank', 'noopener,noreferrer')
    return
  }
  try {
    await invoke('open_external_url', { url })
  } catch (error) {
    feedNotice.value = typeof error === 'string' ? error : '暂时无法打开系统浏览器，请稍后重试。'
  }
}
function sourceLabel(insight: DiscoveryCard) {
  return insight.kind === 'project' || insight.kind === 'trend' ? '开源项目' : '社区讨论'
}
function hasPrimaryEvidence(insight: DiscoveryCard) {
  return insight.evidence.some((evidence) => evidence.url === insight.primaryUrl)
}
function addToLearning(insight: DiscoveryCard, goal: LearningGoal) {
  emit('add-learning', {
    title: insight.title,
    summary: insight.summary,
    url: insight.primaryUrl,
    topics: insight.topics,
    source: sourceLabel(insight),
  }, goal)
  feedNotice.value = `已加入学习清单：${goal}`
}
onMounted(() => { void refreshFeed() })
</script>

<template>
  <main class="page explore-page">
    <section class="page-heading grid-heading"><h1>早上好，Ivan</h1></section>
    <form class="global-search" @submit.prevent="submit"><i class="pi pi-search" /><input v-model="searchText" :placeholder="`搜索最新 AI、编程技术或问题...`" /><button>搜索</button></form>
    <div class="explore-grid">
      <section class="panel happening-panel"><header class="panel-title"><strong>正在发生</strong><button type="button" :disabled="loadingFeed" @click="refreshFeed(true)">{{ loadingFeed ? '更新中…' : '刷新' }} <i class="pi pi-refresh" /></button></header>
        <article v-for="([title, label, meta, art], index) in signals" :key="title" class="signal-row clickable" @click="openInsight(happeningItems[index])"><div class="tech-thumb" :class="art"><i class="pi pi-circle-fill" /></div><div><span>{{ label }}</span><h3>{{ title }}</h3><small>{{ meta }}</small></div><i class="pi pi-angle-right" /></article>
        <p v-if="!loadingFeed && !signals.length" class="feed-notice">正在等待社区与项目趋势信号同步；点击“刷新”可重新获取。</p>
        <small v-if="feedNotice" class="feed-notice">{{ feedNotice }}</small>
      </section>
      <section class="panel path-card continue-card"><header class="panel-title"><strong>继续学习 <em>本地保存</em></strong><button type="button" @click="emit('navigate', 'path')">全部课程 <i class="pi pi-arrow-right" /></button></header>
        <div v-if="continueLesson" class="path-top"><div><p class="continue-track">{{ continueLesson.track }} · {{ continueLesson.level }}</p><h2>{{ continueLesson.title }}</h2><p>{{ continueLesson.description }}</p><div class="meta-pills"><span><i class="pi pi-clock" />{{ continueLesson.duration }} 分钟</span><span><i class="pi pi-book" />{{ continueLesson.concepts.join(' / ') }}</span></div></div><div class="progress-ring" :style="{ background: `conic-gradient(#0966ed ${continueProgress}%, #e7edf7 0)` }"><b>{{ continueProgress }}%</b><small>本节进度</small></div></div>
        <div class="continue-summary"><div><i :class="continueAssessment?.level === 'mastered' ? 'pi pi-check-circle' : continueAssessment ? 'pi pi-refresh' : 'pi pi-play-circle'" /><span><strong>{{ continueAssessment?.level === 'mastered' ? '已通过理解检验，挑战下一节' : continueAssessment?.level === 'practice' ? '需要实践：继续完善练习' : continueAssessment?.level === 'review' ? '需要复习：根据反馈再试一次' : continueProgress ? '从上次阅读位置继续' : '开始第一节课程' }}</strong><small>阅读后提交理解检验，导师会决定下一步。</small></span></div><button type="button" class="primary-button" @click="emit('open-lesson', continueLesson.id)">{{ continueProgress ? '继续学习' : '开始学习' }} <i class="pi pi-arrow-right" /></button></div>
        <div v-if="pendingLearning.length" class="pending-mini"><strong>待推进的学习清单</strong><button v-for="item in pendingLearning" :key="item.id" type="button" @click="emit('navigate', 'path')"><span>{{ item.goal }}</span>{{ item.title }}<i class="pi pi-angle-right" /></button></div>
      </section>
    </div>
    <div v-if="selectedInsight" class="insight-backdrop" @click.self="closeInsight">
      <aside class="insight-drawer" aria-label="洞察详情">
        <header><div><span>{{ sourceLabel(selectedInsight) }} · {{ selectedInsight.topics.join(' / ') || 'AI 技术' }}</span><h2>{{ selectedInsight.title }}</h2></div><button type="button" aria-label="关闭洞察详情" @click="closeInsight"><i class="pi pi-times" /></button></header>
        <div class="insight-score"><strong>{{ selectedInsight.status === 'hot' ? '已确认热点' : selectedInsight.status === 'watch' ? '持续观察中' : '本周新发现' }}</strong><b>热度 {{ selectedInsight.hotScore }}</b></div>
        <section><h3>它是什么</h3><p>{{ selectedInsight.summary || '该条目来自公开 AI 技术信号，详情将在后续同步中补充。' }}</p></section>
        <section><h3>为什么现在值得关注</h3><p>{{ selectedInsight.whyNow }}</p></section>
        <section><h3>学习与实践价值</h3><p>{{ selectedInsight.learningValue }}</p></section>
        <section class="learning-actions"><h3>下一步怎么学</h3><p>选择一个目标，内容会加入“收藏”中的学习清单。</p><div><button type="button" @click="addToLearning(selectedInsight, '了解概念')"><i class="pi pi-lightbulb" />了解概念</button><button type="button" @click="addToLearning(selectedInsight, '动手试用')"><i class="pi pi-wrench" />动手试用</button><button type="button" @click="addToLearning(selectedInsight, '深入学习')"><i class="pi pi-book" />深入学习</button></div></section>
        <section><h3>证据与原始来源</h3><a v-for="evidence in selectedInsight.evidence" :key="evidence.url" :href="evidence.url" @click.prevent="openExternal(evidence.url)"><span>{{ evidence.name }}</span><i class="pi pi-external-link" /></a><a v-if="selectedInsight && !hasPrimaryEvidence(selectedInsight)" :href="selectedInsight.primaryUrl" @click.prevent="openExternal(selectedInsight.primaryUrl)"><span>原始项目 / 文章</span><i class="pi pi-external-link" /></a></section>
      </aside>
    </div>
  </main>
</template>
