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
          <div v-for="(label, idx) in buttonLabelsWithDirty" :key="idx" class="flex flex-col items-center">
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
    <div v-if="deviceApi.isConnected" class="w-full max-w-4xl flex flex-col items-center mt-4">
      <div class="w-full bg-muted/40 rounded-lg p-4 flex flex-col gap-4 border border-muted">
        <div class="font-semibold text-base mb-2">Device Settings</div>
        <div class="flex flex-col sm:flex-row gap-4">
          <div class="flex-1 flex flex-col gap-2">
            <label class="text-sm font-medium">WiFi SSID</label>
            <div class="flex items-center gap-2">
              <template v-if="!isEditingSsid">
                <span>{{ tempSsid !== wifiSsid ? tempSsid : (wifiSsid || '-') }}</span>
                <span class="ml-1 cursor-pointer text-muted-foreground hover:text-primary" @click="startEditSsid" title="Edit SSID" tabindex="0" role="button" aria-label="Edit SSID">✏️</span>
              </template>
              <template v-else>
                <input ref="ssidInputRef" v-model="tempSsid" type="text" class="border rounded px-3 py-2 bg-background text-foreground w-32" maxlength="32" placeholder="WiFi SSID"
                  @keyup.enter="stopEditSsid" @blur="stopEditSsid" />
              </template>
            </div>
          </div>
          <div class="flex-1 flex flex-col gap-2">
            <label class="text-sm font-medium">WiFi Password</label>
            <div class="flex items-center gap-2">
              <template v-if="!isEditingPassword">
                <span>{{ tempPassword !== wifiPassword ? (tempPassword ? '••••••••' : '-') : (wifiPassword ? '••••••••' : '-') }}</span>
                <span class="ml-1 cursor-pointer text-muted-foreground hover:text-primary" @click="startEditPassword" title="Edit Password" tabindex="0" role="button" aria-label="Edit Password">✏️</span>
              </template>
              <template v-else>
                <form @submit.prevent="stopEditPassword" class="inline flex items-center gap-2">
                  <input
                    ref="passwordInputRef"
                    v-model="tempPassword"
                    :type="showPassword ? 'text' : 'password'"
                    class="border rounded px-3 py-2 bg-background text-foreground w-32"
                    maxlength="64"
                    placeholder="WiFi Password"
                    @blur="stopEditPassword"
                  />
                  <button
                    type="button"
                    @mousedown.prevent.stop
                    @click.prevent.stop="showPassword = !showPassword"
                    tabindex="0"
                    aria-label="Toggle password visibility"
                    class="ml-1 text-muted-foreground hover:text-primary focus:outline-none"
                  >
                    {{ showPassword ? '🙈' : '👁️' }}
                  </button>
                </form>
              </template>
            </div>
          </div>
          <div class="flex-1 flex flex-col gap-2">
            <label class="text-sm font-medium">Timezone Offset</label>
            <div class="flex items-center gap-2">
              <template v-if="!isEditingTz">
                <span>{{ tempTz !== timezoneOffset ? (tempTz ?? '-') : (timezoneOffset !== null && timezoneOffset !== undefined ? timezoneOffset : '-') }}</span>
                <span class="ml-1 cursor-pointer text-muted-foreground hover:text-primary" @click="startEditTz" title="Edit Timezone Offset" tabindex="0" role="button" aria-label="Edit Timezone Offset">✏️</span>
              </template>
              <template v-else>
                <input ref="tzInputRef" v-model.number="tempTz" type="number" step="0.01" class="border rounded px-3 py-2 bg-background text-foreground w-24" placeholder="e.g. -7.0, 5.5"
                  @keyup.enter="stopEditTz" @blur="stopEditTz" />
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="deviceApi.isConnected" class="w-full max-w-4xl flex justify-center mt-4">
      <div class="flex flex-col sm:flex-row gap-2">
        <button
          @click="onSaveSettings"
          :disabled="!hasSettingsChanged || deviceStore.loading"
          class="px-4 py-2 rounded bg-primary text-primary-foreground font-semibold shadow hover:bg-primary/80 transition disabled:opacity-60 disabled:cursor-not-allowed"
          title="Save all pending changes (WiFi, timezone, macros, button names) to the device."
        >
          Save Settings
        </button>
        <button
          @click="deviceStore.fetchConfig"
          :disabled="deviceStore.loading || !deviceApi.isConnected"
          class="px-4 py-2 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm font-semibold flex items-center gap-1 disabled:opacity-60 disabled:cursor-not-allowed"
          title="Reload the latest config from the device, discarding any unsaved changes."
        >
          Reload Settings
        </button>
        <button
          @click="onResetConfig"
          :disabled="deviceApi.loading"
          class="px-4 py-2 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm font-semibold disabled:opacity-60 disabled:cursor-not-allowed"
          title="Reset the device config to factory defaults."
        >
          Reset Settings
        </button>
        <button
          @click="onReboot"
          :disabled="deviceApi.loading"
          class="px-4 py-2 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm font-semibold disabled:opacity-60 disabled:cursor-not-allowed"
          title="Reboot the device."
        >
          Reboot Device
        </button>
        <button
          @click="onBackupDeviceConfig"
          :disabled="deviceStore.loading || !deviceApi.isConnected"
          class="px-4 py-2 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm font-semibold disabled:opacity-60 disabled:cursor-not-allowed"
          title="Download the latest config from the device as a JSON backup."
        >
          Backup Device Config
        </button>
        <button
          @click="onBackupCurrentConfig"
          :disabled="!deviceStore.deviceConfig"
          class="px-4 py-2 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm font-semibold disabled:opacity-60 disabled:cursor-not-allowed"
          title="Download the currently loaded config (including unsaved changes) as a JSON backup."
        >
          Backup Current Config
        </button>
      </div>
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-4">
      <DeviceStatus :status="deviceStatus" />
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-2">
      <button @click="toggleDebug" class="px-3 py-1 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-xs font-semibold">
        {{ showDebug ? 'Hide Debug Logs' : 'Show Debug Logs' }}
      </button>
    </div>
    <div v-if="showDebug" class="w-full max-w-4xl mx-auto bg-background border border-muted rounded-lg mt-2 p-4 overflow-x-auto max-h-96">
      <div class="font-semibold mb-2">Debug Logs</div>
      <div class="text-xs text-muted-foreground mb-2">Sent and received data with timestamps.</div>
      <div class="space-y-2 max-h-72 overflow-y-auto font-mono">
        <template v-if="deviceApi.debugLogs.length">
          <div v-for="(log, idx) in deviceApi.debugLogs" :key="idx" :class="log.type === 'sent' ? 'text-blue-700' : 'text-green-700'">
            <div>
              <span class="font-bold">[{{ log.type.toUpperCase() }}]</span>
              <span class="ml-2 text-gray-500">{{ log.timestamp.toLocaleString() }}</span>
            </div>
            <pre class="whitespace-pre-wrap break-all bg-muted/40 rounded p-2 mt-1">{{ log.data }}</pre>
          </div>
        </template>
        <template v-else>
          <div class="text-center text-muted-foreground py-8">No logs yet.</div>
        </template>
      </div>
    </div>
    <div v-if="deviceStore.loading" class="absolute left-0 right-0 top-0 flex items-center justify-center bg-background/80 rounded-2xl z-10" style="min-height: 200px;">
      <span class="text-muted-foreground">Loading...</span>
    </div>
    <FeedbackToast
      v-if="deviceStore.error || normalizedApiError"
      :message="deviceStore.error || normalizedApiError || ''"
      type="error"
      :show="!!(deviceStore.error || normalizedApiError)"
    />
    <SaveSettingsModal
      v-if="showSaveModal"
      :changedWifi="changedWifi"
      :changedTz="changedTz"
      :changedMappings="changedMappings"
      @confirm="handleSaveModalConfirm"
      @cancel="handleSaveModalCancel"
    />
  </div>
