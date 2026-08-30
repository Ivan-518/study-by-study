<script setup lang="ts">
import { ref } from 'vue'
const emit = defineEmits<{ navigate: [page: 'path' | 'library'] }>()
const question = ref('Agent 的规划、记忆与工具调用分别解决什么问题？')
const followUp = ref('')
const messages = ref([{ user: question.value }])
function send() { if (!followUp.value.trim()) return; messages.value.push({ user: followUp.value }); followUp.value = '' }
</script>

<template>
  <main class="page assistant-page"><section class="page-heading grid-heading"><h1><i class="pi pi-sparkles" /> AI 学习助手</h1><p>围绕你正在学习的内容随时提问。</p></section><div class="context-bar panel"><i class="pi pi-book" />当前上下文：构建 Agentic RAG 应用 · 第 3 阶段 <button>切换上下文 <i class="pi pi-sync" /></button></div>
    <div class="assistant-layout"><aside class="panel assistant-context"><h3>学习上下文</h3><p>当前学习内容</p><button> <i class="pi pi-file" />实现一个可观察的 Agent <i class="pi pi-angle-right" /></button><p>关联资料（3）</p><button class="source"><i class="pi pi-file" />官方文档 <i class="pi pi-check-circle" /></button><button class="source"><i class="pi pi-github" />GitHub <i class="pi pi-check-circle" /></button><button class="source"><i class="pi pi-file-edit" />学习笔记 <i class="pi pi-check-circle" /></button><p>你可能想问</p><button v-for="q in ['Agent 的记忆有哪些类型？', '工具调用失败时如何处理？', '如何让 Agent 的规划更可靠？', '可观测性在 Agent 中如何落地？']" :key="q" @click="question=q">{{ q }}<i class="pi pi-angle-right" /></button></aside>
      <section class="panel chat-area"><div v-for="message in messages" :key="message.user" class="chat-question"><span><i class="pi pi-user" /></span><p>{{ message.user }}</p><small>10:24</small></div><article class="chat-answer"><span class="n-avatar">N</span><div><p>这是构建 Agent 时最核心的三大能力模块，它们各自解决的问题不同，但协同工作，让 Agent 能够更智能、更稳定地完成复杂任务。</p><section v-for="([num,title,desc,icon]) in [['1','规划（Planning）','解决“下一步该做什么”的问题。将用户目标拆解为可执行的子任务，决定步骤顺序与策略。','pi-sitemap'],['2','记忆（Memory）','解决“需要记住什么，从哪里取回”的问题。存储并检索上下文、历史结果与长期知识，保持任务连续性。','pi-database'],['3','工具调用（Tool Use）','解决“如何与外部世界交互”的问题。通过调用外部 API（如搜索、计算等）获得信息或执行操作。','pi-wrench']]" :key="title" class="explanation"><i class="pi" :class="icon" /><div><h3><b>{{ num }}</b>{{ title }}</h3><p>{{ desc }}</p></div></section><p>把它理解成：规划是大脑（想清楚），记忆是心智（记住），工具调用是双手（做得到）。三者协同，Agent 才能从思考走向对外持续行动。</p><div class="citations">来源：<span>官方文档</span><span>GitHub</span><span>学习笔记</span></div></div></article><form class="chat-input" @submit.prevent="send"><input v-model="followUp" placeholder="继续追问，或粘贴一段内容..." /><button><i class="pi pi-send" /></button></form></section>
      <aside class="assistant-aid"><section class="panel"><h3>学习辅助</h3><button><i class="pi pi-book" />生成本节知识卡<i class="pi pi-angle-right" /></button><button><i class="pi pi-question-circle" />检查我的理解<i class="pi pi-angle-right" /></button><div class="aid-progress"><div class="progress-ring"><b>64%</b></div><p><strong>继续加油！</strong><small>已掌握 64%，继续巩固可提升理解与应用能力。</small></p></div></section><section class="panel"><h3>本次对话要点</h3><button v-for="text in ['规划负责拆解任务与制定执行策略', '记忆用于存储与检索上下文', '工具调用让 Agent 能与外部系统交互']" :key="text"><i class="pi pi-file" />{{ text }}<i class="pi pi-bookmark" /></button><button class="link-button" @click="emit('navigate', 'library')">查看全部要点（3）<i class="pi pi-angle-right" /></button></section></aside>
    </div>
  </main>
</template>
