import { useDeviceApi } from '@/composables/useDeviceApi';
import type { FullDeviceConfig, DeviceConnectionInfo, ProtocolResponse } from '@/types/protocol';

// --- Service Method Types (consistent with stores) ---
type ServiceResponse<T> = {
    success: boolean;
    data?: T;
    error?: string;
    status?: number; // Optional status, can map from device errors or keep for consistency
};

const PROTOCOL_VERSION_NUMBER = 0x00010000; // 65536 decimal, matches useDeviceApi and protocol.rs

// Helper to get a correlation ID, though useDeviceApi handles its own internal one.
// This service might not need to generate them if useDeviceApi does it internally for commands.
// For now, let's assume useDeviceApi's internal correlation ID is sufficient.

// Instantiate the composable to use its state and methods
// This might be better done within each function if its state is meant to be fresh per call,
// or once if it truly manages a global singleton state for the USB connection.
// Given useDeviceApi is structured with global refs (isDeviceConnected, etc.), using one instance is correct.
const deviceApi = useDeviceApi();

const connect = async (targetIp?: string /* targetIp is no longer relevant for USB */):
    Promise<ServiceResponse<DeviceConnectionInfo>> => {
    console.log('[deviceService] Attempting USB connection...');
    const result = await deviceApi.connectToDevice();
    if (result.data && !result.error) {
        console.log('[deviceService] USB Connection successful', result.data);
        return { success: true, data: result.data, status: 200 };
    }
    console.error('[deviceService] USB Connection failed:', result.error);
    return { success: false, error: result.error || 'Failed to connect to device', status: 503 };
};

const disconnect = async (): Promise<ServiceResponse<null>> => {
    console.log('[deviceService] Attempting USB disconnection...');
    const result = await deviceApi.disconnectDevice();
    if (result.data && !result.error) {
        console.log('[deviceService] USB Disconnection successful');
        return { success: true, status: 200 };
    }
    // If disconnectDevice indicated 'Device was not connected', it's still a success from this service's perspective.
    if (result.error === 'Device was not connected.') {
        console.log('[deviceService] Device was already disconnected.');
        return { success: true, status: 200 };
    }
    console.error('[deviceService] USB Disconnection failed:', result.error);
    return { success: false, error: result.error || 'Failed to disconnect device', status: 500 };
};

const fetchConfig = async (): Promise<ServiceResponse<FullDeviceConfig>> => {
    console.log('[deviceService] Fetching config via USB...');
    if (!deviceApi.isDeviceConnected) {
        return { success: false, error: 'Device not connected', status: 400 };
    }
    const result = await deviceApi.getConfig();
    if (result.data && !result.error) {
        console.log('[deviceService] Config fetched successfully via USB');
        return { success: true, data: result.data, status: 200 };
    }
    console.error('[deviceService] Fetch config via USB failed:', result.error);
    return { success: false, error: result.error || 'Failed to fetch config', status: 500 };
};

const saveConfig = async (configToSave: FullDeviceConfig): Promise<ServiceResponse<null>> => {
    console.log('[deviceService] Saving config via USB...');
    if (!deviceApi.isDeviceConnected) {
        return { success: false, error: 'Device not connected', status: 400 };
    }
    const result = await deviceApi.setConfig(configToSave);
    if (result.data && !result.error) {
        console.log('[deviceService] Config saved successfully via USB');
        return { success: true, status: 200 };
    }
    console.error('[deviceService] Save config via USB failed:', result.error);
    return { success: false, error: result.error || 'Failed to save config', status: 500 };
};

const resetConfig = async (): Promise<ServiceResponse<null>> => {
    console.log('[deviceService] Resetting config via USB...');
    if (!deviceApi.isDeviceConnected) {
        return { success: false, error: 'Device not connected', status: 400 };
    }
    const result = await deviceApi.resetConfig();
    if (result.data && !result.error) {
        console.log('[deviceService] Config reset successfully via USB');
        return { success: true, status: 200 };
    }
    console.error('[deviceService] Reset config via USB failed:', result.error);
    return { success: false, error: result.error || 'Failed to reset config', status: 500 };
};

const rebootDevice = async (): Promise<ServiceResponse<null>> => {
    console.log('[deviceService] Sending reboot command via USB...');
    if (!deviceApi.isDeviceConnected) {
        return { success: false, error: 'Device not connected', status: 400 };
    }
    const result = await deviceApi.reboot();
    // For reboot, success means the command was acknowledged.
    // The deviceApi.reboot() itself will set isDeviceConnected to false.
    if (result.data && !result.error) {
        console.log('[deviceService] Reboot command acknowledged via USB. Device should disconnect.');
        return { success: true, status: 200 };
    }
    console.error('[deviceService] Reboot command via USB failed:', result.error);
    return { success: false, error: result.error || 'Failed to send reboot command', status: 500 };
};

export const deviceService = {
    connect,
    disconnect,
    fetchConfig,
    saveConfig,
    resetConfig,
    rebootDevice,
    // Expose reactive properties from useDeviceApi if needed by stores directly
    // For example, for isConnected or isLoading, though stores usually manage their own granular loading states.
    // This can be useful for debug logs from useDeviceApi if the store wants to consolidate them.
    get isConnected() { return deviceApi.isDeviceConnected; },
    get isLoading() { return deviceApi.isLoading; }, // Reflects useDeviceApi's loading state
    get lastError() { return deviceApi.lastError; },
    get debugLogs() { return deviceApi.debugLogs; } // Expose debug logs from the API layer
}; 