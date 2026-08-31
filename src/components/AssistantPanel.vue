<script setup lang="ts">
import { ref } from 'vue'

const props = withDefaults(defineProps<{ compact?: boolean; title?: string }>(), { compact: false, title: 'AI 学习助手' })
const emit = defineEmits<{ open: []; ask: [question: string] }>()
const prompt = ref('')

function send() {
  if (!prompt.value.trim()) return
  emit('ask', prompt.value)
  prompt.value = ''
}
</script>

<template>
  <section class="assistant-panel" :class="{ compact }">
    <header class="panel-title"><span><i class="pi pi-sparkles" />{{ title }}</span><span><i class="pi pi-refresh" /> <i class="pi pi-ellipsis-h" /></span></header>
    <div class="assistant-greeting"><span class="bot-avatar"><i class="pi pi-lock" /></span><div><strong v-if="!compact">AI 导师待配置</strong><strong v-else>配置模型后可在这里提问</strong><p>{{ compact ? '本地课程进度会保留；连接模型后才会生成个性化解释。' : '为了避免伪造式回答，连接云端模型前不会生成内容。' }}</p></div></div>
    <div class="assistant-actions">
      <button @click="emit('open')"><i class="pi pi-cog" />配置模型与数据边界 <i class="pi pi-angle-right" /></button>
      <button @click="emit('open')"><i class="pi pi-book" />先完成本地课程 <i class="pi pi-angle-right" /></button>
    </div>
    <form class="assistant-input" @submit.prevent="send"><input v-model="prompt" disabled placeholder="配置模型后可在此提问" /><button aria-label="发送" disabled><i class="pi pi-arrow-right" /></button></form>
    <small>配置前不会发送任何内容到外部服务</small>
  </section>
</template>
