import { reactive, ref } from 'vue'

// Types for device config and responses (to be expanded)
type DeviceConfig = any

type ApiResult<T> = {
    data: T | null
    error: string | null
    loading: boolean
}

const VID = 0x5AA6
const PID = 0x60E1
const INTERFACE = 1
const ENDPOINT_OUT = 2
const ENDPOINT_IN = 2
const MAGIC_WORD = 0xE59DECC0

let device: any = null // USBDevice | null

// Singleton refs
const isConnected = ref(false)
const loading = ref(false)
const error = ref<string | null>(null)

async function connectToDevice(): Promise<ApiResult<boolean>> {
    loading.value = true
    error.value = null
    try {
        // @ts-ignore
        device = await navigator.usb.requestDevice({ filters: [{ vendorId: VID, productId: PID }] })
        if (!device) throw new Error('No device selected')
        await device.open()
        if (device.configuration === null) await device.selectConfiguration(1)
        await device.claimInterface(INTERFACE)
        isConnected.value = true
        return { data: true, error: null, loading: false }
    } catch (e: any) {
        error.value = e.message
        isConnected.value = false
        return { data: false, error: e.message, loading: false }
    } finally {
        loading.value = false
    }
}

function encodeCommand(payload: string): Uint8Array {
    const header = new Uint8Array(4)
    new DataView(header.buffer).setUint32(0, MAGIC_WORD, false)
    const length = new Uint8Array(4)
    new DataView(length.buffer).setUint32(0, payload.length, false)
    const body = new TextEncoder().encode(payload)
    const result = new Uint8Array(header.length + length.length + body.length)
    result.set(header, 0)
    result.set(length, header.length)
    result.set(body, header.length + length.length)
    return result
}

async function sendCommand(payload: string): Promise<string> {
    if (!device) throw new Error('Device not connected')
    const data = encodeCommand(payload)
    await device.transferOut(ENDPOINT_OUT, data)

    // Accumulate incoming data until a full response is received
    let receiveBuffer = new Uint8Array(0)
    let expectedPayloadLength: number | null = null
    let timeoutMs = 2000
    let start = Date.now()

    while (true) {
        // Timeout check
        if (Date.now() - start > timeoutMs) {
            throw new Error('Timeout waiting for device response')
        }
        const result = await device.transferIn(ENDPOINT_IN, 64)
        if (!result.data || result.data.byteLength === 0) continue
        // Append new chunk
        const newChunk = new Uint8Array(result.data.buffer)
        const combined = new Uint8Array(receiveBuffer.length + newChunk.length)
        combined.set(receiveBuffer, 0)
        combined.set(newChunk, receiveBuffer.length)
        receiveBuffer = combined

        // Wait for at least header + length
        if (receiveBuffer.length < 8) continue
        const view = new DataView(receiveBuffer.buffer, receiveBuffer.byteOffset, receiveBuffer.byteLength)
        const magic = view.getUint32(0, true) // big-endian
        if (magic !== 0xE59DECC0) throw new Error('Invalid response magic word')
        const len = view.getUint32(4, true) // little-endian
        expectedPayloadLength = len
        // Wait for full payload
        if (receiveBuffer.length < 8 + expectedPayloadLength) continue
        // Extract payload
        const payloadBytes = receiveBuffer.slice(8, 8 + expectedPayloadLength)
        return new TextDecoder().decode(payloadBytes)
    }
}

async function getConfig(): Promise<ApiResult<DeviceConfig>> {
    loading.value = true
    error.value = null
    try {
        if (!isConnected.value) throw new Error('Device not connected')
        const cmd = JSON.stringify({ type: 'GetConfig', header: { version: 65536 } })
        const resp = await sendCommand(cmd)
        return { data: JSON.parse(resp), error: null, loading: false }
    } catch (e: any) {
        error.value = e.message
        return { data: null, error: e.message, loading: false }
    } finally {
        loading.value = false
    }
}

async function setConfig(config: DeviceConfig): Promise<ApiResult<boolean>> {
    loading.value = true
    error.value = null
    try {
        if (!isConnected.value) throw new Error('Device not connected')
        const cmd = JSON.stringify({ type: 'SetConfig', header: { version: 65536 }, config })
        await sendCommand(cmd)
        return { data: true, error: null, loading: false }
    } catch (e: any) {
        error.value = e.message
        return { data: false, error: e.message, loading: false }
    } finally {
        loading.value = false
    }
}

async function resetConfig(): Promise<ApiResult<boolean>> {
    loading.value = true
    error.value = null
    try {
        if (!isConnected.value) throw new Error('Device not connected')
        const cmd = JSON.stringify({ type: 'ResetConfig', header: { version: 65536 } })
        await sendCommand(cmd)
        return { data: true, error: null, loading: false }
    } catch (e: any) {
        error.value = e.message
        return { data: false, error: e.message, loading: false }
    } finally {
        loading.value = false
    }
}

async function reboot(): Promise<ApiResult<boolean>> {
    loading.value = true
    error.value = null
    try {
        if (!isConnected.value) throw new Error('Device not connected')
        const cmd = JSON.stringify({ type: 'Reboot', header: { version: 65536 } })
        await sendCommand(cmd)
        return { data: true, error: null, loading: false }
    } catch (e: any) {
        error.value = e.message
        return { data: false, error: e.message, loading: false }
    } finally {
        loading.value = false
    }
}

export function useDeviceApi() {
    return reactive({
        loading,
        error,
        isConnected,
        connectToDevice,
        getConfig,
        setConfig,
        resetConfig,
        reboot,
    })
} 