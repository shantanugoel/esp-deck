import { ref } from 'vue'

// Types for device config and responses (to be expanded)
type DeviceConfig = any

type ApiResult<T> = {
    data: T | null
    error: string | null
    loading: boolean
}

export function useDeviceApi() {
    const loading = ref(false)
    const error = ref<string | null>(null)

    // Mock: Fetch device config
    async function getConfig(): Promise<ApiResult<DeviceConfig>> {
        loading.value = true
        error.value = null
        try {
            // TODO: Replace with real device communication
            await new Promise(r => setTimeout(r, 500))
            return { data: { mock: true }, error: null, loading: false }
        } catch (e: any) {
            error.value = e.message
            return { data: null, error: e.message, loading: false }
        } finally {
            loading.value = false
        }
    }

    // Mock: Set device config
    async function setConfig(config: DeviceConfig): Promise<ApiResult<boolean>> {
        loading.value = true
        error.value = null
        try {
            await new Promise(r => setTimeout(r, 500))
            return { data: true, error: null, loading: false }
        } catch (e: any) {
            error.value = e.message
            return { data: false, error: e.message, loading: false }
        } finally {
            loading.value = false
        }
    }

    // Mock: Reset config
    async function resetConfig(): Promise<ApiResult<boolean>> {
        loading.value = true
        error.value = null
        try {
            await new Promise(r => setTimeout(r, 500))
            return { data: true, error: null, loading: false }
        } catch (e: any) {
            error.value = e.message
            return { data: false, error: e.message, loading: false }
        } finally {
            loading.value = false
        }
    }

    // Mock: Reboot device
    async function reboot(): Promise<ApiResult<boolean>> {
        loading.value = true
        error.value = null
        try {
            await new Promise(r => setTimeout(r, 500))
            return { data: true, error: null, loading: false }
        } catch (e: any) {
            error.value = e.message
            return { data: false, error: e.message, loading: false }
        } finally {
            loading.value = false
        }
    }

    return {
        loading,
        error,
        getConfig,
        setConfig,
        resetConfig,
        reboot,
    }
} 