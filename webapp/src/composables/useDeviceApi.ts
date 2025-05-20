// Placeholder types for WebUSB if @types/webusb is not installed
// Ideally, install @types/webusb for proper type checking
declare global {
    interface USBDevice {
        readonly productName?: string;
        readonly serialNumber?: string;
        readonly configuration: USBConfiguration | null;
        open(): Promise<void>;
        selectConfiguration(configurationValue: number): Promise<void>;
        claimInterface(interfaceNumber: number): Promise<void>;
        releaseInterface(interfaceNumber: number): Promise<void>;
        close(): Promise<void>;
        transferOut(endpointNumber: number, data: BufferSource): Promise<USBOutTransferResult>;
        transferIn(endpointNumber: number, length: number): Promise<USBInTransferResult>;
        clearHalt(direction: USBDirection, endpointNumber: number): Promise<void>;
        // Add other methods/properties if used and not covered
    }

    interface USBConfiguration {
        // Add properties if needed
    }

    // interface USBInterface {
    //     // Add properties if needed
    // }

    interface USBOutTransferResult {
        readonly bytesWritten: number;
        readonly status: USBTransferStatus;
    }

    interface USBInTransferResult {
        readonly data?: DataView;
        readonly status: USBTransferStatus;
    }

    type USBDirection = "in" | "out";
    type USBTransferStatus = "ok" | "stall" | "babble";

    interface Navigator {
        readonly usb: USB;
    }
    interface USB {
        requestDevice(options?: USBDeviceRequestOptions): Promise<USBDevice>;
        // getDevices(): Promise<USBDevice[]>; // If you use it
    }
    interface USBDeviceRequestOptions {
        filters: USBDeviceFilter[];
    }
    interface USBDeviceFilter {
        vendorId?: number;
        productId?: number;
        classCode?: number;
        subclassCode?: number;
        protocolCode?: number;
        serialNumber?: string;
    }
}
// End of Placeholder types

import { reactive, ref } from 'vue'
import type { FullDeviceConfig, Command, ProtocolHeader, GetConfigCommand, SetConfigCommand, ResetConfigCommand, RebootCommand, DeviceConnectionInfo } from '@/types/protocol';

// Type for ApiResult used internally in this composable
type ApiResult<T> = {
    data: T | null;
    error: string | null;
    loading: boolean; // Specific to this composable's async operations
    // status?: number; // Optional: if we want to relay a status code like HTTP status
};

const VID = 0x5AA6;
const PID = 0x60E1;
const INTERFACE_NUMBER = 1; // Renamed for clarity
const ENDPOINT_OUT = 2;
const ENDPOINT_IN = 2;

// This is the protocol version defined in Rust: 0x00010000 (Major 1, Minor 0)
const PROTOCOL_VERSION = 0x00010000; // 65536 decimal

let usbDevice: USBDevice | null = null; // Renamed for clarity

// Singleton refs managed by this composable
const isDeviceConnected = ref(false); // Renamed for clarity
const isLoading = ref(false); // Renamed for clarity
const lastError = ref<string | null>(null); // Renamed for clarity
const internalDebugLogs = ref<{ type: 'sent' | 'received', data: string, timestamp: Date }[]>([]);

let correlationCounter = 1;
function getNextCorrelationId(): number {
    return correlationCounter++;
}

function addDebugLog(type: 'sent' | 'received', data: string) {
    internalDebugLogs.value.unshift({ type, data, timestamp: new Date() });
    if (internalDebugLogs.value.length > 50) internalDebugLogs.value.pop(); // Keep logs bounded
}

async function connectToDevice(): Promise<ApiResult<DeviceConnectionInfo>> {
    if (isDeviceConnected.value) {
        // If already connected, perhaps return current device info or a specific message
        // For now, let's assume we try to re-verify or just return success with existing info
        if (usbDevice) {
            const deviceInfo: DeviceConnectionInfo = {
                productName: usbDevice.productName,
                serialNumber: usbDevice.serialNumber,
                firmwareVersion: `0x${PROTOCOL_VERSION.toString(16)}` // Example, could be part of a handshake later
            };
            return { data: deviceInfo, error: null, loading: false };
        }
    }

    isLoading.value = true;
    lastError.value = null;
    try {
        // @ts-ignore: navigator.usb is standard but might not be in all TS lib versions by default
        const requestedDevice = await navigator.usb.requestDevice({ filters: [{ vendorId: VID, productId: PID }] });
        if (!requestedDevice) throw new Error('No device selected by user.');
        usbDevice = requestedDevice;

        await usbDevice.open();
        if (usbDevice.configuration === null) await usbDevice.selectConfiguration(1);
        await usbDevice.claimInterface(INTERFACE_NUMBER);
        isDeviceConnected.value = true;

        const deviceInfo: DeviceConnectionInfo = {
            productName: usbDevice.productName,
            serialNumber: usbDevice.serialNumber,
            firmwareVersion: `0x${PROTOCOL_VERSION.toString(16)}`
        };
        return { data: deviceInfo, error: null, loading: false };
    } catch (e: any) {
        lastError.value = e.message || 'Unknown error during device connection.';
        isDeviceConnected.value = false;
        usbDevice = null; // Ensure device is null on error
        return { data: null, error: lastError.value, loading: false };
    } finally {
        isLoading.value = false;
    }
}

