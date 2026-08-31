<script setup lang="ts">
import { computed, ref } from 'vue'
import { useLearningStore, type PageKey } from './stores/learning'
import AppSidebar from './components/AppSidebar.vue'
import ExploreView from './views/ExploreView.vue'
import RadarView from './views/RadarView.vue'
import SearchView from './views/SearchView.vue'
import PathView from './views/PathView.vue'
import LessonView from './views/LessonView.vue'
import LibraryView from './views/LibraryView.vue'
import NotesView from './views/NotesView.vue'
import AssistantView from './views/AssistantView.vue'
import SettingsView from './views/SettingsView.vue'
import AssistantPanel from './components/AssistantPanel.vue'

const store = useLearningStore()
const drawerOpen = ref(false)
const currentView = computed(() => ({
  explore: ExploreView, radar: RadarView, search: SearchView, path: PathView, library: LibraryView,
  notes: NotesView, assistant: AssistantView, settings: SettingsView,
}[store.page]))

function navigate(page: PageKey) { store.setCurrentLesson(''); store.go(page) }
function openAssistant(question: string) { store.addNote(`待追问：${question}`); store.go('assistant') }
function openFullAssistant() { drawerOpen.value = false; store.go('assistant') }
</script>

<template>
  <div class="app-shell"><AppSidebar :active="store.page" @navigate="navigate" /><LessonView v-if="store.currentLessonId" :lesson-id="store.currentLessonId" :progress="store.lessonProgress[store.currentLessonId] || 0" :assessment="store.lessonAssessments[store.currentLessonId]" @back="store.setCurrentLesson('')" @navigate="store.go" @update-progress="store.updateLessonProgress" @save-assessment="store.saveLessonAssessment" @clear-assessment="store.clearLessonAssessment" @add="store.addNote" /><component v-else :is="currentView" :query="store.query" :selected="store.selectedSources" :notes="store.notes" :learning-items="store.learningItems" :lesson-progress="store.lessonProgress" :current-lesson-id="store.currentLessonId" @navigate="navigate" @search="store.search" @ask="openAssistant" @add="store.addNote" @remove="store.removeNote" @add-learning="store.addLearningItem" @set-learning-status="store.setLearningStatus" @remove-learning="store.removeLearningItem" @open-lesson="store.setCurrentLesson" />
    <button class="assistant-fab" type="button" @click="drawerOpen = true"><i class="pi pi-sparkles" /></button>
    <aside v-if="drawerOpen" class="tutor-drawer" aria-label="AI 学习助手侧栏"><button class="drawer-close" type="button" aria-label="关闭 AI 学习助手" @click="drawerOpen = false"><i class="pi pi-times" /></button><AssistantPanel @open="openFullAssistant" @ask="openFullAssistant" /></aside></div>
</template>
