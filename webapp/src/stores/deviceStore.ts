import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useDeviceApi } from '../composables/useDeviceApi'

// Types for device config (expand as needed)
type DeviceConfig = any

type StagedButtonChange = {
    macro?: any[],
    name?: string,
}

export const useDeviceStore = defineStore('device', () => {
    const deviceConfig = ref<DeviceConfig | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    // Track staged (unsaved) changes per button index
    const stagedButtonChanges = ref<Record<number, StagedButtonChange>>({})

    const { getConfig, setConfig, resetConfig, reboot } = useDeviceApi()

    async function fetchConfig() {
        loading.value = true
        error.value = null
        const result = await getConfig()
        deviceConfig.value = result.data
        error.value = result.error
        loading.value = false
        stagedButtonChanges.value = {} // clear staged changes on fetch
    }

    async function saveConfig(config: DeviceConfig) {
        loading.value = true
        error.value = null
        const result = await setConfig(config)
        error.value = result.error
        loading.value = false
        if (!result.error) {
            deviceConfig.value = config
            stagedButtonChanges.value = {} // clear staged changes on save
        }
    }

    function stageButtonMacro(idx: number, macro: any[]) {
        if (!stagedButtonChanges.value[idx]) stagedButtonChanges.value[idx] = {}
        stagedButtonChanges.value[idx].macro = macro
    }
    function stageButtonName(idx: number, name: string) {
        if (!stagedButtonChanges.value[idx]) stagedButtonChanges.value[idx] = {}
        stagedButtonChanges.value[idx].name = name
    }
    function isButtonDirty(idx: number) {
        return !!stagedButtonChanges.value[idx]
    }
    function getStagedButtonMacro(idx: number): any[] | undefined {
        return stagedButtonChanges.value[idx]?.macro
    }
    function getStagedButtonName(idx: number): string | undefined {
        return stagedButtonChanges.value[idx]?.name
    }
    function clearStagedChanges() {
        stagedButtonChanges.value = {}
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
        deviceConfig: computed(() => deviceConfig.value),
        loading: computed(() => loading.value),
        error: computed(() => error.value),
        fetchConfig,
        saveConfig,
        resetDeviceConfig,
        rebootDevice,
        // staged changes API
        stagedButtonChanges,
        stageButtonMacro,
        stageButtonName,
        isButtonDirty,
        getStagedButtonMacro,
        getStagedButtonName,
        clearStagedChanges,
    }
}) 