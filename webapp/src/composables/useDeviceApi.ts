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
const MAGIC_WORD = 0xE59DECC0; // Restore magic word

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

    // Send data using WebUSB transferOut with MAGIC_WORD + Length + Payload framing
    const headerBuffer = new ArrayBuffer(4);
    new DataView(headerBuffer).setUint32(0, MAGIC_WORD, false); // false for Little Endian
    const bodyBuffer = new TextEncoder().encode(commandString);
    const lengthBuffer = new ArrayBuffer(4);
    new DataView(lengthBuffer).setUint32(0, bodyBuffer.byteLength, false); // false for Little Endian

    const dataToSend = new Uint8Array(headerBuffer.byteLength + lengthBuffer.byteLength + bodyBuffer.byteLength);
    dataToSend.set(new Uint8Array(headerBuffer), 0);
    dataToSend.set(new Uint8Array(lengthBuffer), headerBuffer.byteLength);
    dataToSend.set(bodyBuffer, headerBuffer.byteLength + lengthBuffer.byteLength);

    await usbDevice.transferOut(ENDPOINT_OUT, dataToSend);

    // Receive data using WebUSB transferIn, expecting MAGIC_WORD + Length + Payload framing
    let accumulatedBuffer = new Uint8Array(0);
    const timeoutMs = 5000;
    const startTime = Date.now();

    while (true) {
        if (Date.now() - startTime > timeoutMs) {
            throw new Error('Timeout waiting for device response.');
        }

        const result = await usbDevice.transferIn(ENDPOINT_IN, 2048); // Read in chunks

        if (result.status === 'stall') {
            await usbDevice.clearHalt('in', ENDPOINT_IN);
            continue;
        }
        if (!result.data || result.data.byteLength === 0) {
            await new Promise(resolve => setTimeout(resolve, 50));
            continue;
        }

        const newChunk = new Uint8Array(result.data.buffer);
        const tempBuffer = new Uint8Array(accumulatedBuffer.length + newChunk.length);
        tempBuffer.set(accumulatedBuffer, 0);
        tempBuffer.set(newChunk, accumulatedBuffer.length);
        accumulatedBuffer = tempBuffer;

        // Try to parse the frame from accumulatedBuffer
        if (accumulatedBuffer.length >= 8) { // Minimum length for Magic Word + Length
            const view = new DataView(accumulatedBuffer.buffer, accumulatedBuffer.byteOffset, accumulatedBuffer.byteLength);
            const magic = view.getUint32(0, true); // true for little Endian
            const payloadLength = view.getUint32(4, true); // true for little Endian

            if (magic !== MAGIC_WORD) {
                // Clear buffer and throw error or try to find next magic word (simpler to error out)
                accumulatedBuffer = new Uint8Array(0);
                throw new Error(`Magic word mismatch. Expected ${MAGIC_WORD.toString(16)}, got ${magic.toString(16)}`);
            }

            if (accumulatedBuffer.length >= 8 + payloadLength) {
                // We have the full frame
                const payloadBytes = accumulatedBuffer.slice(8, 8 + payloadLength);
                const decodedPayload = new TextDecoder().decode(payloadBytes);
                addDebugLog('received', decodedPayload);
                // Remaining bytes in buffer are part of the next message, if any (not handled here, assumes one response per command)
                // accumulatedBuffer = accumulatedBuffer.slice(8 + payloadLength);
                return decodedPayload;
            }
            // Not enough data for the full payload yet, continue accumulating
        }
        await new Promise(resolve => setTimeout(resolve, 20));
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
        const parsedResponse = JSON.parse(responseString);

        // Check if it's an ErrorResponse (sent directly by Rust)
        if (typeof parsedResponse.message === 'string' && typeof parsedResponse.errorCode === 'number' && parsedResponse.header) {
            throw new Error(`Device Error: ${parsedResponse.message} (Code: ${parsedResponse.errorCode})`);
        }
        // Check if it's a GetConfigResponse (sent directly by Rust)
        if (parsedResponse.config && parsedResponse.header) {
            return { data: parsedResponse.config, error: null, loading: false };
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

        // Check if it's an ErrorResponse
        if (typeof parsedResponse.message === 'string' && typeof parsedResponse.errorCode === 'number' && parsedResponse.header) {
            throw new Error(`Device Error: ${parsedResponse.message} (Code: ${parsedResponse.errorCode})`);
        }
        // Check if it's an AckResponse
        if (typeof parsedResponse.success === 'boolean' && parsedResponse.header) {
            if (parsedResponse.success) {
                return { data: true, error: null, loading: false };
            }
            throw new Error(parsedResponse.message || 'SetConfig failed on device (Ack success false)');
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

        // Check if it's an ErrorResponse
        if (typeof parsedResponse.message === 'string' && typeof parsedResponse.errorCode === 'number' && parsedResponse.header) {
            throw new Error(`Device Error: ${parsedResponse.message} (Code: ${parsedResponse.errorCode})`);
        }
        // Check if it's an AckResponse
        if (typeof parsedResponse.success === 'boolean' && parsedResponse.header) {
            if (parsedResponse.success) {
                return { data: true, error: null, loading: false };
            }
            throw new Error(parsedResponse.message || 'ResetConfig failed on device (Ack success false)');
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
        const responseString = await sendCommandAndGetResponse(command);
        const parsedResponse = JSON.parse(responseString);

        // Check if it's an ErrorResponse
        if (typeof parsedResponse.message === 'string' && typeof parsedResponse.errorCode === 'number' && parsedResponse.header) {
            throw new Error(`Device Error: ${parsedResponse.message} (Code: ${parsedResponse.errorCode})`);
        }
        // Check if it's an AckResponse
        if (typeof parsedResponse.success === 'boolean' && parsedResponse.header) {
            if (parsedResponse.success) {
                isDeviceConnected.value = false; // Expect disconnect
                usbDevice = null;
                return { data: true, error: null, loading: false };
            }
            throw new Error(parsedResponse.message || 'Reboot command failed on device (Ack success false)');
        }
        throw new Error('Invalid response structure from reboot');
    } catch (e: any) {
        lastError.value = e.message;
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