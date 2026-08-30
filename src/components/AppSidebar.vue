<script setup lang="ts">
import type { PageKey } from '../stores/learning'

defineProps<{ active: PageKey }>()
const emit = defineEmits<{ navigate: [page: PageKey] }>()

const nav: { key: PageKey; label: string; icon: string }[] = [
  { key: 'explore', label: '探索', icon: 'pi-compass' },
  { key: 'path', label: '学习路径', icon: 'pi-map' },
  { key: 'library', label: '知识库', icon: 'pi-book' },
  { key: 'notes', label: '收藏', icon: 'pi-star' },
]
</script>

<template>
  <aside class="sidebar">
    <button class="brand" aria-label="Nexus 学习站" @click="emit('navigate', 'explore')"><b>N</b><span>Nexus 学习站</span></button>
    <nav class="side-nav" aria-label="主导航">
      <button v-for="item in nav" :key="item.key" class="nav-item" :class="{ active: active === item.key }" @click="emit('navigate', item.key)">
        <i class="pi" :class="item.icon" /><span>{{ item.label }}</span>
      </button>
    </nav>
    <div class="sidebar-bottom">
      <button class="nav-item" :class="{ active: active === 'settings' }" @click="emit('navigate', 'settings')"><i class="pi pi-cog" /><span>设置</span></button>
      <span class="collapse"><i class="pi pi-angle-left" /></span>
    </div>
  </aside>
</template>
