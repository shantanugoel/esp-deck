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
      <ButtonGrid :button-labels="buttonLabels" @edit="handleEditButton" />
      <ButtonEditModal
        :open="isEditModalOpen"
        :button-index="selectedButtonIndex ?? 0"
        :button-data="selectedButtonData"
        @save="handleModalSave"
        @close="handleModalClose"
      />
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-4">
      <DeviceStatus
        :status="deviceStatus"
        :wifi="deviceWifi"
        :time="deviceTime" />
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
import ButtonGrid from '../components/ButtonGrid.vue'
import ButtonEditModal from '../components/ButtonEditModal.vue'
import DeviceStatus from '../components/DeviceStatus.vue'
import FeedbackToast from '../components/FeedbackToast.vue'
import { useDeviceStore } from '../stores/deviceStore'
import { useDeviceApi } from '../composables/useDeviceApi'
import { computed, ref } from 'vue'

const deviceStore = useDeviceStore()
const deviceApi = useDeviceApi()

const defaultLabels = Array.from({ length: 16 }, (_, i) => `Button ${i + 1}`)

const buttonLabels = computed(() => {
  const names = deviceStore.deviceConfig?.config?.button_names
  console.log('[DEBUG] buttonLabels computed, deviceConfig:', deviceStore.deviceConfig)
  if (Array.isArray(names) && names.length === 16) return names
  if (names && typeof names === 'object') {
    // If button_names is an object (from backend), map to array
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

// --- Button Edit Modal State ---
const isEditModalOpen = ref(false)
const selectedButtonIndex = ref<number | null>(null)
const selectedButtonData = ref({ name: '', actionType: 'keyboard', actionDetail: '' })

function handleEditButton(idx: number) {
  selectedButtonIndex.value = idx
  const config = deviceStore.deviceConfig?.config
  const buttonNames = config?.button_names || {}
  const mappings = config?.mappings || {}
  const mappingKey = String(idx + 1)
  const actions = mappings[mappingKey] || []

  let actionType = 'keyboard'
  let actionDetail = ''
  // If there are multiple actions or a Sequence, treat as macro
  if (actions.length > 1 || (actions.length > 0 && actions[0].Sequence)) {
    actionType = 'macro'
    actionDetail = JSON.stringify(actions, null, 2)
  } else if (actions.length > 0) {
    const first = actions[0]
    if (first.KeyPress) {
      actionType = 'keyboard'
      actionDetail = first.KeyPress.key + (first.KeyPress.modifier ? ` + ${first.KeyPress.modifier}` : '')
    } else if (first.MousePress) {
      actionType = 'mouse'
      actionDetail = String(first.MousePress.button)
    } else if (first.ConsumerPress) {
      actionType = 'media'
      actionDetail = String(first.ConsumerPress.usage_id)
    } else {
      actionType = 'macro'
      actionDetail = JSON.stringify(actions, null, 2)
    }
  }
  selectedButtonData.value = {
    name: buttonNames[idx] || defaultLabels[idx],
    actionType,
    actionDetail
  }
  isEditModalOpen.value = true
}

function handleModalSave(data: { name: string; actionType: string; actionDetail: string }) {
  if (selectedButtonIndex.value == null) return
  const config = deviceStore.deviceConfig?.config
  if (config) {
    if (!config.button_names) config.button_names = {}
    config.button_names[selectedButtonIndex.value] = data.name
    if (!config.mappings) config.mappings = {}
    const mappingKey = String(selectedButtonIndex.value + 1)
    let actions: any[] = []
    if (data.actionType === 'macro') {
      // Raw JSON for macro sequence
      try {
        const seq = JSON.parse(data.actionDetail)
        actions = Array.isArray(seq) ? seq : []
      } catch {
        actions = []
      }
    } else if (data.actionType === 'keyboard') {
      const [key, mod] = data.actionDetail.split(' + ').map(s => s.trim())
      actions = [{ KeyPress: { key, modifier: mod || undefined } }, { Delay: { ms: 10 } }, 'KeyRelease']
    } else if (data.actionType === 'mouse') {
      const button = parseInt(data.actionDetail, 10) || 1
      actions = [{ MousePress: { button } }, { Delay: { ms: 10 } }, 'MouseRelease']
    } else if (data.actionType === 'media') {
      const usage_id = parseInt(data.actionDetail, 16) || 0
      actions = [{ ConsumerPress: { usage_id } }, { Delay: { ms: 10 } }, 'ConsumerRelease']
    }
    config.mappings[mappingKey] = actions
    deviceStore.saveConfig({ ...config })
  }
  isEditModalOpen.value = false
}

function handleModalClose() {
  isEditModalOpen.value = false
}

async function connectAndFetch() {
  const result = await deviceApi.connectToDevice()
  console.log('[DEBUG] connectAndFetch result:', result)
  if (result.data) {
    await deviceStore.fetchConfig()
    console.log('[DEBUG] after fetchConfig, deviceConfig:', deviceStore.deviceConfig)
  }
}
</script> 