async function disconnectDevice(): Promise<ApiResult<boolean>> {
    isLoading.value = true;
    lastError.value = null;
    if (!usbDevice || !isDeviceConnected.value) {
        isDeviceConnected.value = false; // Ensure state consistency
        usbDevice = null;
        return { data: true, error: 'Device was not connected.', loading: false }; // Not an error per se, but a state
    }
    try {
        // Note: Order might matter, and error handling for each step could be added.
        await usbDevice.releaseInterface(INTERFACE_NUMBER);
        await usbDevice.close();
        isDeviceConnected.value = false;
        usbDevice = null;
        return { data: true, error: null, loading: false };
    } catch (e: any) {
        lastError.value = e.message || 'Error during device disconnection.';
        // Even on error, we consider it disconnected from our perspective
        isDeviceConnected.value = false;
        usbDevice = null;
        return { data: false, error: lastError.value, loading: false };
    } finally {
        isLoading.value = false;
    }
}


// Framing logic - MAGIC_WORD is not used in current Rust protocol.rs for response sending.
// The device sends raw JSON. This framing seems to be for sending commands TO the device.
// If device expects this framing, it needs to be implemented on its side to read it.
// For now, assuming this send framing is correct as per original `useDeviceApi`.
// The receive logic, however, might simplify if device just sends JSON without this framing.
// The current receive logic tries to parse this frame from device - this is a mismatch with protocol.rs.
// For now, this will be MODIFIED to read raw JSON from device.

async function sendCommandAndGetResponse(command: Command): Promise<string> {
    if (!usbDevice || !isDeviceConnected.value) throw new Error('Device not connected');

    const commandString = JSON.stringify(command);
    addDebugLog('sent', commandString);

    // Send data using WebUSB transferOut
    // MODIFIED: Send raw JSON string, as protocol.rs uses serde_json::from_slice directly.
    // The custom MAGIC_WORD framing for sending has been removed.
    const dataToSend = new TextEncoder().encode(commandString);
    await usbDevice.transferOut(ENDPOINT_OUT, dataToSend);

    // Receive data using WebUSB transferIn
    // Assumes device sends raw JSON as per protocol.rs `serde_json::to_vec(&response)`
    let receiveBuffer = '';
    const timeoutMs = 5000; // Increased timeout
    const startTime = Date.now();

    while (true) {
        if (Date.now() - startTime > timeoutMs) {
            throw new Error('Timeout waiting for device response.');
        }

        const result = await usbDevice.transferIn(ENDPOINT_IN, 2048); // Read larger chunks
        if (result.status === 'stall') {
            await usbDevice.clearHalt('in', ENDPOINT_IN);
            continue;
        }
        if (!result.data || result.data.byteLength === 0) {
            await new Promise(resolve => setTimeout(resolve, 50)); // Small delay before retrying
            continue;
        }

        receiveBuffer += new TextDecoder().decode(result.data);

        try {
            // Try to parse the buffer as JSON. If it succeeds, we have the full message.
            JSON.parse(receiveBuffer); // This will throw if not complete JSON
            addDebugLog('received', receiveBuffer);
            return receiveBuffer;
        } catch (e) {
            // JSON is not complete yet, continue accumulating
            if (receiveBuffer.length > 10 * 1024 * 1024) { // Safety break for too large buffer
                throw new Error('Received data too large without forming valid JSON.');
            }
        }
        await new Promise(resolve => setTimeout(resolve, 20)); // Wait a bit for more data
    }
}

