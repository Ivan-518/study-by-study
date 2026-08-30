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
    <div class="assistant-greeting"><span class="bot-avatar"><i class="pi pi-android" /></span><div><strong v-if="!compact">今天想深入什么？</strong><strong v-else>你正在学习「设计与实现 Agent」阶段。</strong><p>{{ compact ? '这是构建智能体能力的关键一步，坚持住，你已经很棒了 💪' : '我可以帮你解答问题、推荐资源、生成知识卡片。' }}</p></div></div>
    <div class="assistant-actions">
      <button @click="emit('ask', '解释一下 RAG 的工作原理')"><i class="pi pi-lightbulb" />解释一下 RAG 的工作原理 <i class="pi pi-angle-right" /></button>
      <button @click="emit('ask', '推荐学习大模型微调的资源')"><i class="pi pi-file-edit" />推荐学习大模型微调的资源 <i class="pi pi-angle-right" /></button>
      <button @click="emit('open')"><i class="pi pi-code" />帮我生成一个 FastAPI 示例 <i class="pi pi-angle-right" /></button>
    </div>
    <form class="assistant-input" @submit.prevent="send"><input v-model="prompt" placeholder="向助手提问..." /><button aria-label="发送"><i class="pi pi-arrow-right" /></button></form>
    <small>AI 生成内容仅供参考</small>
  </section>
</template>
