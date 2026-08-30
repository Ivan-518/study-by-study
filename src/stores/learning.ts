import { defineStore } from 'pinia'

export type PageKey = 'explore' | 'search' | 'path' | 'library' | 'notes' | 'assistant' | 'settings'

const savedNotes = localStorage.getItem('nexus-notes')

export const useLearningStore = defineStore('learning', {
  state: () => ({
    page: 'explore' as PageKey,
    query: 'Agentic RAG 最佳实践',
    progress: 64,
    notes: savedNotes ? JSON.parse(savedNotes) as string[] : ['Agent 不是一次性调用 LLM', '如何评估 RAG 的检索质量？'],
    selectedSources: ['Agentic RAG 最佳实践指南', 'agentic-rag-best-practices', '面向复杂任务的 Agentic RAG'],
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
  },
})
