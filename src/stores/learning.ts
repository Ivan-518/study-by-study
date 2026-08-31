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

const savedNotes = localStorage.getItem('nexus-notes')
const savedLearningItems = localStorage.getItem('nexus-learning-items')

function readLearningItems(): LearningItem[] {
  if (!savedLearningItems) return []
  try {
    const value = JSON.parse(savedLearningItems) as unknown
    return Array.isArray(value) ? value as LearningItem[] : []
  } catch {
    return []
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
  },
})
