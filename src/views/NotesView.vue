<script setup lang="ts">
import { ref } from 'vue'
import StatCard from '../components/StatCard.vue'
const props = defineProps<{ notes: string[] }>()
const emit = defineEmits<{ add: [note: string]; remove: [index: number]; navigate: [page: 'library' | 'assistant'] }>()
const note = ref('')
function save() { emit('add', note.value); note.value = '' }
const saved = ['构建可观测 Agent 的实践清单', 'RAG 评估指标速记', 'Transformer 架构注意力分布', '检索质量问题排查']
</script>

<template>
  <main class="page notes-page"><section class="page-heading grid-heading"><h1>收藏与笔记</h1><p>把值得深思的资料和自己的理解留在一起。</p></section><div class="tab-row"><button class="active">全部</button><button>资料收藏</button><button>我的笔记</button><button>待深入</button><select><option>最近更新</option></select></div><div class="stats-row three"><StatCard icon="pi-file" label="收藏资料" :value="46" /><StatCard icon="pi-pencil" label="我的笔记" :value="36" tone="blue" /><StatCard icon="pi-clock" label="待深入" :value="12" tone="blue" /></div>
    <div class="notes-layout"><aside class="panel note-filter"><h3>标签与筛选</h3><p>标签</p><div class="tag-list"><span>RAG</span><span>Agent</span><span>Transformer</span><span>工程实践</span><span>论文</span><span>官方文档</span></div><p>文件夹</p><button><i class="pi pi-folder" />本周标记 <span>8</span><i class="pi pi-angle-right" /></button><button><i class="pi pi-folder" />高价值 <span>12</span><i class="pi pi-angle-right" /></button><button><i class="pi pi-folder" />稍后处理 <span>6</span><i class="pi pi-angle-right" /></button></aside>
      <section class="panel favorites"><header class="panel-title"><strong>最近收藏</strong></header><article v-for="(item, index) in saved" :key="item" :class="{ selected: index === 0 }"><div class="tech-thumb" :class="index === 0 ? 'blocks' : 'orb'"><i class="pi pi-circle-fill" /></div><div><span v-if="index === 0">官方文档</span><h3>{{ item }}</h3><p>{{ index === 0 ? '通篇梳理 Agent、日志追踪、评估与可观察性的落地要素，帮助团队快速建立可观测能力。' : '从原理到实践的学习资料，便于快速回顾。' }}</p><div class="tag-list"><span>Agent</span><span>工具调用</span><span>可观测性</span></div><small>更新于 {{ index + 1 }} 天前　· 官方文档</small></div><i class="pi pi-bookmark" /></article></section>
      <aside class="panel scratchpad"><h3>我的随手笔记</h3><article v-for="(item,index) in notes" :key="`${item}-${index}`"><h3>{{ item }}</h3><p>• Agent 是一个循环系统：感知 → 思考 → 行动 → 反思。<br>• 规划决定下一步应该做什么，工具决定边界。</p><div class="tag-list"><span>Agent</span><span>原理</span></div><small>编辑于 1 小时前</small><button @click="emit('remove', index)"><i class="pi pi-times" /></button></article><form @submit.prevent="save"><textarea v-model="note" placeholder="记下一段想法或一个还没弄懂的问题..." /><button class="primary-outline">保存笔记</button></form><button class="ai-note" @click="emit('navigate', 'assistant')"><i class="pi pi-sparkles" />AI 学习助手<br><small>把这条笔记变成知识卡</small></button></aside>
    </div><section class="panel notes-deep"><header class="panel-title"><strong>待深入（12）</strong><button>继续整理 <i class="pi pi-angle-right" /></button></header><div><article v-for="item in ['多智能体协作模式总结', '工具选择与编排策略', 'Agent 安全与边界控制', 'Long Context 处理要点']" :key="item"><i class="pi pi-chart-bar" /><h3>{{ item }}</h3><p>对比常见协作模式的适用场景与优缺点。</p><small>预计 25 分钟</small><i class="pi pi-check-circle" /></article></div></section>
  </main>
</template>
