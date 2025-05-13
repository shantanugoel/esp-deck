import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useDeviceApi } from '../composables/useDeviceApi'

// Types for device config (expand as needed)
type DeviceConfig = any

export const useDeviceStore = defineStore('device', () => {
    const deviceConfig = ref<DeviceConfig | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    const { getConfig, setConfig, resetConfig, reboot } = useDeviceApi()

    async function fetchConfig() {
        loading.value = true
        error.value = null
        const result = await getConfig()
        deviceConfig.value = result.data
        error.value = result.error
        loading.value = false
    }

    async function saveConfig(config: DeviceConfig) {
        loading.value = true
        error.value = null
        const result = await setConfig(config)
        error.value = result.error
        loading.value = false
        if (!result.error) deviceConfig.value = config
    }

    async function resetDeviceConfig() {
        loading.value = true
        error.value = null
        const result = await resetConfig()
        error.value = result.error
        loading.value = false
        if (!result.error) await fetchConfig()
    }

    async function rebootDevice() {
        loading.value = true
        error.value = null
        const result = await reboot()
        error.value = result.error
        loading.value = false
    }

    return {
        deviceConfig,
        loading,
        error,
        fetchConfig,
        saveConfig,
        resetDeviceConfig,
        rebootDevice,
    }
}) 