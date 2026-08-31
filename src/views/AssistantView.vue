<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { lessonById, lessons } from '../data/learningContent'

type AssistantConfigStatus = { configured: boolean; baseUrl: string; model: string }
type AssistantAnswer = { content: string; model: string }
type Message = { role: 'user' | 'assistant'; content: string }
const props = defineProps<{ currentLessonId: string; lessonProgress: Record<string, number> }>()
const emit = defineEmits<{ navigate: [page: 'path' | 'settings']; 'open-lesson': [id: string] }>()
const configured = ref(false)
const model = ref('')
const loading = ref(false)
const notice = ref('')
const prompt = ref('')
const messages = ref<Message[]>([])
const activeLesson = computed(() => lessonById(props.currentLessonId) || lessons.find((lesson) => (props.lessonProgress[lesson.id] || 0) > 0) || lessons[0])

async function loadConfig() {
  if (!isTauri()) return
  try {
    const config = await invoke<AssistantConfigStatus>('get_assistant_config')
    configured.value = config.configured
    model.value = config.model
  } catch { configured.value = false }
}
async function ask(question: string, mode = 'explain') {
  const value = question.trim()
  if (!value || !activeLesson.value || loading.value) return
  if (!configured.value) { notice.value = '请先在设置中配置模型服务。'; return }
  messages.value.push({ role: 'user', content: value })
  prompt.value = ''
  loading.value = true
  notice.value = ''
  try {
    const lesson = activeLesson.value
    const contextText = `${lesson.overview}\n关键点：${lesson.keyPoints.join('；')}\n练习：${lesson.practice}`
    const answer = await invoke<AssistantAnswer>('ask_assistant', { input: { question: value, contextTitle: lesson.title, contextText, mode } })
    messages.value.push({ role: 'assistant', content: answer.content })
    model.value = answer.model
  } catch (error) {
    notice.value = typeof error === 'string' ? error : '导师暂时无法回答，请稍后再试。'
  } finally { loading.value = false }
}
onMounted(() => { void loadConfig() })
</script>

<template>
  <main class="page assistant-page"><section class="page-heading grid-heading"><h1><i class="pi pi-sparkles" /> AI 学习导师</h1><p>围绕当前课程解释概念、检验理解，并把问题落到下一步实践。</p></section><div class="context-bar panel"><i class="pi pi-book" />当前上下文：{{ activeLesson?.title }} · {{ activeLesson?.track }}<button type="button" @click="emit('open-lesson', activeLesson.id)">打开课程 <i class="pi pi-arrow-right" /></button></div>
    <section v-if="!configured" class="assistant-setup panel"><div class="assistant-setup-icon"><i class="pi pi-lock" /></div><div><p class="eyebrow">模型尚未配置</p><h2>连接模型后，导师才能根据你的问题教学</h2><p>API Key 仅保存到 Windows 凭据管理器，前端和项目文件不会读取它。每次提问只发送当前课程的必要上下文。</p><div class="assistant-setup-actions"><button type="button" class="primary-button" @click="emit('navigate', 'settings')">去配置模型 <i class="pi pi-arrow-right" /></button><button type="button" class="primary-outline" @click="emit('navigate', 'path')">先学习课程</button></div></div></section>
    <div v-else class="assistant-live-layout"><aside class="panel assistant-context"><h3>当前学习上下文</h3><p>本次发送给导师</p><button type="button" @click="emit('open-lesson', activeLesson.id)"><i class="pi pi-file" />{{ activeLesson.title }}<i class="pi pi-angle-right" /></button><p>快速动作</p><button type="button" @click="ask('请用更直观的方式解释本节的核心概念。')"><i class="pi pi-lightbulb" />用直觉解释<i class="pi pi-angle-right" /></button><button type="button" @click="ask('请给我一个贴近工程实践的最小例子。')"><i class="pi pi-code" />给一个工程例子<i class="pi pi-angle-right" /></button><button type="button" @click="ask('请出一道能检验我是否真正理解本节的小题。', 'quiz')"><i class="pi pi-question-circle" />出一道检验题<i class="pi pi-angle-right" /></button></aside><section class="panel live-chat-area"><div v-if="!messages.length" class="chat-welcome"><span class="n-avatar">N</span><h2>从一个真正的困惑开始</h2><p>例如：{{ activeLesson.keyPoints[0] }}</p></div><article v-for="(message, index) in messages" :key="`${message.role}-${index}`" class="live-message" :class="message.role"><span v-if="message.role === 'assistant'" class="n-avatar">N</span><div><strong>{{ message.role === 'user' ? '你' : `导师 · ${model}` }}</strong><p>{{ message.content }}</p></div></article><div v-if="loading" class="live-message assistant"><span class="n-avatar">N</span><div><strong>导师正在思考</strong><p><i class="pi pi-spinner pi-spin" /> 正在结合当前课程组织解释…</p></div></div><p v-if="notice" class="assistant-notice">{{ notice }}</p><form class="live-chat-input" @submit.prevent="ask(prompt)"><textarea v-model="prompt" :disabled="loading" placeholder="说说你卡在哪里，或粘贴你的理解让我帮你检查…" /><button class="primary-button" :disabled="loading || !prompt.trim()">发送 <i class="pi pi-send" /></button></form></section><aside class="assistant-aid"><section class="panel"><h3>导师原则</h3><button><i class="pi pi-shield" />不会假装引用未提供的资料</button><button><i class="pi pi-sitemap" />先连接概念，再给实践建议</button><button><i class="pi pi-eye" />不确定时会明确说明边界</button></section></aside></div>
  </main>
</template>
