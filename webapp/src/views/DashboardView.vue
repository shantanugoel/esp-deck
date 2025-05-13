<template>

  <div class="w-full min-h-[calc(100vh-5rem)] flex flex-col items-center justify-center pt-8 relative">
    <div class="w-full max-w-4xl bg-card/80 rounded-2xl shadow-lg p-4 flex flex-col items-center justify-center">
      <button
        v-if="!deviceApi.isConnected && !deviceApi.loading"
        @click="connectAndFetch"
        class="mb-6 px-6 py-2 rounded-lg bg-primary text-primary-foreground font-semibold shadow hover:bg-primary/80 transition"
      >
        Connect to Device
      </button>
      <button
        v-if="!deviceApi.isConnected && deviceApi.loading"
        disabled
        class="mb-6 px-6 py-2 rounded-lg bg-muted text-muted-foreground font-semibold shadow cursor-not-allowed"
      >
        Connecting...
      </button>
      <div class="w-full">
        <div class="grid grid-cols-4 gap-3 w-full mb-2">
          <div v-for="(label, idx) in buttonLabels" :key="idx" class="flex flex-col items-center">
            <button
              class="h-12 sm:h-14 md:h-16 w-full rounded-lg bg-primary text-primary-foreground font-semibold text-base flex items-center justify-center shadow hover:bg-muted transition"
              @click="goToMacroEditor(idx)"
            >
              {{ label }}
            </button>
          </div>
        </div>
      </div>
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-4">
      <DeviceStatus
        :status="deviceStatus"
        :wifi="deviceWifi"
        :time="deviceTime" />
    </div>
    <div v-if="deviceApi.isConnected" class="w-full max-w-4xl flex justify-center mt-4 gap-4">
      <button
        @click="onResetConfig"
        :disabled="deviceApi.loading"
        class="px-4 py-2 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm font-semibold disabled:opacity-60 disabled:cursor-not-allowed"
      >
        Reset Config
      </button>
      <button
        @click="onReboot"
        :disabled="deviceApi.loading"
        class="px-4 py-2 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm font-semibold disabled:opacity-60 disabled:cursor-not-allowed"
      >
        Reboot Device
      </button>
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-2">
      <div class="text-xs text-muted-foreground font-mono">Time: Synced</div>
    </div>
    <div v-if="deviceStore.loading" class="absolute inset-0 flex items-center justify-center bg-background/80 rounded-2xl z-10">
      <span class="text-muted-foreground">Loading...</span>
    </div>
    <FeedbackToast
      v-if="deviceStore.error || normalizedApiError"
      :message="deviceStore.error || normalizedApiError || ''"
      type="error"
      :show="!!(deviceStore.error || normalizedApiError)"
    />
  </div>
</template>

<script setup lang="ts">
import DeviceStatus from '../components/DeviceStatus.vue'
import FeedbackToast from '../components/FeedbackToast.vue'
import { useDeviceStore } from '../stores/deviceStore'
import { useDeviceApi } from '../composables/useDeviceApi'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

const deviceStore = useDeviceStore()
const deviceApi = useDeviceApi()
const router = useRouter()

const defaultLabels = Array.from({ length: 16 }, (_, i) => `Button ${i + 1}`)

const buttonLabels = computed(() => {
  const names = deviceStore.deviceConfig?.config?.button_names
  if (Array.isArray(names) && names.length === 16) return names
  if (names && typeof names === 'object') {
    return Array.from({ length: 16 }, (_, i) => names[i] || defaultLabels[i])
  }
  return defaultLabels
})

const deviceStatus = computed(() => deviceStore.deviceConfig ? 'Connected' : undefined)
const deviceWifi = computed(() => deviceStore.deviceConfig?.config?.settings?.wifi?.ssid ?? undefined)
const deviceTime = computed(() => deviceStore.deviceConfig?.config?.device_time ?? '-')

const normalizedApiError = computed(() => {
  const err = deviceApi.error
  if (!err) return undefined
  if (typeof err === 'string') return err
  return undefined
})

function goToMacroEditor(idx: number) {
  router.push({ name: 'edit-macro', params: { buttonIndex: idx } })
}

async function connectAndFetch() {
  const result = await deviceApi.connectToDevice()
  if (result.data) {
    await deviceStore.fetchConfig()
  }
}

async function onResetConfig() {
  const result = await deviceApi.resetConfig()
  if (result.data) {
    deviceStore.fetchConfig()
  }
}

async function onReboot() {
  await deviceApi.reboot()
}
</script> 