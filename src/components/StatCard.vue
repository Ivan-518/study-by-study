<script setup lang="ts">
defineProps<{ icon: string; label: string; value: string | number; tone?: string }>()
</script>

<template>
  <article class="stat-card">
    <i class="stat-icon pi" :class="[icon, tone]" />
    <div><small>{{ label }}</small><strong>{{ value }}</strong></div>
  </article>
</template>
