<script setup lang="ts">
import { computed, ref } from 'vue'
import { lessonById } from '../data/learningContent'

const props = defineProps<{ lessonId: string; progress: number }>()
const emit = defineEmits<{ back: []; navigate: [page: 'assistant']; 'update-progress': [id: string, progress: number]; add: [note: string] }>()
const lesson = computed(() => lessonById(props.lessonId))
const note = ref('')
const activeLayer = ref<'overview' | 'code' | 'practice'>('overview')

function markProgress(progress: number) {
  if (lesson.value) emit('update-progress', lesson.value.id, progress)
}
function saveNote() {
  if (!note.value.trim()) return
  emit('add', `${lesson.value?.title || '学习笔记'}：${note.value.trim()}`)
  note.value = ''
}
</script>

<template>
  <main v-if="lesson" class="page lesson-page">
    <button type="button" class="back-button" @click="emit('back')"><i class="pi pi-arrow-left" />返回课程目录</button>
    <section class="lesson-heading"><div><p class="eyebrow"><i :class="lesson.track === '应用开发' ? 'pi pi-sitemap' : 'pi pi-sparkles'" />{{ lesson.track }} · {{ lesson.level }}</p><h1>{{ lesson.title }}</h1><p>{{ lesson.description }}</p><div class="tag-list"><span v-for="concept in lesson.concepts" :key="concept">{{ concept }}</span></div></div><aside class="lesson-progress panel"><strong>本节进度 {{ progress }}%</strong><span><i :style="{ width: `${progress}%` }" /></span><small>{{ lesson.duration }} 分钟 · 可随时继续</small></aside></section>
    <div class="lesson-layout"><section class="panel lesson-content"><nav class="lesson-tabs"><button type="button" :class="{ active: activeLayer === 'overview' }" @click="activeLayer = 'overview'">1. 核心理解</button><button type="button" :class="{ active: activeLayer === 'code' }" @click="activeLayer = 'code'">2. 最小代码</button><button type="button" :class="{ active: activeLayer === 'practice' }" @click="activeLayer = 'practice'">3. 练习与延伸</button></nav>
        <article v-if="activeLayer === 'overview'" class="lesson-section"><h2>先建立正确的直觉</h2><p>{{ lesson.overview }}</p><h3>关键要点</h3><ol><li v-for="point in lesson.keyPoints" :key="point">{{ point }}</li></ol><section class="prerequisite-card"><i class="pi pi-sitemap" /><div><strong>需要的前置知识</strong><p>{{ lesson.prerequisites.join('、') }}。如果这里有不熟悉的概念，先在知识地图中补齐，再回到本节。</p></div></section><div class="lesson-action-row"><button type="button" class="primary-button" @click="activeLayer = 'code'; markProgress(Math.max(progress, 35))">查看最小代码 <i class="pi pi-arrow-right" /></button><button type="button" class="primary-outline" @click="emit('navigate', 'assistant')"><i class="pi pi-sparkles" />让导师换一种方式解释</button></div></article>
        <article v-else-if="activeLayer === 'code'" class="lesson-section"><h2>最小代码示例</h2><p>下面的代码只表达核心流程，真实项目仍需要错误处理、日志与评估。</p><pre><code>{{ lesson.code }}</code></pre><section class="engineering-card"><i class="pi pi-wrench" /><div><strong>连接到工程实践</strong><p>阅读代码时不要只看 API：请指出输入验证在哪里做、失败如何记录，以及怎样用一个小评估集验证它。</p></div></section><button type="button" class="primary-button" @click="activeLayer = 'practice'; markProgress(Math.max(progress, 70))">完成一个小练习 <i class="pi pi-arrow-right" /></button></article>
        <article v-else class="lesson-section"><h2>用实践验证理解</h2><p class="practice-copy">{{ lesson.practice }}</p><section class="resource-card"><span><i class="pi pi-link" />延伸原始资料</span><a :href="lesson.sourceUrl" target="_blank" rel="noreferrer">{{ lesson.sourceLabel }} <i class="pi pi-external-link" /></a></section><div class="completion-actions"><button type="button" class="primary-button" @click="markProgress(100)"><i class="pi pi-check-circle" />标记本节完成</button><button type="button" class="primary-outline" @click="emit('navigate', 'assistant')"><i class="pi pi-question-circle" />让导师出题检验</button><button type="button" class="primary-outline" @click="markProgress(Math.max(progress, 70))">稍后继续</button></div></article>
      </section>
      <aside class="lesson-sidebar"><section class="panel lesson-outline"><h3>本节结构</h3><button type="button" :class="{ active: activeLayer === 'overview' }" @click="activeLayer = 'overview'"><i class="pi pi-lightbulb" />核心理解 <i v-if="progress >= 35" class="pi pi-check" /></button><button type="button" :class="{ active: activeLayer === 'code' }" @click="activeLayer = 'code'"><i class="pi pi-code" />最小代码 <i v-if="progress >= 70" class="pi pi-check" /></button><button type="button" :class="{ active: activeLayer === 'practice' }" @click="activeLayer = 'practice'"><i class="pi pi-flag" />练习与延伸 <i v-if="progress === 100" class="pi pi-check" /></button></section><section class="panel lesson-note"><h3>本节笔记</h3><p>写下你理解的新概念、一个疑问，或将要验证的实践假设。</p><form @submit.prevent="saveNote"><textarea v-model="note" placeholder="例如：重排解决的是召回后的排序问题…" /><button class="primary-outline">保存到学习笔记</button></form></section></aside>
    </div>
  </main>
</template>
