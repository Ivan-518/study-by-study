<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { lessonById } from '../data/learningContent'
import type { LessonAssessment } from '../stores/learning'

type AssessmentResult = Omit<LessonAssessment, 'assessedAt'>
type AssistantConfigStatus = { configured: boolean }
const props = defineProps<{ lessonId: string; progress: number; assessment?: LessonAssessment }>()
const emit = defineEmits<{ back: []; navigate: [page: 'assistant']; 'update-progress': [id: string, progress: number]; 'save-assessment': [id: string, assessment: AssessmentResult]; 'clear-assessment': [id: string]; add: [note: string] }>()
const lesson = computed(() => lessonById(props.lessonId))
const note = ref('')
const activeLayer = ref<'overview' | 'code' | 'practice'>('overview')
const assessmentAnswer = ref('')
const followUpQuestion = ref('')
const assessing = ref(false)
const assessmentNotice = ref('')
const configured = ref(false)
const assessmentQuestion = computed(() => followUpQuestion.value || (lesson.value ? `请用自己的话解释：${lesson.value.keyPoints[0]}。再说明它在一个实际项目中会怎样影响你的设计选择。` : ''))

function markProgress(progress: number) { if (lesson.value) emit('update-progress', lesson.value.id, progress) }
function saveNote() { if (!note.value.trim()) return; emit('add', `${lesson.value?.title || '学习笔记'}：${note.value.trim()}`); note.value = '' }
async function loadConfig() {
  if (!isTauri()) return
  try { configured.value = (await invoke<AssistantConfigStatus>('get_assistant_config')).configured } catch { configured.value = false }
}
async function submitAssessment() {
  if (!lesson.value || !assessmentAnswer.value.trim() || assessing.value) return
  if (!configured.value) { assessmentNotice.value = '请先在设置中配置模型，才能进行个性化理解检验。'; return }
  assessing.value = true
  assessmentNotice.value = ''
  try {
    const contextText = `${lesson.value.overview}\n关键点：${lesson.value.keyPoints.join('；')}\n练习：${lesson.value.practice}`
    const result = await invoke<AssessmentResult>('assess_understanding', { input: { lessonTitle: lesson.value.title, contextText, question: assessmentQuestion.value, answer: assessmentAnswer.value } })
    emit('save-assessment', lesson.value.id, result)
  } catch (error) { assessmentNotice.value = typeof error === 'string' ? error : '理解检验暂时失败，请稍后再试。' } finally { assessing.value = false }
}
function levelCopy(level: LessonAssessment['level']) { return level === 'mastered' ? '已掌握' : level === 'practice' ? '需要实践' : '需要复习' }
function retryAssessment() {
  if (!lesson.value) return
  followUpQuestion.value = props.assessment?.followUpQuestion || ''
  assessmentAnswer.value = ''
  emit('clear-assessment', lesson.value.id)
}
onMounted(() => { void loadConfig() })
</script>

