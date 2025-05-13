<template>
  <div class="w-full rounded-lg bg-muted text-muted-foreground p-4 flex flex-col items-center justify-center mb-6">
    <div class="font-semibold">Device Status: <span :class="statusClass">{{ statusValue }}</span></div>
    <div class="mt-2">WiFi: <span class="font-mono">{{ wifiValue }}</span></div>
    <div class="mt-1">Time: <span class="font-mono">{{ timeValue }}</span></div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineProps } from 'vue'

const props = defineProps<{
  status?: string
  wifi?: string
  time?: string
}>()

const statusValue = computed(() => props.status ?? 'Not connected')
const wifiValue = computed(() => props.wifi ?? '-')
const timeValue = computed(() => props.time ?? '-')

const statusClass = computed(() =>
  statusValue.value === 'Connected' ? 'text-primary' : 'text-destructive')
</script> 