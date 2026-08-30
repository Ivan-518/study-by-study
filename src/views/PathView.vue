<script setup lang="ts">
const emit = defineEmits<{ navigate: [page: 'assistant' | 'library'] }>()
const modules = [
  ['理解 Agent 的决策循环', '掌握 ReAct 思维模式、规划、反思与行动的循环机制。', '阅读 12 分钟'],
  ['工具调用与状态管理', '学习如何定义工具、管理工具调用与异步状态。', '阅读 15 分钟'],
  ['实现一个可观察的 Agent', '构建可追踪、可观测，并输出执行轨迹与中间结果的 Agent。', '阅读 18 分钟'],
  ['多轮记忆与上下文管理', '阅读 14 分钟'], ['错误处理与降级策略', '阅读 12 分钟'],
]
</script>

<template>
  <main class="page path-page"><section class="page-heading grid-heading"><h1>构建 Agentic RAG 应用</h1><p>灵活可暂停，按自己的节奏掌握 Agentic RAG 的核心能力。</p></section>
    <section class="progress-banner panel"><strong>已完成 <b>2 / 5</b> 个阶段</strong><div><span>学习进度 <b>64%</b></span><div class="progress-line"><i /></div></div><div class="progress-ring small"><b>64%</b></div><button class="primary-button">继续学习</button></section>
    <div class="path-layout"><aside class="panel path-directory"><h3>路径目录</h3><button class="done"><b>01</b><span>RAG 核心概念<small>已完成</small></span><i class="pi pi-check-circle" /></button><button class="done"><b>02</b><span>知识检索系统<small>已完成</small></span><i class="pi pi-check-circle" /></button><button class="active"><b>03</b><span>设计与实现 Agent<small>当前学习</small></span><i class="pi pi-angle-right" /></button><button><b>04</b><span>评估、部署与优化<small>未开始</small></span><i class="pi pi-lock" /></button><button><b>05</b><span>实践：构建个人学习助手<small>未开始</small></span><i class="pi pi-lock" /></button><p class="path-tip"><i class="pi pi-lightbulb" />完成当前阶段后将解锁下一阶段</p></aside>
      <section class="panel learning-stage"><header class="panel-title"><strong>当前学习：设计与实现 Agent</strong></header><div class="stage-list"><article v-for="(module, index) in modules" :key="module[0]" :class="{ featured: index === 2 }"><i class="stage-check pi" :class="index < 2 ? 'pi-check-circle' : index === 2 ? 'pi-circle-fill' : 'pi-circle'" /><div class="mini-art"><i class="pi" :class="index === 0 ? 'pi-refresh' : index === 1 ? 'pi-wrench' : index === 2 ? 'pi-eye' : 'pi-file'" /></div><div><h3>{{ module[0] }}</h3><p v-if="index < 3">{{ module[1] }}</p><small>{{ module[index > 2 ? 1 : 2] }}　难度 ★★★☆☆</small></div><i class="pi" :class="index < 2 ? 'pi-check-circle' : index === 2 ? 'pi-check-circle' : 'pi-lock'" />
            <div v-if="index === 2" class="featured-extra"><div class="feature-visual"><i class="pi pi-search" /></div><ul><li>可观测 Agent 执行轨迹</li><li>记录与回放中间步骤</li><li>便于调试与性能优化</li></ul><button class="primary-button" @click="emit('navigate', 'assistant')">进入本节学习 <i class="pi pi-arrow-right" /></button></div>
          </article></div></section>
    <section class="panel continuation path-continuation"><header><strong>继续探索</strong><span>按兴趣自由探索，随时暂停或恢复</span></header><div><button v-for="item in ['RAG 检索评测', 'Agent Memory', 'Tool Calling', '安全与可观测性']" :key="item"><span class="tech-thumb small-thumb"><i class="pi pi-circle-fill" /></span><strong>{{ item }}</strong><small>阅读 20 分钟</small><i class="pi pi-bookmark" /></button></div></section></div></main>
</template>
