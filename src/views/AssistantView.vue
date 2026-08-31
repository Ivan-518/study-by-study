<script setup lang="ts">
const emit = defineEmits<{ navigate: [page: 'path' | 'settings'] }>()
</script>

<template>
  <main class="page assistant-page">
    <section class="page-heading grid-heading"><h1><i class="pi pi-sparkles" /> AI 学习导师</h1><p>导师会围绕当前课程、已选资料与笔记提供解释、练习和方案审阅。</p></section>
    <section class="assistant-setup panel"><div class="assistant-setup-icon"><i class="pi pi-sparkles" /></div><div><p class="eyebrow">本地学习闭环已可用</p><h2>配置模型后启用个性化解释</h2><p>当前没有连接云端模型，因此不会生成看似真实但无法验证的回答。你仍可使用课程、搜索、学习清单和笔记；配置后，导师才会根据你主动选择的上下文回答。</p><div class="assistant-setup-actions"><button type="button" class="primary-button" @click="emit('navigate', 'settings')">去配置模型 <i class="pi pi-arrow-right" /></button><button type="button" class="primary-outline" @click="emit('navigate', 'path')">继续本地学习</button></div></div></section>
    <div class="assistant-setup-grid"><section class="panel setup-card"><i class="pi pi-shield" /><h3>数据边界</h3><p>只有你主动发起提问时，问题文本和你选择的最小必要上下文才会发送给模型服务；学习进度、笔记和本地索引不会自动上传。</p></section><section class="panel setup-card"><i class="pi pi-file-edit" /><h3>配置后能做什么</h3><ul><li>按不同深度解释概念、代码或公式</li><li>从当前课程生成小测与实践建议</li><li>将外部资料改写为可学习的卡片</li><li>审阅技术方案并提示前置知识</li></ul></section><section class="panel setup-card"><i class="pi pi-link" /><h3>导师使用的上下文</h3><p>当前课程、你选择的学习清单条目、相关笔记和来源链接。每次请求都会在发送前清楚展示所用内容。</p></section></div>
    <section class="panel assistant-local-actions"><header class="panel-title"><strong>无需模型也可继续</strong></header><button type="button" @click="emit('navigate', 'path')"><i class="pi pi-book" /><span><strong>完成一节内置课程</strong><small>按“概念 → 代码 → 练习”建立扎实理解。</small></span><i class="pi pi-angle-right" /></button><button type="button" @click="emit('navigate', 'settings')"><i class="pi pi-cog" /><span><strong>查看隐私与模型配置</strong><small>选择供应商前，先确认发送的数据边界与成本。</small></span><i class="pi pi-angle-right" /></button></section>
  </main>
</template>
