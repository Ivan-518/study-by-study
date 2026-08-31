<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { DiscoveryLearningItem, LearningGoal } from '../stores/learning'

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

const items = ref<DiscoveryCard[]>([])
const loading = ref(false)
const notice = ref('')
const refreshedAt = ref('')
const selectedInsight = ref<DiscoveryCard | null>(null)
const githubPage = ref(0)
const newsPage = ref(0)
const projectsPage = ref(0)
const activeFeed = ref<'news' | 'projects'>('news')
const ITEMS_PER_PAGE = 3

const githubWeekly = computed(() => items.value
  .filter((item) => item.weeklyStars > 0 || item.sources.includes('GitHub 本周 Trending'))
  .sort((left, right) => right.weeklyStars - left.weeklyStars))
const dailyNews = computed(() => items.value
  .filter((item) => item.sources.includes('AI工具集 · 每日 AI 资讯')))
const latestProjects = computed(() => items.value
  .filter((item) => item.sources.includes('AI工具集 · 最新 AI 项目')))
const visibleGithubWeekly = computed(() => pageItems(githubWeekly.value, githubPage.value))
const visibleDailyNews = computed(() => pageItems(dailyNews.value, newsPage.value))
const visibleLatestProjects = computed(() => pageItems(latestProjects.value, projectsPage.value))
const activeFeedItems = computed(() => activeFeed.value === 'news' ? visibleDailyNews.value : visibleLatestProjects.value)
const activeFeedTotal = computed(() => activeFeed.value === 'news' ? dailyNews.value : latestProjects.value)
const activeFeedPage = computed(() => activeFeed.value === 'news' ? newsPage.value : projectsPage.value)
const emit = defineEmits<{ 'add-learning': [item: DiscoveryLearningItem, goal: LearningGoal] }>()

function relativeTime(value: string) {
  const timestamp = new Date(value).getTime()
  if (Number.isNaN(timestamp)) return '刚刚同步'
  const minutes = Math.max(0, Math.floor((Date.now() - timestamp) / 60000))
  if (minutes < 2) return '刚刚同步'
  if (minutes < 60) return `${minutes} 分钟前`
  if (minutes < 24 * 60) return `${Math.floor(minutes / 60)} 小时前`
  return `${Math.floor(minutes / (24 * 60))} 天前`
}

function pageCount(entries: DiscoveryCard[]) {
  return Math.max(1, Math.ceil(entries.length / ITEMS_PER_PAGE))
}

function pageItems(entries: DiscoveryCard[], page: number) {
  const start = page * ITEMS_PER_PAGE
  return entries.slice(start, start + ITEMS_PER_PAGE)
}

function changePage(target: 'github' | 'news' | 'projects', direction: number) {
  const entries = target === 'github' ? githubWeekly.value : target === 'news' ? dailyNews.value : latestProjects.value
  const current = target === 'github' ? githubPage : target === 'news' ? newsPage : projectsPage
  current.value = Math.min(Math.max(current.value + direction, 0), pageCount(entries) - 1)
}

async function refresh(force = false) {
  if (!isTauri()) {
    notice.value = '请在桌面应用中查看实时热榜。'
    return
  }
  loading.value = true
  try {
    const payload = await invoke<DiscoveryPayload>('refresh_discoveries', { force })
    items.value = payload.events
    refreshedAt.value = payload.refreshedAt
    githubPage.value = 0
    newsPage.value = 0
    projectsPage.value = 0
    notice.value = payload.isStale
      ? '网络暂不可用，正在展示上次成功同步的热榜。'
      : payload.failedSources ? `${payload.failedSources} 个来源暂时不可用，其余内容已更新。` : ''
  } catch (error) {
    notice.value = typeof error === 'string' ? error : '暂时无法更新热榜，请稍后重试。'
  } finally {
    loading.value = false
  }
}

async function openExternal(url: string) {
  if (!url) return
  if (!isTauri()) {
    window.open(url, '_blank', 'noopener,noreferrer')
    return
  }
  try {
    await invoke('open_external_url', { url })
  } catch (error) {
    notice.value = typeof error === 'string' ? error : '暂时无法打开系统浏览器，请稍后重试。'
  }
}

