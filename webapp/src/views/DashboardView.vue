<template>
  <div class="w-full min-h-[calc(100vh-5rem)] flex flex-col items-center justify-center pt-8 relative">
    <div class="w-full max-w-4xl bg-card/80 rounded-2xl shadow-lg p-4 flex flex-col items-center justify-center">
      <button
        v-if="!deviceStore.deviceConfig && !deviceStore.loading"
        @click="deviceStore.fetchConfig()"
        class="mb-6 px-6 py-2 rounded-lg bg-primary text-primary-foreground font-semibold shadow hover:bg-primary/80 transition"
      >
        Connect to Device
      </button>
      <button
        v-if="!deviceStore.deviceConfig && deviceStore.loading"
        disabled
        class="mb-6 px-6 py-2 rounded-lg bg-muted text-muted-foreground font-semibold shadow cursor-not-allowed"
      >
        Connecting...
      </button>
      <ButtonGrid :button-labels="buttonLabels" />
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-4">
      <DeviceStatus
        :status="deviceStore.deviceConfig ? 'Connected' : undefined"
        :wifi="deviceStore.deviceConfig?.settings?.wifi?.ssid ?? undefined"
        :time="deviceStore.deviceConfig ? '12:34' : undefined" />
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-2">
      <div class="text-xs text-muted-foreground font-mono">Time: Synced</div>
    </div>
    <div v-if="deviceStore.loading" class="absolute inset-0 flex items-center justify-center bg-background/80 rounded-2xl z-10">
      <span class="text-muted-foreground">Loading...</span>
    </div>
    <FeedbackToast
      v-if="deviceStore.error"
      :message="deviceStore.error"
      type="error"
      :show="!!deviceStore.error"
    />
  </div>
</template>

<script setup lang="ts">
import ButtonGrid from '../components/ButtonGrid.vue'
import DeviceStatus from '../components/DeviceStatus.vue'
import FeedbackToast from '../components/FeedbackToast.vue'
import { useDeviceStore } from '../stores/deviceStore'
import { computed } from 'vue'

const deviceStore = useDeviceStore()

const defaultLabels = Array.from({ length: 16 }, (_, i) => `Button ${i + 1}`)

const buttonLabels = computed(() => {
  const names = deviceStore.deviceConfig?.button_names
  if (Array.isArray(names) && names.length === 16) return names
  if (names && typeof names === 'object') {
    // If button_names is an object (from backend), map to array
    return Array.from({ length: 16 }, (_, i) => names[i] || defaultLabels[i])
  }
  return defaultLabels
})
</script> 