async function getConfig(): Promise<ApiResult<FullDeviceConfig>> {
    isLoading.value = true;
    lastError.value = null;
    try {
        const command: GetConfigCommand = {
            type: 'GetConfig',
            header: { version: PROTOCOL_VERSION, correlationId: getNextCorrelationId() }
        };
        const responseString = await sendCommandAndGetResponse(command);
        const parsedResponse = JSON.parse(responseString); // Expecting ProtocolResponse structure
        // Now, handle the ProtocolResponse structure
        if (parsedResponse.Config) {
            return { data: parsedResponse.Config.config, error: null, loading: false };
        } else if (parsedResponse.Error) {
            throw new Error(`Device Error: ${parsedResponse.Error.message} (Code: ${parsedResponse.Error.errorCode})`);
        }
        throw new Error('Invalid response structure from getConfig');
    } catch (e: any) {
        lastError.value = e.message;
        return { data: null, error: e.message, loading: false };
    } finally {
        isLoading.value = false;
    }
}

async function setConfig(config: FullDeviceConfig): Promise<ApiResult<boolean>> {
    isLoading.value = true;
    lastError.value = null;
    try {
        const command: SetConfigCommand = {
            type: 'SetConfig',
            header: { version: PROTOCOL_VERSION, correlationId: getNextCorrelationId() },
            config: config
        };
        const responseString = await sendCommandAndGetResponse(command);
        const parsedResponse = JSON.parse(responseString);
        if (parsedResponse.Ack) {
            if (parsedResponse.Ack.success) {
                return { data: true, error: null, loading: false };
            }
            throw new Error(parsedResponse.Ack.message || 'SetConfig failed on device (Ack success false)');
        } else if (parsedResponse.Error) {
            throw new Error(`Device Error: ${parsedResponse.Error.message} (Code: ${parsedResponse.Error.errorCode})`);
        }
        throw new Error('Invalid response structure from setConfig');
    } catch (e: any) {
        lastError.value = e.message;
        return { data: false, error: e.message, loading: false };
    } finally {
        isLoading.value = false;
    }
}

async function resetConfig(): Promise<ApiResult<boolean>> {
    isLoading.value = true;
    lastError.value = null;
    try {
        const command: ResetConfigCommand = {
            type: 'ResetConfig',
            header: { version: PROTOCOL_VERSION, correlationId: getNextCorrelationId() }
        };
        const responseString = await sendCommandAndGetResponse(command);
        const parsedResponse = JSON.parse(responseString);
        if (parsedResponse.Ack) {
            if (parsedResponse.Ack.success) {
                return { data: true, error: null, loading: false };
            }
            throw new Error(parsedResponse.Ack.message || 'ResetConfig failed on device (Ack success false)');
        } else if (parsedResponse.Error) {
            throw new Error(`Device Error: ${parsedResponse.Error.message} (Code: ${parsedResponse.Error.errorCode})`);
        }
        throw new Error('Invalid response structure from resetConfig');
    } catch (e: any) {
        lastError.value = e.message;
        return { data: false, error: e.message, loading: false };
    } finally {
        isLoading.value = false;
    }
}

async function reboot(): Promise<ApiResult<boolean>> {
    isLoading.value = true;
    lastError.value = null;
    try {
        const command: RebootCommand = {
            type: 'Reboot',
            header: { version: PROTOCOL_VERSION, correlationId: getNextCorrelationId() }
        };
        // For reboot, we might not get a conventional response or the connection might drop.
        // The original sendCommand might throw a timeout or error if device reboots too fast.
        // We'll assume an Ack is expected if the command is accepted before reboot.
        const responseString = await sendCommandAndGetResponse(command);
        const parsedResponse = JSON.parse(responseString);
        if (parsedResponse.Ack) {
            if (parsedResponse.Ack.success) {
                // Expect disconnect after this
                isDeviceConnected.value = false;
                usbDevice = null;
                return { data: true, error: null, loading: false };
            }
            throw new Error(parsedResponse.Ack.message || 'Reboot command failed on device (Ack success false)');
        } else if (parsedResponse.Error) {
            throw new Error(`Device Error: ${parsedResponse.Error.message} (Code: ${parsedResponse.Error.errorCode})`);
        }
        throw new Error('Invalid response structure from reboot');
    } catch (e: any) {
        lastError.value = e.message;
        // If it's a timeout because device rebooted, that's sort of expected.
        // However, this generic catch makes it an error.
        // For now, we reflect the error. Could refine later if specific timeout errors need special handling.
        isDeviceConnected.value = false; // Assume disconnected on any error during reboot
        usbDevice = null;
        return { data: false, error: e.message, loading: false };
    } finally {
        isLoading.value = false;
    }
}

export function useDeviceApi() {
    // Expose reactive state and methods
    return reactive({
        isLoading,
        lastError,
        isDeviceConnected,
        connectToDevice,
        disconnectDevice, // Added disconnect
        getConfig,
        setConfig,
        resetConfig,
        reboot,
        debugLogs: internalDebugLogs, // Expose the internal logs
    });
} 