<template>
  <main v-if="lesson" class="page lesson-page">
    <button type="button" class="back-button" @click="emit('back')"><i class="pi pi-arrow-left" />返回课程目录</button>
    <section class="lesson-heading"><div><p class="eyebrow"><i :class="lesson.track === '应用开发' ? 'pi pi-sitemap' : 'pi pi-sparkles'" />{{ lesson.track }} · {{ lesson.level }}</p><h1>{{ lesson.title }}</h1><p>{{ lesson.description }}</p><div class="tag-list"><span v-for="concept in lesson.concepts" :key="concept">{{ concept }}</span></div></div><aside class="lesson-progress panel"><strong>本节进度 {{ progress }}%</strong><span><i :style="{ width: `${progress}%` }" /></span><small>{{ assessment ? `理解状态：${levelCopy(assessment.level)}` : '完成理解检验后才会标记掌握' }}</small></aside></section>
    <div class="lesson-layout"><section class="panel lesson-content"><nav class="lesson-tabs"><button type="button" :class="{ active: activeLayer === 'overview' }" @click="activeLayer = 'overview'">1. 核心理解</button><button type="button" :class="{ active: activeLayer === 'code' }" @click="activeLayer = 'code'">2. 最小代码</button><button type="button" :class="{ active: activeLayer === 'practice' }" @click="activeLayer = 'practice'">3. 检验与实践</button></nav>
        <article v-if="activeLayer === 'overview'" class="lesson-section"><h2>先建立正确的直觉</h2><p>{{ lesson.overview }}</p><h3>关键要点</h3><ol><li v-for="point in lesson.keyPoints" :key="point">{{ point }}</li></ol><section class="prerequisite-card"><i class="pi pi-sitemap" /><div><strong>需要的前置知识</strong><p>{{ lesson.prerequisites.join('、') }}。如果这里有不熟悉的概念，先在知识地图中补齐，再回到本节。</p></div></section><div class="lesson-action-row"><button type="button" class="primary-button" @click="activeLayer = 'code'; markProgress(Math.max(progress, 35))">查看最小代码 <i class="pi pi-arrow-right" /></button><button type="button" class="primary-outline" @click="emit('navigate', 'assistant')"><i class="pi pi-sparkles" />让导师换一种方式解释</button></div></article>
        <article v-else-if="activeLayer === 'code'" class="lesson-section"><h2>最小代码示例</h2><p>下面的代码只表达核心流程，真实项目仍需要错误处理、日志与评估。</p><pre><code>{{ lesson.code }}</code></pre><section class="engineering-card"><i class="pi pi-wrench" /><div><strong>连接到工程实践</strong><p>阅读代码时不要只看 API：请指出输入验证在哪里做、失败如何记录，以及怎样用一个小评估集验证它。</p></div></section><button type="button" class="primary-button" @click="activeLayer = 'practice'; markProgress(Math.max(progress, 70))">进入理解检验 <i class="pi pi-arrow-right" /></button></article>
        <article v-else class="lesson-section"><h2>先检验，再决定下一步</h2><p class="practice-copy">{{ lesson.practice }}</p><section v-if="assessment" class="assessment-result" :class="assessment.level"><header><span><i :class="assessment.level === 'mastered' ? 'pi pi-check-circle' : assessment.level === 'practice' ? 'pi pi-wrench' : 'pi pi-refresh'" />{{ levelCopy(assessment.level) }}</span><small>由 {{ assessment.model }} 于本机记录</small></header><h3>导师反馈</h3><p>{{ assessment.feedback }}</p><ul v-if="assessment.strengths.length"><li v-for="strength in assessment.strengths" :key="strength">{{ strength }}</li></ul><div v-if="assessment.gap" class="assessment-gap"><strong>当前卡点</strong><p>{{ assessment.gap }}</p></div><div class="assessment-next"><strong>下一步</strong><p>{{ assessment.nextStep }}</p><button type="button" class="primary-outline" @click="retryAssessment">{{ assessment.level === 'mastered' ? '挑战迁移题' : '回答追问' }} <i class="pi pi-arrow-right" /></button></div></section><section v-else class="assessment-form"><header><span><i class="pi pi-question-circle" />理解检验</span><small>不是背诵检查：请用自己的话连接概念与工程选择。</small></header><h3>{{ assessmentQuestion }}</h3><textarea v-model="assessmentAnswer" placeholder="写下你的理解。可以先说结论，再说明原因与一个例子…" /><p v-if="assessmentNotice" class="assessment-notice">{{ assessmentNotice }}</p><button type="button" class="primary-button" :disabled="assessing || !assessmentAnswer.trim()" @click="submitAssessment">{{ assessing ? '导师正在评估…' : '提交并获取反馈' }} <i class="pi pi-arrow-right" /></button></section><section class="resource-card"><span><i class="pi pi-link" />延伸原始资料</span><a :href="lesson.sourceUrl" target="_blank" rel="noreferrer">{{ lesson.sourceLabel }} <i class="pi pi-external-link" /></a></section></article>
      </section>
      <aside class="lesson-sidebar"><section class="panel lesson-outline"><h3>本节结构</h3><button type="button" :class="{ active: activeLayer === 'overview' }" @click="activeLayer = 'overview'"><i class="pi pi-lightbulb" />核心理解 <i v-if="progress >= 35" class="pi pi-check" /></button><button type="button" :class="{ active: activeLayer === 'code' }" @click="activeLayer = 'code'"><i class="pi pi-code" />最小代码 <i v-if="progress >= 70" class="pi pi-check" /></button><button type="button" :class="{ active: activeLayer === 'practice' }" @click="activeLayer = 'practice'"><i class="pi pi-flag" />检验与实践 <i v-if="assessment" class="pi pi-check" /></button></section><section class="panel lesson-note"><h3>本节笔记</h3><p>写下你理解的新概念、一个疑问，或将要验证的实践假设。</p><form @submit.prevent="saveNote"><textarea v-model="note" placeholder="例如：重排解决的是召回后的排序问题…" /><button class="primary-outline">保存到学习笔记</button></form></section></aside>
    </div>
  </main>
</template>
