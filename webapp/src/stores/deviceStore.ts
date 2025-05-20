import { defineStore } from 'pinia'
// import { ref, computed } from 'vue'
import { deviceService } from '@/services/deviceService'
import type { FullDeviceConfig, DeviceConnectionInfo, ConfigAction, MappingConfiguration } from '@/types/protocol'
import { useMacroPadConfigStore, type ButtonConfig as MacroPadButtonConfig } from './macroPadConfigStore'
import { useDeviceSettingsStore } from './deviceSettingsStore'

// Keeping store-specific debug log type, but deviceApi also has its own logs
export type DebugLog = {
    timestamp: Date;
    message: string;
    type: 'info' | 'error' | 'action'; // Store-level log types
};

// Type for DeviceInfo in this store, now aliasing DeviceConnectionInfo from protocol types
export type StoreDeviceInfo = DeviceConnectionInfo;

export const useDeviceStore = defineStore('device', {
    state: () => ({
        isConnected: false,
        isConnecting: false, // For the connect action itself
        isLoading: false,    // For generic device operations like fetch/save config
        deviceInfo: null as StoreDeviceInfo | null,
        // Store-specific debug logs. UI can also display deviceService.debugLogs for USB-level comms.
        storeDebugLogs: [] as DebugLog[],
        _lastFetchedConfig: null as FullDeviceConfig | null,
        error: null as string | null,
    }),
    getters: {
        getFormattedStatus: (state): string => {
            if (state.isConnecting) return 'Connecting via USB...';
            if (state.isConnected && state.deviceInfo) {
                return `Connected to ${state.deviceInfo.productName || 'USB Device'}${state.deviceInfo.serialNumber ? ' (S/N: ' + state.deviceInfo.serialNumber + ')' : ''}`;
            }
            if (state.isConnected) return 'Connected via USB';
            return 'Disconnected';
        },
        hasStoreDebugLogs: (state): boolean => state.storeDebugLogs.length > 0,
        lastFetchedConfig: (state): FullDeviceConfig | null => state._lastFetchedConfig,
        // Expose reactive connection state from the service for convenience if needed by UI directly
        // This avoids duplicating the isConnected state if deviceService.isConnected is the source of truth
        // However, the store maintains its own isConnected for actions and fine-grained control.
        // So, these are more for observing the raw USB state if desired.
        isUsbConnected: () => deviceService.isConnected,
        isUsbLoading: () => deviceService.isLoading,
    },
    actions: {
        _addStoreLog(message: string, type: DebugLog['type'] = 'info') {
            const newLog: DebugLog = { message, type, timestamp: new Date() };
            this.storeDebugLogs.unshift(newLog);
            if (this.storeDebugLogs.length > 100) this.storeDebugLogs.pop(); // Keep logs bounded
            if (type === 'error') console.error(`[DeviceStore] ${message}`);
            else console.log(`[DeviceStore] ${message}`);
        },

        _updateConnectionState(connected: boolean, deviceInfo?: StoreDeviceInfo | null, error?: string | null) {
            this.isConnected = connected;
            this.isConnecting = false; // Connection attempt finished
            // isLoading should be managed by specific operations like fetch/save, not connection itself unless it implies a fetch
            this.deviceInfo = deviceInfo || null;
            this.error = error || null;
            if (error) this._addStoreLog(error, 'error');
            if (!connected) {
                this._lastFetchedConfig = null;
            }
        },

        async connect() {
            if (this.isConnecting || this.isConnected) return;
            this._addStoreLog('Attempting to connect via USB...', 'action');
            this.isConnecting = true;
            this.isLoading = true; // Show global loading during connect sequence
            this.error = null;

            const response = await deviceService.connect(); // No argument needed
            if (response.success && response.data) {
                this._updateConnectionState(true, response.data);
                this._addStoreLog(`Successfully connected to ${response.data.productName || 'USB Device'}.`, 'info');
                // await this.fetchConfig(); // Auto-fetch on successful connect (optional)
            } else {
                this._updateConnectionState(false, null, response.error || 'USB Connection failed');
            }
            this.isLoading = false; // Hide global loading
        },

        async disconnect() {
            // if (!this.isConnected && !this.isConnecting) return; // Allow disconnect even if only connecting
            this._addStoreLog('Attempting to disconnect USB...', 'action');
            // No need to set isLoading for a quick disconnect op unless service indicates it
            // deviceService.disconnect will update its own reactive state (deviceApi.isDeviceConnected)
            const response = await deviceService.disconnect();
            if (response.success) {
                this._updateConnectionState(false); // Update store's view of connection
                this._addStoreLog('Successfully disconnected USB.', 'info');
            } else {
                // Handle potential disconnect failure
                this.error = response.error || 'Failed to disconnect USB properly';
                this._addStoreLog(`USB Disconnect error: ${this.error}`, 'error');
                // Even if service reports error, we mark as disconnected in store
                this._updateConnectionState(false, null, this.error);
            }
        },

        async fetchConfig() {
            if (!this.isConnected) {
                this._addStoreLog('Cannot fetch config: Not connected.', 'error');
                return;
            }
            if (this.isLoading) return; // Avoid concurrent fetches
            this._addStoreLog('Fetching configuration...', 'action');
            this.isLoading = true;
            this.error = null;

            const response = await deviceService.fetchConfig();
            if (response.success && response.data) {
                this._lastFetchedConfig = response.data;
                const macroPadStore = useMacroPadConfigStore();
                const deviceSettingsStore = useDeviceSettingsStore();

                const mappings: MappingConfiguration = response.data.mappings;
                const buttonNames = response.data.button_names;

                const buttonConfigsForMacroPadStore: MacroPadButtonConfig[] = Object.entries(mappings)
                    .filter(([idStr]) => idStr !== 'default') // Filter out the "default" mapping
                    .map(([idStr, actions]) => {
                        const numericId = parseInt(idStr, 10);
                        let buttonName: string | undefined = undefined;
                        if (buttonNames && !isNaN(numericId) && numericId > 0) {
                            // Adjust for 0-indexed buttonNames array (idStr "1" maps to buttonNames[0])
                            buttonName = buttonNames[numericId - 1];
                        }
                        return {
                            id: idStr,
                            actions: actions as ConfigAction[],
                            name: buttonName,
                        };
                    });
                macroPadStore.loadConfig(buttonConfigsForMacroPadStore);

                deviceSettingsStore.loadSettings(response.data.settings);
                this._addStoreLog('Configuration fetched and applied to stores.', 'info');
            } else {
                this.error = response.error || 'Failed to fetch config';
                this._addStoreLog(`Fetch config error: ${this.error}`, 'error');
            }
            this.isLoading = false;
        },

        async saveConfig(configPayload: FullDeviceConfig) { // Expecting the full config structure
            if (!this.isConnected) {
                this._addStoreLog('Cannot save config: Not connected.', 'error');
                return;
            }
            if (this.isLoading) return;
            this._addStoreLog(`Saving configuration...`, 'action');
            // this._addStoreLog(`Config: ${JSON.stringify(configPayload)}`, 'info'); // Can be too verbose
            this.isLoading = true;
            this.error = null;

            const response = await deviceService.saveConfig(configPayload);
            if (response.success) {
                this._addStoreLog('Configuration saved successfully.', 'info');
                // Optimistically update _lastFetchedConfig or refetch to ensure consistency.
                // Fetching is safer.
                await this.fetchConfig();
            } else {
                this.error = response.error || 'Failed to save config';
                this._addStoreLog(`Save config error: ${this.error}`, 'error');
            }
            this.isLoading = false;
        },

        async resetConfig() {
            if (!this.isConnected) {
                this._addStoreLog('Cannot reset config: Not connected.', 'error');
                return;
            }
            if (this.isLoading) return;
            this._addStoreLog('Requesting device reset to factory defaults...', 'action');
            this.isLoading = true;
            this.error = null;

            const response = await deviceService.resetConfig();
            if (response.success) {
                this._addStoreLog('Device reset successfully. Fetching new config...', 'info');
                await this.fetchConfig(); // Fetch fresh config after reset
            } else {
                this.error = response.error || 'Failed to reset config';
                this._addStoreLog(`Reset config error: ${this.error}`, 'error');
            }
            this.isLoading = false;
        },

        async rebootDevice() {
            if (!this.isConnected) {
                this._addStoreLog('Cannot reboot: Not connected.', 'error');
                return;
            }
            // Unlike other loading operations, reboot means we expect disconnection.
            // isLoading might briefly be true, but isConnected state change is primary.
            this._addStoreLog('Requesting device reboot...', 'action');
            this.error = null;
            // Set isLoading for the command send, but expect it to be cleared by connection state change
            const wasLoading = this.isLoading;
            this.isLoading = true;

            const response = await deviceService.rebootDevice();
            if (response.success) {
                this._addStoreLog('Device reboot command acknowledged. Connection will be lost.', 'info');
                // The deviceService/useDeviceApi should set its internal isConnected to false.
                // We update our store's state accordingly.
                this._updateConnectionState(false, null, 'Device rebooted. Connection lost.');
            } else {
                this.error = response.error || 'Failed to send reboot command';
                this._addStoreLog(`Reboot error: ${this.error}`, 'error');
                this.isLoading = wasLoading; // Restore previous loading state if command failed
            }
            // isLoading should naturally become false if _updateConnectionState(false) is called.
            // If reboot command failed without disconnect, ensure isLoading is reset.
            if (this.isConnected) this.isLoading = false;
        },

        // Backup/Upload actions remain largely the same but use FullDeviceConfig
        async backupDeviceConfig() {
            if (!this.isConnected) {
                this._addStoreLog('Cannot backup: Not connected.', 'error');
                return;
            }
            if (this.isLoading) return;
            this._addStoreLog('Preparing to backup device config from device...', 'action');
            this.isLoading = true;
            this.error = null;

            // Fetch the latest config directly from device for backup
            const fetchResponse = await deviceService.fetchConfig();
            if (fetchResponse.success && fetchResponse.data) {
                this._downloadJson(fetchResponse.data, `device-config-backup-${new Date().toISOString().slice(0, 19).replace(/[-T:]/g, '')}.json`);
                this._addStoreLog('Device config downloaded for backup.', 'info');
            } else {
                this.error = fetchResponse.error || 'Failed to fetch config for backup';
                this._addStoreLog(`Backup error (fetch): ${this.error}`, 'error');
            }
            this.isLoading = false;
        },

        async backupCurrentUiConfig(currentConfig: FullDeviceConfig) { // Parameter type updated
            if (this.isLoading && !this.isConnecting) return; // Allow during connection for initial state backup if needed
            this._addStoreLog('Downloading current UI config snapshot...', 'action');
            this._downloadJson(currentConfig, `ui-current-config-backup-${new Date().toISOString().slice(0, 19).replace(/[-T:]/g, '')}.json`);
            this._addStoreLog('UI config snapshot downloaded.', 'info');
        },

        async uploadConfig(fileContents: string) {
            if (!this.isConnected) {
                this._addStoreLog('Cannot upload: Not connected.', 'error');
                return;
            }
            if (this.isLoading) return;
            this._addStoreLog('Uploading new configuration...', 'action');
            this.isLoading = true;
            this.error = null;
            try {
                const jsonDataToSave: FullDeviceConfig = JSON.parse(fileContents);
                // Basic validation could be added here against a schema if available
                const response = await deviceService.saveConfig(jsonDataToSave);
                if (response.success) {
                    this._addStoreLog('Uploaded config saved successfully. Fetching new state...', 'info');
                    await this.fetchConfig();
                } else {
                    this.error = response.error || 'Failed to save uploaded config';
                    this._addStoreLog(`Upload save error: ${this.error}`, 'error');
                }
            } catch (e: any) {
                console.error('[deviceStore] Error parsing/uploading config file:', e);
                this.error = e.message || 'Invalid config file format during upload.';
                this._addStoreLog(`Upload parsing error: ${this.error}`, 'error');
            }
            this.isLoading = false;
        },

        _downloadJson(data: any, filename: string) {
            try {
                const jsonString = JSON.stringify(data, null, 2);
                const blob = new Blob([jsonString], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = filename;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            } catch (e: any) {
                console.error("[deviceStore] Error creating download:", e);
                this.error = e.message || "Failed to prepare download.";
                this._addStoreLog(`Download prep error: ${this.error}`, 'error');
            }
        }
    }
}) 