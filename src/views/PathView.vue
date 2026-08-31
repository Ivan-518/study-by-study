<script setup lang="ts">
import { computed, ref } from 'vue'
import { lessons, type Lesson, type LearningTrack } from '../data/learningContent'
import type { LessonAssessment } from '../stores/learning'

const props = defineProps<{ lessonProgress: Record<string, number>; lessonAssessments: Record<string, LessonAssessment>; currentLessonId: string }>()
const emit = defineEmits<{ 'open-lesson': [id: string] }>()
const activeTrack = ref<LearningTrack>('应用开发')
const trackLessons = computed(() => lessons.filter((lesson) => lesson.track === activeTrack.value))
const completedCount = computed(() => lessons.filter((lesson) => (props.lessonProgress[lesson.id] || 0) === 100).length)
const overallProgress = computed(() => lessons.length ? Math.round(Object.values(props.lessonProgress).reduce((sum, value) => sum + value, 0) / lessons.length) : 0)

function lessonStatus(lesson: Lesson) {
  const progress = props.lessonProgress[lesson.id] || 0
  const assessment = props.lessonAssessments[lesson.id]
  if (assessment) return assessment.level === 'mastered' ? '已掌握' : assessment.level === 'practice' ? '需要实践' : '需要复习'
  if (progress === 100) return '已完成'
  if (progress > 0) return `已学习 ${progress}%`
  return '未开始'
}
function openLesson(id: string) { emit('open-lesson', id) }
</script>

<template>
  <main class="page path-page">
    <section class="page-heading grid-heading"><h1>知识地图与课程目录</h1><p>两条主线并行：把模型原理连接到 AI 应用工程中的真实决策。</p></section>
    <section class="progress-banner panel"><strong>已完成 <b>{{ completedCount }} / {{ lessons.length }}</b> 节</strong><div><span>整体进度 <b>{{ overallProgress }}%</b></span><div class="progress-line"><i :style="{ width: `${overallProgress}%` }" /></div></div><div class="progress-ring small" :style="{ background: `conic-gradient(#0966ed ${overallProgress}%, #e7edf7 0)` }"><b>{{ overallProgress }}%</b></div><button class="primary-button" type="button" @click="openLesson(currentLessonId || lessons[0].id)">继续学习 <i class="pi pi-arrow-right" /></button></section>
    <div class="course-tabs" role="tablist"><button v-for="track in (['应用开发', '模型原理'] as LearningTrack[])" :key="track" type="button" :class="{ active: activeTrack === track }" @click="activeTrack = track"><i :class="track === '应用开发' ? 'pi pi-sitemap' : 'pi pi-sparkles'" />{{ track }}<small>{{ track === '应用开发' ? 'RAG、Agent、评估与工程' : 'Transformer、Embedding、推理与量化' }}</small></button></div>
    <section class="course-directory panel"><header class="panel-title"><strong>{{ activeTrack }}课程</strong><span>{{ trackLessons.length }} 节 · 可随时暂停和恢复</span></header><article v-for="(lesson, index) in trackLessons" :key="lesson.id" class="course-row" :class="{ current: lesson.id === currentLessonId, complete: lessonProgress[lesson.id] === 100 }"><b>{{ String(index + 1).padStart(2, '0') }}</b><div class="course-row-icon"><i :class="lesson.track === '应用开发' ? 'pi pi-wrench' : 'pi pi-sparkles'" /></div><div class="course-row-copy"><span>{{ lesson.level }} · {{ lesson.duration }} 分钟</span><h2>{{ lesson.title }}</h2><p>{{ lesson.description }}</p><div class="tag-list"><span v-for="concept in lesson.concepts" :key="concept">{{ concept }}</span></div></div><div class="course-row-progress"><strong>{{ lessonStatus(lesson) }}</strong><span><i :style="{ width: `${lessonProgress[lesson.id] || 0}%` }" /></span></div><button type="button" class="primary-outline" @click="openLesson(lesson.id)">{{ (lessonProgress[lesson.id] || 0) > 0 ? '继续' : '开始' }} <i class="pi pi-arrow-right" /></button></article></section>
  </main>
</template>
