<script setup lang="ts">
import { computed } from 'vue'
import { useLearningStore, type PageKey } from './stores/learning'
import AppSidebar from './components/AppSidebar.vue'
import ExploreView from './views/ExploreView.vue'
import SearchView from './views/SearchView.vue'
import PathView from './views/PathView.vue'
import LibraryView from './views/LibraryView.vue'
import NotesView from './views/NotesView.vue'
import AssistantView from './views/AssistantView.vue'
import SettingsView from './views/SettingsView.vue'

const store = useLearningStore()
const currentView = computed(() => ({
  explore: ExploreView, search: SearchView, path: PathView, library: LibraryView,
  notes: NotesView, assistant: AssistantView, settings: SettingsView,
}[store.page]))

function navigate(page: PageKey) { store.go(page) }
function openAssistant(question: string) { store.addNote(`待追问：${question}`); store.go('assistant') }
</script>

<template>
  <div class="app-shell"><AppSidebar :active="store.page" @navigate="navigate" /><component :is="currentView" :query="store.query" :selected="store.selectedSources" :notes="store.notes" @navigate="navigate" @search="store.search" @ask="openAssistant" @add="store.addNote" @remove="store.removeNote" /></div>
</template>
