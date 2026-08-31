<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'

type AssistantConfigStatus = { configured: boolean; baseUrl: string; model: string }
const autoResume = ref(true), citations = ref(true), notes = ref(true)
const baseUrl = ref('https://cf.api.fan/v1')
const model = ref('')
const apiKey = ref('')
const saving = ref(false)
const notice = ref('')
const configured = ref(false)

async function loadConfig() {
  if (!isTauri()) return
  try {
    const config = await invoke<AssistantConfigStatus>('get_assistant_config')
    baseUrl.value = config.baseUrl || baseUrl.value
    model.value = config.model
    configured.value = config.configured
  } catch (error) {
    notice.value = typeof error === 'string' ? error : '无法读取模型配置。'
  }
}
async function saveConfig() {
  if (!isTauri()) { notice.value = '请在桌面应用中保存模型配置。'; return }
  saving.value = true
  notice.value = ''
  try {
    const config = await invoke<AssistantConfigStatus>('save_assistant_config', { input: { baseUrl: baseUrl.value, model: model.value, apiKey: apiKey.value } })
    configured.value = config.configured
    apiKey.value = ''
    notice.value = '模型已配置。API Key 已保存到 Windows 凭据管理器，不会写入项目文件或本地数据库。'
  } catch (error) {
    notice.value = typeof error === 'string' ? error : '无法保存模型配置。'
  } finally {
    saving.value = false
  }
}
onMounted(() => { void loadConfig() })
</script>

<template>
  <main class="page settings-page"><section class="page-heading grid-heading"><h1>设置</h1><p>学习数据保留在本机；模型服务仅在你主动提问时调用。</p></section><div class="database-ok"><i class="pi pi-check-circle" />本地学习数据已启用 · 课程进度、笔记与学习清单均保存于本机</div><div class="settings-grid"><section class="panel settings-main"><h2>AI 模型与连接</h2><article class="cloud-model configured-model"><i :class="configured ? 'pi pi-check-circle' : 'pi pi-cloud-download'" /><div><h3>{{ configured ? 'AI 学习导师已配置' : '配置 OpenAI 兼容模型服务' }}</h3><span :class="{ online: configured }">{{ configured ? '已连接配置' : '未配置' }}</span><p>调用只由 Rust 本地后端发起，前端不会读取 API Key。</p></div></article><form class="assistant-config" @submit.prevent="saveConfig"><label>服务地址<input v-model="baseUrl" type="url" autocomplete="url" placeholder="https://example.com/v1" /></label><label>模型名称<input v-model="model" autocomplete="off" placeholder="例如：gpt-4o-mini 或服务商提供的模型 ID" /></label><label>API Key<input v-model="apiKey" type="password" autocomplete="new-password" placeholder="仅保存到 Windows 凭据管理器" /></label><button class="primary-button" :disabled="saving">{{ saving ? '保存中…' : configured ? '更新模型配置' : '保存并启用导师' }} <i class="pi pi-arrow-right" /></button></form><p v-if="notice" class="config-notice">{{ notice }}</p><div class="privacy-callout"><i class="pi pi-shield" /><span><strong>发送边界</strong>：只在提问时发送问题、当前课程/资料的最小必要上下文；不会自动上传学习清单、笔记或本地数据库。</span></div><h2>内容来源</h2><div class="source-tabs"><button><i class="pi pi-chart-line" />热榜</button><button><i class="pi pi-github" />GitHub</button><button><i class="pi pi-book" />内置课程</button><button><i class="pi pi-database" />本地搜索</button></div></section><aside><section class="panel local-data"><h2><i class="pi pi-database" />本地数据</h2><button><i class="pi pi-folder" />学习资料、笔记与清单<span>正常</span><i class="pi pi-angle-right" /></button><button><i class="pi pi-search" />本地学习搜索索引<span>正常</span><i class="pi pi-angle-right" /></button><button><i class="pi pi-key" />模型密钥<span>Windows 凭据库</span><i class="pi pi-lock" /></button></section><section class="panel preferences"><h2>体验偏好</h2><label><i class="pi pi-history" />启动时恢复上次学习<input v-model="autoResume" type="checkbox" /><b /></label><label><i class="pi pi-link" />助手回答附带来源<input v-model="citations" type="checkbox" /><b /></label><label><i class="pi pi-pencil" />自动保存笔记<input v-model="notes" type="checkbox" /><b /></label></section><section class="panel about"><strong><b>N</b> Nexus 学习站　· 0.1.0</strong><small>Tauri 2 / 本地优先桌面应用</small></section></aside></div></main>
</template>
