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
    <div v-if="deviceApi.isConnected" class="w-full max-w-4xl flex flex-col items-center mt-6">
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
        <div class="flex justify-end mt-4">
          <button
            @click="onSaveSettings"
            :disabled="!hasSettingsChanged || deviceStore.loading"
            class="px-4 py-2 rounded bg-primary text-primary-foreground font-semibold shadow hover:bg-primary/80 transition disabled:opacity-60 disabled:cursor-not-allowed"
          >
            Save Settings
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import DeviceStatus from '../components/DeviceStatus.vue'
import FeedbackToast from '../components/FeedbackToast.vue'
import { useDeviceStore } from '../stores/deviceStore'
import { useDeviceApi } from '../composables/useDeviceApi'
import { computed, ref, watch, nextTick } from 'vue'
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
  tempTz.value !== timezoneOffset.value
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

async function onSaveSettings() {
  await saveSettings(tempSsid.value, tempPassword.value, tempTz.value)
  // After save, temp values will be reset by watcher
}

async function saveSettings(ssid: string, password: string, tz: number | null) {
  const current = deviceStore.deviceConfig?.config || {}
  const newConfig = {
    ...current,
    settings: {
      ...current.settings,
      wifi: {
        ssid,
        password,
      },
      timezone_offset: tz,
    },
  }
  await deviceStore.saveConfig(newConfig)
  await deviceStore.fetchConfig()
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
</script> 