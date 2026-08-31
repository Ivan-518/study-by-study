import { defineStore } from 'pinia'

export type PageKey = 'explore' | 'radar' | 'search' | 'path' | 'library' | 'notes' | 'assistant' | 'settings'
export type LearningGoal = '了解概念' | '动手试用' | '深入学习'
export type LearningStatus = 'todo' | 'doing' | 'done'
export type LearningItem = {
  id: string
  title: string
  summary: string
  url: string
  topics: string[]
  source: string
  goal: LearningGoal
  status: LearningStatus
  createdAt: string
  completedAt?: string
}
export type DiscoveryLearningItem = Pick<LearningItem, 'title' | 'summary' | 'url' | 'topics' | 'source'>
export type LessonAssessmentLevel = 'mastered' | 'review' | 'practice'
export type LessonAssessment = {
  level: LessonAssessmentLevel
  feedback: string
  strengths: string[]
  gap: string
  nextStep: string
  followUpQuestion: string
  assessedAt: string
  model: string
}

const savedNotes = localStorage.getItem('nexus-notes')
const savedLearningItems = localStorage.getItem('nexus-learning-items')
const savedLessonProgress = localStorage.getItem('nexus-lesson-progress')
const savedLessonAssessments = localStorage.getItem('nexus-lesson-assessments')

function readLearningItems(): LearningItem[] {
  if (!savedLearningItems) return []
  try {
    const value = JSON.parse(savedLearningItems) as unknown
    return Array.isArray(value) ? value as LearningItem[] : []
  } catch {
    return []
  }
}
function readLessonProgress(): Record<string, number> {
  if (!savedLessonProgress) return {}
  try {
    const value = JSON.parse(savedLessonProgress) as unknown
    return value && typeof value === 'object' ? value as Record<string, number> : {}
  } catch {
    return {}
  }
}
function readLessonAssessments(): Record<string, LessonAssessment> {
  if (!savedLessonAssessments) return {}
  try {
    const value = JSON.parse(savedLessonAssessments) as unknown
    return value && typeof value === 'object' ? value as Record<string, LessonAssessment> : {}
  } catch {
    return {}
  }
}

export const useLearningStore = defineStore('learning', {
  state: () => ({
    page: 'explore' as PageKey,
    query: 'Agentic RAG 最佳实践',
    progress: 64,
    notes: savedNotes ? JSON.parse(savedNotes) as string[] : ['Agent 不是一次性调用 LLM', '如何评估 RAG 的检索质量？'],
    selectedSources: ['Agentic RAG 最佳实践指南', 'agentic-rag-best-practices', '面向复杂任务的 Agentic RAG'],
    learningItems: readLearningItems(),
    lessonProgress: readLessonProgress(),
    lessonAssessments: readLessonAssessments(),
    currentLessonId: '',
  }),
  actions: {
    go(page: PageKey) { this.page = page },
    search(query: string) { this.query = query.trim() || 'Agentic RAG 最佳实践'; this.page = 'search' },
    addNote(note: string) {
      const value = note.trim()
      if (!value) return
      this.notes.unshift(value)
      localStorage.setItem('nexus-notes', JSON.stringify(this.notes))
    },
    removeNote(index: number) {
      this.notes.splice(index, 1)
      localStorage.setItem('nexus-notes', JSON.stringify(this.notes))
    },
    addLearningItem(item: DiscoveryLearningItem, goal: LearningGoal) {
      const existing = this.learningItems.find((entry) => entry.url === item.url)
      if (existing) {
        existing.goal = goal
        existing.status = existing.status === 'done' ? 'todo' : existing.status
      } else {
        this.learningItems.unshift({
          ...item,
          id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
          goal,
          status: 'todo',
          createdAt: new Date().toISOString(),
        })
      }
      localStorage.setItem('nexus-learning-items', JSON.stringify(this.learningItems))
    },
    setLearningStatus(id: string, status: LearningStatus) {
      const item = this.learningItems.find((entry) => entry.id === id)
      if (!item) return
      item.status = status
      item.completedAt = status === 'done' ? new Date().toISOString() : undefined
      localStorage.setItem('nexus-learning-items', JSON.stringify(this.learningItems))
    },
    removeLearningItem(id: string) {
      this.learningItems = this.learningItems.filter((item) => item.id !== id)
      localStorage.setItem('nexus-learning-items', JSON.stringify(this.learningItems))
    },
    updateLessonProgress(id: string, progress: number) {
      this.lessonProgress[id] = Math.max(0, Math.min(100, Math.round(progress)))
      this.currentLessonId = id
      localStorage.setItem('nexus-lesson-progress', JSON.stringify(this.lessonProgress))
    },
    setCurrentLesson(id: string) { this.currentLessonId = id },
    saveLessonAssessment(id: string, assessment: Omit<LessonAssessment, 'assessedAt'>) {
      this.lessonAssessments[id] = { ...assessment, assessedAt: new Date().toISOString() }
      const progress = assessment.level === 'mastered' ? 100 : Math.max(this.lessonProgress[id] || 0, 70)
      this.updateLessonProgress(id, progress)
      localStorage.setItem('nexus-lesson-assessments', JSON.stringify(this.lessonAssessments))
    },
    clearLessonAssessment(id: string) {
      delete this.lessonAssessments[id]
      localStorage.setItem('nexus-lesson-assessments', JSON.stringify(this.lessonAssessments))
    },
  },
})
