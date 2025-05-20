import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useDeviceApi } from '../composables/useDeviceApi'

// Types for device config (expand as needed)
export type DeviceConfig = any

type StagedButtonChange = {
    macro?: any[],
    name?: string,
}

export type DeviceInfo = {
    ipAddress?: string;
    firmwareVersion?: string;
    // Add other relevant device info fields
}

export type DebugLog = {
    timestamp: Date;
    message: string;
    type: 'sent' | 'received';
}

export const useDeviceStore = defineStore('device', {
    state: () => ({
        isConnected: false,
        isConnecting: false,
        isLoading: false, // For generic device operations
        deviceInfo: null as DeviceInfo | null,
        debugLogs: [] as DebugLog[],
        // The following were likely from an old store, decide if they belong in the new structure
        // For Phase 1, they are not strictly part of the `deviceStore` as per the new plan,
        // but I will keep them commented out for now if they need to be re-integrated.
        // lastFetchedConfig: null as DeviceConfig | null, 
        // deviceConfig: null as DeviceConfig | null,
        // loading: false, // duplicate of isLoading
        // error: null as string | null, 
        // stagedButtonChanges: {} as Record<number, StagedButtonChange>,
    }),
    getters: {
        getFormattedStatus: (state): string => {
            if (state.isConnecting) return 'Connecting...';
            if (state.isConnected && state.deviceInfo) return `Connected to ${state.deviceInfo.ipAddress || 'device'}`;
            if (state.isConnected) return 'Connected';
            return 'Disconnected';
        },
        hasDebugLogs: (state): boolean => state.debugLogs.length > 0,
    },
    actions: {
        // Core actions to be implemented in Phase 2, with placeholders for Phase 1
        async connect(targetDevice?: string) {
            console.log('Attempting to connect to', targetDevice || 'default device');
            this.isConnecting = true;
            this.isLoading = true;
            // Simulate connection attempt
            await new Promise(resolve => setTimeout(resolve, 1500));
            this.isConnected = true;
            this.isConnecting = false;
            this.isLoading = false;
            this.deviceInfo = { ipAddress: targetDevice || '192.168.1.100', firmwareVersion: '1.0.0' };
            console.log('Connected to', this.deviceInfo.ipAddress);
            // In a real scenario, this would also trigger fetching initial config
            // await this.fetchConfig(); 
        },
        disconnect() {
            console.log('Disconnecting...');
            this.isConnected = false;
            this.isConnecting = false;
            this.isLoading = false;
            this.deviceInfo = null;
            console.log('Disconnected');
        },
        async fetchConfig() {
            console.log('Fetching config...');
            this.isLoading = true;
            await new Promise(resolve => setTimeout(resolve, 1000));
            // Example: this.deviceConfig = fetchedData.config;
            // Example: useMacroPadConfigStore().loadConfig(fetchedData.config.macros);
            // Example: useDeviceSettingsStore().loadSettings(fetchedData.config.settings);
            this.isLoading = false;
            console.log('Config fetched (simulated)');
        },
        async saveConfig(configPayload: any) {
            console.log('Saving config:', configPayload);
            this.isLoading = true;
            await new Promise(resolve => setTimeout(resolve, 1000));
            this.isLoading = false;
            console.log('Config saved (simulated)');
            // Optionally re-fetch config after save
            // await this.fetchConfig();
        },
        async resetConfig() {
            console.log('Resetting config...');
            this.isLoading = true;
            await new Promise(resolve => setTimeout(resolve, 1000));
            this.isLoading = false;
            console.log('Config reset (simulated)');
            // await this.fetchConfig(); 
        },
        async rebootDevice() {
            console.log('Rebooting device...');
            this.isLoading = true;
            await new Promise(resolve => setTimeout(resolve, 500));
            this.isLoading = false;
            this.isConnected = false; // Device will likely disconnect
            this.deviceInfo = null;
            console.log('Device reboot command sent (simulated)');
        },
        async backupDeviceConfig() {
            console.log('Backing up device config...');
            this.isLoading = true;
            await new Promise(resolve => setTimeout(resolve, 500));
            // Simulate download
            alert('Device config backup downloaded (simulated).');
            this.isLoading = false;
        },
        async backupCurrentUiConfig(currentConfig: any) {
            console.log('Backing up current UI config:', currentConfig);
            this.isLoading = true;
            await new Promise(resolve => setTimeout(resolve, 500));
            alert('Current UI config backup downloaded (simulated).');
            this.isLoading = false;
        },
        async uploadConfig(fileContents: string | ArrayBuffer | null) {
            console.log('Uploading config...');
            this.isLoading = true;
            await new Promise(resolve => setTimeout(resolve, 1000));
            try {
                if (typeof fileContents === 'string') {
                    const jsonData = JSON.parse(fileContents);
                    console.log('Uploaded config data:', jsonData);
                    // Process jsonData, potentially update other stores or call saveConfig
                    // e.g., await this.saveConfig(jsonData.config || jsonData);
                } else {
                    throw new Error('File content is not a string');
                }
            } catch (error) {
                console.error('Error processing uploaded config file:', error);
                alert('Invalid config file format or content.');
            }
            this.isLoading = false;
            console.log('Config upload processed (simulated)');
        },
        addDebugLog(log: { message: string; type: 'sent' | 'received' }) {
            const newLog = { ...log, timestamp: new Date() };
            this.debugLogs = [newLog, ...this.debugLogs.slice(0, 199)]; // Add to start, limit size
        },
        sendMessage(message: any) {
            // Placeholder for sending message to device (e.g., via WebSocket)
            const messageString = JSON.stringify(message);
            console.log('Sending message to device:', messageString);
            this.addDebugLog({ message: messageString, type: 'sent' });
            // Actual send logic via deviceService.ts will be here in later phases
        },
        // Actions from the old store that might need re-integration or belong elsewhere:
        // stageButtonMacro(idx: number, macro: any[]) { ... }
        // stageButtonName(idx: number, name: string) { ... }
        // isButtonDirty(idx: number) { ... }
        // getStagedButtonMacro(idx: number): any[] | undefined { ... }
        // getStagedButtonName(idx: number): string | undefined { ... }
        // clearStagedChanges() { ... }
        // setDeviceConfig(config: DeviceConfig) { ... }
    }
}) 