function sourceLabel(item: DiscoveryCard) {
  return item.kind === 'discussion' ? '社区讨论' : item.kind === 'technology' ? '技术资讯' : 'AI 项目'
}
function hasPrimaryEvidence(item: DiscoveryCard) {
  return item.evidence.some((evidence) => evidence.url === item.primaryUrl)
}
function addToLearning(item: DiscoveryCard, goal: LearningGoal) {
  emit('add-learning', {
    title: item.title,
    summary: item.summary,
    url: item.primaryUrl,
    topics: item.topics,
    source: item.weeklyStars > 0 ? 'GitHub 本周热门' : item.sources[0] || sourceLabel(item),
  }, goal)
  notice.value = `已加入学习清单：${goal}`
}

onMounted(() => { void refresh() })
</script>

<template>
  <main class="page radar-page">
    <section class="page-heading">
      <div><p class="eyebrow"><i class="pi pi-bolt" /> 趋势雷达</p><h1>热榜</h1><p>把公开热度、中文每日资讯与学习项目分开看，保留各自的来源和证据。</p></div>
      <button class="primary-button radar-refresh" type="button" :disabled="loading" @click="refresh(true)">{{ loading ? '同步中…' : '刷新热榜' }} <i class="pi pi-refresh" /></button>
    </section>

    <section class="radar-summary panel">
      <div><i class="pi pi-github" /><span><strong>GitHub 本周热门</strong><small>按近 7 天新增 star 排序</small></span><b>{{ githubWeekly.length }}</b></div>
      <div><i class="pi pi-calendar" /><span><strong>每日 AI 资讯</strong><small>AI工具集编辑收录的技术动态</small></span><b>{{ dailyNews.length }}</b></div>
      <div><i class="pi pi-box" /><span><strong>最新 AI 项目</strong><small>可进一步学习或尝试的项目</small></span><b>{{ latestProjects.length }}</b></div>
      <small class="radar-sync">{{ refreshedAt ? `最近同步：${relativeTime(refreshedAt)}` : '等待首次同步' }}</small>
    </section>

    <p v-if="notice" class="radar-notice">{{ notice }}</p>

    <section class="radar-section panel">
      <header class="radar-section-title"><div><span class="radar-icon github"><i class="pi pi-github" /></span><div><h2>GitHub 本周热门</h2><p>AI 相关仓库，按照 GitHub Trending 的近 7 天真实新增 star 排序。</p></div></div><span class="source-chip">GitHub Trending</span></header>
      <div v-if="githubWeekly.length" class="radar-list">
        <article v-for="(item, index) in visibleGithubWeekly" :key="item.id" class="radar-row clickable" @click="selectedInsight = item">
          <b class="ranking">{{ githubPage * ITEMS_PER_PAGE + index + 1 }}</b><div class="radar-row-copy"><span>{{ item.topics.slice(0, 2).join(' / ') || 'AI 技术' }}</span><h3>{{ item.title }}</h3><p>{{ item.summary || '来自 GitHub 本周 Trending 的 AI 项目。' }}</p></div><strong class="weekly-stars">+{{ item.weeklyStars.toLocaleString() }}<small>stars / 周</small></strong><i class="pi pi-angle-right" />
        </article>
        <footer v-if="pageCount(githubWeekly) > 1" class="radar-pagination"><span>第 {{ githubPage + 1 }} / {{ pageCount(githubWeekly) }} 组 · 共 {{ githubWeekly.length }} 个</span><div><button type="button" :disabled="githubPage === 0" @click="changePage('github', -1)"><i class="pi pi-angle-left" /></button><button type="button" :disabled="githubPage >= pageCount(githubWeekly) - 1" @click="changePage('github', 1)"><i class="pi pi-angle-right" /></button></div></footer>
      </div>
      <p v-else class="radar-empty">本周尚未同步到符合 AI 主题与质量门槛的 GitHub Trending 项目。点击“刷新热榜”后会重新抓取。</p>
    </section>

    <section class="radar-section radar-feed-panel panel">
      <header class="radar-section-title"><div><span class="radar-icon" :class="activeFeed === 'news' ? 'news' : 'projects'"><i :class="activeFeed === 'news' ? 'pi pi-calendar' : 'pi pi-box'" /></span><div><h2>{{ activeFeed === 'news' ? '每日 AI 资讯' : '最新 AI 项目' }}</h2><p>{{ activeFeed === 'news' ? '技术、模型、产品与行业动态。' : '工具、模型、框架和学习型项目。' }}</p></div></div><span class="source-chip">AI工具集</span></header>
      <div class="radar-tabs" role="tablist"><button type="button" :class="{ active: activeFeed === 'news' }" @click="activeFeed = 'news'"><i class="pi pi-calendar" /> 每日 AI 资讯 <b>{{ dailyNews.length }}</b></button><button type="button" :class="{ active: activeFeed === 'projects' }" @click="activeFeed = 'projects'"><i class="pi pi-box" /> 最新 AI 项目 <b>{{ latestProjects.length }}</b></button></div>
      <div v-if="activeFeedItems.length" class="radar-list compact">
        <article v-for="item in activeFeedItems" :key="item.id" class="radar-row clickable" @click="selectedInsight = item"><div class="radar-row-copy"><span>{{ item.topics.slice(0, 2).join(' / ') || 'AI 综合' }}</span><h3>{{ item.title }}</h3><p>{{ item.summary }}</p></div><i class="pi pi-angle-right" /></article>
        <footer v-if="pageCount(activeFeedTotal) > 1" class="radar-pagination"><span>第 {{ activeFeedPage + 1 }} / {{ pageCount(activeFeedTotal) }} 组 · 共 {{ activeFeedTotal.length }} {{ activeFeed === 'news' ? '条' : '个' }}</span><div><button type="button" :disabled="activeFeedPage === 0" @click="changePage(activeFeed, -1)"><i class="pi pi-angle-left" /></button><button type="button" :disabled="activeFeedPage >= pageCount(activeFeedTotal) - 1" @click="changePage(activeFeed, 1)"><i class="pi pi-angle-right" /></button></div></footer>
      </div>
      <p v-else class="radar-empty">{{ activeFeed === 'news' ? '每日资讯正在等待同步。它们会显示来源链接与简短摘要。' : '最新项目正在等待同步。学习资料、工具与开源项目都会保留。' }}</p>
    </section>

    <div v-if="selectedInsight" class="insight-backdrop" @click.self="selectedInsight = null">
      <aside class="insight-drawer" aria-label="洞察详情">
        <header><div><span>{{ sourceLabel(selectedInsight) }} · {{ selectedInsight.topics.join(' / ') || 'AI 技术' }}</span><h2>{{ selectedInsight.title }}</h2></div><button type="button" aria-label="关闭洞察详情" @click="selectedInsight = null"><i class="pi pi-times" /></button></header>
        <div class="insight-score"><strong>{{ selectedInsight.weeklyStars > 0 ? 'GitHub 本周热门' : selectedInsight.status === 'hot' ? '已确认热点' : '持续观察中' }}</strong><b>{{ selectedInsight.weeklyStars > 0 ? `+${selectedInsight.weeklyStars.toLocaleString()} stars / 周` : `热度 ${selectedInsight.hotScore}` }}</b></div>
        <section><h3>它是什么</h3><p>{{ selectedInsight.summary || '该条目来自公开 AI 技术信号。' }}</p></section>
        <section><h3>为什么现在值得关注</h3><p>{{ selectedInsight.whyNow }}</p></section>
        <section><h3>学习与实践价值</h3><p>{{ selectedInsight.learningValue }}</p></section>
        <section class="learning-actions"><h3>下一步怎么学</h3><p>选择目标后，会同步到“收藏”的学习清单。</p><div><button type="button" @click="addToLearning(selectedInsight, '了解概念')"><i class="pi pi-lightbulb" />了解概念</button><button type="button" @click="addToLearning(selectedInsight, '动手试用')"><i class="pi pi-wrench" />动手试用</button><button type="button" @click="addToLearning(selectedInsight, '深入学习')"><i class="pi pi-book" />深入学习</button></div></section>
        <section><h3>证据与原始来源</h3><a v-for="evidence in selectedInsight.evidence" :key="evidence.url" :href="evidence.url" @click.prevent="openExternal(evidence.url)"><span>{{ evidence.name }}</span><i class="pi pi-external-link" /></a><a v-if="!hasPrimaryEvidence(selectedInsight)" :href="selectedInsight.primaryUrl" @click.prevent="openExternal(selectedInsight.primaryUrl)"><span>原始项目 / 文章</span><i class="pi pi-external-link" /></a></section>
      </aside>
    </div>
  </main>
</template>