</template>

<script setup lang="ts">
import DeviceStatus from '../components/DeviceStatus.vue'
import FeedbackToast from '../components/FeedbackToast.vue'
import { useDeviceStore } from '../stores/deviceStore'
import { useDeviceApi } from '../composables/useDeviceApi'
import { computed, ref, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import SaveSettingsModal from '../components/SaveSettingsModal.vue'

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

// Show * for dirty buttons
const buttonLabelsWithDirty = computed(() =>
  buttonLabels.value.map((label, idx) => deviceStore.isButtonDirty(idx) ? `${label} *` : label)
)

const deviceStatus = computed(() => deviceStore.deviceConfig ? 'Connected' : undefined)

const normalizedApiError = computed(() => {
  const err = deviceApi.error
  if (!err) return undefined
  if (typeof err === 'string') return err
  return undefined
})

const wifiSsid = ref('')
const wifiPassword = ref('')
const timezoneOffset = ref<number | null>(null)

const isEditingSsid = ref(false)
const isEditingPassword = ref(false)
const isEditingTz = ref(false)
const tempSsid = ref('')
const tempPassword = ref('')
const tempTz = ref<number | null>(null)
const ssidInputRef = ref<HTMLInputElement | null>(null)
const passwordInputRef = ref<HTMLInputElement | null>(null)
const tzInputRef = ref<HTMLInputElement | null>(null)
const showPassword = ref(false)

const showSaveModal = ref(false)
const saveModalSelection = ref<{ wifi: boolean, tz: boolean, mappings: boolean, sendAllMappings?: boolean } | null>(null)

// Track previous values for comparison
const prevWifiSsid = computed(() => wifiSsid.value)
const prevWifiPassword = computed(() => wifiPassword.value)
const prevTimezoneOffset = computed(() => timezoneOffset.value)

// For button mappings, assume deviceStore.deviceConfig?.config?.button_mappings is the source
const prevButtonMappings = computed(() => deviceStore.deviceConfig?.config?.button_mappings || {})
// Assume tempButtonMappings is available if you support editing mappings in this view; otherwise, use prevButtonMappings
const tempButtonMappings = prevButtonMappings // placeholder, replace with actual temp if needed

const changedWifi = computed(() => {
  const changes: any = {}
  if (tempSsid.value !== prevWifiSsid.value) changes.ssid = { old: prevWifiSsid.value, new: tempSsid.value }
  if (tempPassword.value !== prevWifiPassword.value) changes.password = true
  return Object.keys(changes).length ? changes : null
})
const changedTz = computed(() => {
  if (tempTz.value !== prevTimezoneOffset.value) {
    return { old: prevTimezoneOffset.value, new: tempTz.value }
  }
  return null
})
const changedMappings = computed(() => {
  // Compare keys and values for changed mappings
  // This is a placeholder; replace with actual logic if you have temp mappings
  return []
})

watch(
  () => deviceStore.deviceConfig,
  (config) => {
    const wifi = config?.config?.settings?.wifi
    wifiSsid.value = wifi?.ssid || ''
    wifiPassword.value = wifi?.password || ''
    timezoneOffset.value = config?.config?.settings?.timezone_offset ?? null
    // Reset edit state and temp values on config change
    isEditingSsid.value = false
    isEditingPassword.value = false
    isEditingTz.value = false
    tempSsid.value = wifiSsid.value
    tempPassword.value = wifiPassword.value
    tempTz.value = timezoneOffset.value
  },
  { immediate: true }
)

const hasSettingsChanged = computed(() =>
  tempSsid.value !== wifiSsid.value ||
  tempPassword.value !== wifiPassword.value ||
  tempTz.value !== timezoneOffset.value ||
  Object.keys(deviceStore.stagedButtonChanges).length > 0
)

function startEditSsid() {
  isEditingSsid.value = true
  nextTick(() => ssidInputRef.value?.focus())
}
function stopEditSsid() {
  isEditingSsid.value = false
}
function startEditPassword() {
  isEditingPassword.value = true
  nextTick(() => passwordInputRef.value?.focus())
}
function stopEditPassword() {
  isEditingPassword.value = false
}
function startEditTz() {
  isEditingTz.value = true
  nextTick(() => tzInputRef.value?.focus())
}
function stopEditTz() {
  isEditingTz.value = false
}

function onSaveSettings() {
  showSaveModal.value = true
}

function handleSaveModalConfirm(selection: { wifi: boolean, tz: boolean, mappings: boolean, sendAllMappings?: boolean }) {
  showSaveModal.value = false
  // Merge staged changes into config before saving
  const config = JSON.parse(JSON.stringify(deviceStore.deviceConfig?.config || {}))
  // Apply staged button changes
  if (selection.mappings) {
    if (selection.sendAllMappings) {
      // Send all mappings and button names
      // (config already has all current mappings and names)
    } else {
      // Only send changed mappings and names
      // Remove unchanged mappings and names from config
      const newMappings: Record<string, any[]> = {}
      const newButtonNames: Record<number, string> = {}
      for (let idx = 0; idx < 16; idx++) {
        const stagedMacro = deviceStore.getStagedButtonMacro(idx)
        const stagedName = deviceStore.getStagedButtonName(idx)
        if (stagedMacro) newMappings[String(idx + 1)] = stagedMacro
        if (stagedName) newButtonNames[idx] = stagedName
      }
      config.mappings = newMappings
      config.button_names = Object.keys(newButtonNames).length ? newButtonNames : undefined
    }
  } else {
    config.mappings = {}
  }
  // Always include settings if any part is selected
  if (selection.wifi || selection.tz) {
    config.settings = config.settings || {}
    if (selection.wifi && changedWifi.value) {
      config.settings.wifi = {
        ssid: tempSsid.value,
        password: tempPassword.value,
      }
    }
    if (selection.tz && changedTz.value) {
      config.settings.timezone_offset = tempTz.value
    }
  }
  saveSettingsPayload(config)
}

function handleSaveModalCancel() {
  showSaveModal.value = false
}

async function saveSettingsPayload(config: any) {
  await deviceStore.saveConfig(config)
  await deviceStore.fetchConfig()
  deviceStore.clearStagedChanges()
}

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

const showDebug = ref(false)
function toggleDebug() { showDebug.value = !showDebug.value }

function downloadJson(obj: any, filename: string) {
  const blob = new Blob([JSON.stringify(obj, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  setTimeout(() => {
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, 100)
}

async function onBackupDeviceConfig() {
  // Fetch latest config from device
  const result = await deviceApi.getConfig()
  if (result.data && result.data.config) {
    const ts = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 15)
    downloadJson(result.data.config, `device-config-backup-${ts}.json`)
  }
}

function onBackupCurrentConfig() {
  if (deviceStore.deviceConfig && deviceStore.deviceConfig.config) {
    const ts = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 15)
    downloadJson(deviceStore.deviceConfig.config, `current-config-backup-${ts}.json`)
  }
}
</script> 