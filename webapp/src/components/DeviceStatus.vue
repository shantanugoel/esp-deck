<template>
  <div class="w-full rounded-lg bg-muted text-muted-foreground p-4 flex flex-col items-center justify-center mb-6">
    <div class="font-semibold">Device Status: <span :class="statusClass">{{ status }}</span></div>
    <div class="mt-2">WiFi: <span class="font-mono">{{ wifi }}</span></div>
    <div class="mt-1">Time: <span class="font-mono">{{ time }}</span></div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineProps } from 'vue'

const props = defineProps<{
  status?: string
  wifi?: string
  time?: string
}>()

const status = props.status ?? 'Not connected'
const wifi = props.wifi ?? '-'
const time = props.time ?? '-'

const statusClass = computed(() =>
  status === 'Connected' ? 'text-primary' : 'text-destructive')
</script> 