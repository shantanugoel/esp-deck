import { defineStore } from 'pinia'
// import { ref, computed } from 'vue'
import { deviceService } from '@/services/deviceService'
import type { FullDeviceConfig, DeviceConnectionInfo, ConfigAction, MappingConfiguration, ConfigActionKeyPress, ConfigActionKeyRelease, ConfigActionMousePress, ConfigActionMouseRelease, ConfigActionMouseMove, ConfigActionMouseWheel, ConfigActionConsumerPress, ConfigActionConsumerRelease, ConfigActionDelay, ConfigActionSendString, ConfigActionSequence } from '@/types/protocol'
import { useMacroPadConfigStore } from './macroPadConfigStore'
import { useDeviceSettingsStore } from './deviceSettingsStore'
import { stringToKeyCodes } from '@/keycodes'; // Import for SendString conversion

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

        _convertRawActionToConfigAction(action: any): ConfigAction {
            if (!action) return { type: 'Unknown', original: null } as any;

            // Handle string-based actions first (for parameterless actions like KeyRelease)
            if (typeof action === 'string') {
                switch (action) {
                    case 'KeyRelease':
                        return { type: 'KeyRelease' } as ConfigActionKeyRelease;
                    case 'MouseRelease':
                        return { type: 'MouseRelease' } as ConfigActionMouseRelease;
                    case 'ConsumerRelease':
                        return { type: 'ConsumerRelease' } as ConfigActionConsumerRelease;
                    default:
                        // If it's a string but not a known parameterless action, it's unknown
                        console.warn('Unrecognized string action:', action);
                        return { type: 'Unknown', originalAction: action } as any;
                }
            }

            // If action already has a 'type' field, assume it's in the new format (or new format but unknown)
            if (action.type) {
                // Basic validation: if it's a known type, cast it. Otherwise, preserve it.
                const knownTypes = ['KeyPress', 'KeyRelease', 'MousePress', 'MouseRelease', 'MouseMove', 'MouseWheel', 'ConsumerPress', 'ConsumerRelease', 'Delay', 'SendString', 'Sequence'];
                if (knownTypes.includes(action.type)) {
                    return action as ConfigAction;
                }
                // If it has a 'type' but it's not one of our known ones, treat as Unknown but keep structure
                console.warn('Action has a type field but it is not a known action type:', action);
                return { type: 'Unknown', originalAction: action } as any;
            }

            // Handle old object-based formats
            if (action.KeyPress) {
                return { type: 'KeyPress', keys: action.KeyPress.keys, modifier: action.KeyPress.modifier } as ConfigActionKeyPress;
            }
            // KeyRelease is handled above if it's a string. If it was an object in old format, it would need specific handling here.
            // Assuming for now that old KeyRelease, if not string, isn't a common case or would be caught by fallback.

            if (action.MousePress) {
                return { type: 'MousePress', button: action.MousePress.button } as ConfigActionMousePress;
            }
            // MouseRelease handled above if string.

            if (action.MouseMove) {
                return { type: 'MouseMove', dx: action.MouseMove.dx, dy: action.MouseMove.dy } as ConfigActionMouseMove;
            }
            if (action.MouseWheel) {
                return { type: 'MouseWheel', amount: action.MouseWheel.amount } as ConfigActionMouseWheel;
            }
            if (action.ConsumerPress) {
                return { type: 'ConsumerPress', usage_id: action.ConsumerPress.usage_id } as ConfigActionConsumerPress;
            }
            // ConsumerRelease handled above if string.

            if (action.Delay) {
                return { type: 'Delay', ms: action.Delay.ms } as ConfigActionDelay;
            }
            // Correctly handle old object-based SendString which contains keys and modifiers directly
            if (action.SendString && Array.isArray(action.SendString.keys) && Array.isArray(action.SendString.modifiers)) {
                return { type: 'SendString', keys: action.SendString.keys, modifiers: action.SendString.modifiers } as ConfigActionSendString;
            }

            if (action.Sequence && Array.isArray(action.Sequence)) {
                const nestedActions = (action.Sequence || []).map(this._convertRawActionToConfigAction.bind(this));
                return { type: 'Sequence', actions: nestedActions } as ConfigActionSequence;
            }

            console.warn('Unrecognized/unhandled old action format during conversion:', action);
            return { type: 'Unknown', originalAction: action } as any;
        },

        _updateConnectionState(connected: boolean, deviceInfo?: StoreDeviceInfo | null, error?: string | null) {
            this.isConnected = connected;
            this.isConnecting = false; // Connection attempt finished
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
            this.isLoading = true;
            this.error = null;

            const response = await deviceService.connect();
            if (response.success && response.data) {
                this._updateConnectionState(true, response.data);
                this._addStoreLog(`Successfully connected to ${response.data.productName || 'USB Device'}.`, 'info');

                this.isLoading = false;
                await this.fetchConfig();
            } else {
                this._updateConnectionState(false, null, response.error || 'USB Connection failed');
                this.isLoading = false;
            }
            if (this.isConnecting) {
                this.isConnecting = false;
                this.isLoading = false;
            }
        },

        async disconnect() {
            this._addStoreLog('Attempting to disconnect USB...', 'action');
            const response = await deviceService.disconnect();
            if (response.success) {
                this._updateConnectionState(false);
                this._addStoreLog('Successfully disconnected USB.', 'info');
            } else {
                this.error = response.error || 'Failed to disconnect USB properly';
                this._addStoreLog(`USB Disconnect error: ${this.error}`, 'error');
                this._updateConnectionState(false, null, this.error);
            }
        },

        async fetchConfig() {
            if (!this.isConnected) {
                this._addStoreLog('Cannot fetch config: Not connected.', 'error');
                return;
            }
            if (this.isLoading) {
                this._addStoreLog('Fetch config skipped: An operation is already in progress (isLoading is true).', 'info');
                return;
            }
            this._addStoreLog('Fetching configuration...', 'action');
            this.isLoading = true;
            this.error = null;

            const response = await deviceService.fetchConfig();
            if (response.success && response.data) {
                this._lastFetchedConfig = JSON.parse(JSON.stringify(response.data)); // Store a deep copy
                const macroPadStore = useMacroPadConfigStore();
                const deviceSettingsStore = useDeviceSettingsStore();

                // Prepare mappings with converted actions
                const processedMappings: MappingConfiguration = {};
                if (response.data.mappings) {
                    for (const rawKey in response.data.mappings) {
                        if (rawKey === 'default') continue;

                        if (Object.prototype.hasOwnProperty.call(response.data.mappings, rawKey)) {
                            const rawActions = response.data.mappings[rawKey];
                            const actions = Array.isArray(rawActions)
                                ? rawActions.map(action => this._convertRawActionToConfigAction(action))
                                : [];

                            const numericKey = parseInt(rawKey, 10);
                            // If the rawKey is a positive integer string (e.g., "1", "2", ...),
                            // assume it's 1-based and convert to a 0-based string key.
                            if (!isNaN(numericKey) && numericKey > 0 && String(numericKey) === rawKey) {
                                const zeroBasedKey = String(numericKey - 1);
                                processedMappings[zeroBasedKey] = actions;
                                this._addStoreLog(`Mapping key "${rawKey}" from device converted to "${zeroBasedKey}".`, 'info');
                            } else {
                                // If key is "0", or not a simple positive integer string (e.g., "special_key"), use it as is.
                                processedMappings[rawKey] = actions;
                                if (rawKey !== "0") {
                                    this._addStoreLog(`Mapping key "${rawKey}" from device used as is.`, 'info');
                                }
                            }
                        }
                    }
                }

                const processedButtonNames = response.data.button_names ? JSON.parse(JSON.stringify(response.data.button_names)) : {};

                macroPadStore.loadConfig(processedMappings, processedButtonNames);
                deviceSettingsStore.loadSettings(response.data.settings);

                this._addStoreLog('Configuration fetched and applied to stores.', 'info');
            } else {
                this.error = response.error || 'Failed to fetch config';
                this._addStoreLog(`Fetch config error: ${this.error}`, 'error');
            }
            this.isLoading = false;
        },

        _convertUiActionToDeviceAction(uiAction: ConfigAction): any {
            switch (uiAction.type) {
                case 'KeyPress':
                    const { type: _kpType, ...kpPayload } = uiAction as ConfigActionKeyPress;
                    return { KeyPress: kpPayload };
                case 'KeyRelease':
                    return 'KeyRelease';
                case 'MousePress':
                    const { type: _mpType, ...mpPayload } = uiAction as ConfigActionMousePress;
                    return { MousePress: mpPayload };
                case 'MouseRelease':
                    return 'MouseRelease';
                case 'MouseMove':
                    const { type: _mmType, ...mmPayload } = uiAction as ConfigActionMouseMove;
                    return { MouseMove: mmPayload };
                case 'MouseWheel':
                    const { type: _mwType, ...mwPayload } = uiAction as ConfigActionMouseWheel;
                    return { MouseWheel: mwPayload };
                case 'ConsumerPress':
                    const { type: _cpType, ...cpPayload } = uiAction as ConfigActionConsumerPress;
                    return { ConsumerPress: cpPayload };
                case 'ConsumerRelease':
                    return 'ConsumerRelease';
                case 'Delay':
                    const { type: _dType, ...dPayload } = uiAction as ConfigActionDelay;
                    return { Delay: dPayload };
                case 'SendString':
                    const { type: _ssType, ...ssPayload } = uiAction as ConfigActionSendString;
                    return { SendString: ssPayload };
                case 'Sequence':
                    const seqAction = uiAction as ConfigActionSequence;
                    const deviceActions = seqAction.actions.map(this._convertUiActionToDeviceAction.bind(this));
                    return { Sequence: { actions: deviceActions } }; // Ensure Sequence payload matches Rust: { actions: Vec<ConfigAction> }
                default:
                    console.warn('Unknown UI action type during device conversion:', uiAction);
                    return { UnknownDeviceAction: { originalUiAction: uiAction } };
            }
        },

        async saveConfig(configPayload: FullDeviceConfig): Promise<boolean> {
            if (!this.isConnected) {
                this._addStoreLog('Cannot save config: Not connected.', 'error');
                return false;
            }
            if (this.isLoading) return false;
            this._addStoreLog(`Saving configuration...`, 'action');
            this.isLoading = true;
            this.error = null;

            const payloadForDevice = JSON.parse(JSON.stringify(configPayload));

            // 1. Convert action structures within mappings (from UI format to device format)
            if (payloadForDevice.mappings) {
                for (const buttonIdKey in payloadForDevice.mappings) { // buttonIdKey is "0", "1", etc. (0-based from UI/store)
                    if (Object.prototype.hasOwnProperty.call(payloadForDevice.mappings, buttonIdKey)) {
                        const uiActions = payloadForDevice.mappings[buttonIdKey];
                        if (Array.isArray(uiActions)) {
                            payloadForDevice.mappings[buttonIdKey] = uiActions.map(this._convertUiActionToDeviceAction.bind(this));
                        }
                    }
                }
            }

            // 2. Transform the MAPPING KEYS from 0-based (UI/store) to 1-based (device)
            if (payloadForDevice.mappings) {
                const deviceKeyedMappings: MappingConfiguration = {};
                for (const zeroBasedKey in payloadForDevice.mappings) { // zeroBasedKey is "0", "1", etc.
                    if (Object.prototype.hasOwnProperty.call(payloadForDevice.mappings, zeroBasedKey)) {
                        const numericKey = parseInt(zeroBasedKey, 10);
                        if (!isNaN(numericKey)) {
                            const oneBasedKey = String(numericKey + 1); // Convert "0" to "1", "1" to "2", etc.
                            deviceKeyedMappings[oneBasedKey] = payloadForDevice.mappings[zeroBasedKey];
                            this._addStoreLog(`Mapping key "${zeroBasedKey}" from UI converted to "${oneBasedKey}" for device.`, 'info');
                        } else {
                            // This case should ideally not happen for standard button keys.
                            // If it does, pass it through but log it, as it might indicate an issue.
                            deviceKeyedMappings[zeroBasedKey] = payloadForDevice.mappings[zeroBasedKey];
                            this._addStoreLog(`Non-numeric mapping key "${zeroBasedKey}" encountered during save. Sent as is to device.`, 'info');
                        }
                    }
                }
                payloadForDevice.mappings = deviceKeyedMappings; // Replace with 1-based key mappings
            }

            // Note: button_names are Record<number, string> (0-indexed) in UI/store,
            // and HashMap<usize, String> (0-indexed) on device. No key conversion needed for button_names.

            const response = await deviceService.saveConfig(payloadForDevice);
            if (response.success) {
                this._addStoreLog('Configuration saved successfully.', 'info');
                await this.fetchConfig();
                this.isLoading = false;
                return true;
            } else {
                this.error = response.error || 'Failed to save config';
                this._addStoreLog(`Save config error: ${this.error}`, 'error');
                this.isLoading = false;
                return false;
            }
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
                await this.fetchConfig();
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
            this._addStoreLog('Requesting device reboot...', 'action');
            this.error = null;
            const wasLoading = this.isLoading;
            this.isLoading = true;

            const response = await deviceService.rebootDevice();
            if (response.success) {
                this._addStoreLog('Device reboot command acknowledged. Connection will be lost.', 'info');
                this._updateConnectionState(false, null, 'Device rebooted. Connection lost.');
            } else {
                this.error = response.error || 'Failed to send reboot command';
                this._addStoreLog(`Reboot error: ${this.error}`, 'error');
                this.isLoading = wasLoading;
            }
            if (this.isConnected) this.isLoading = false;
        },

        async backupDeviceConfig() {
            if (!this.isConnected) {
                this._addStoreLog('Cannot backup: Not connected.', 'error');
                return;
            }
            if (this.isLoading) return;
            this._addStoreLog('Preparing to backup device config from device...', 'action');
            this.isLoading = true;
            this.error = null;

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

        async backupCurrentUiConfig(currentConfig: FullDeviceConfig) {
            if (this.isLoading && !this.isConnecting) return;
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
                let jsonDataToSave: FullDeviceConfig = JSON.parse(fileContents);

                // Convert actions within mappings from UI format (if present) to device-expected format
                if (jsonDataToSave.mappings) {
                    for (const buttonId in jsonDataToSave.mappings) {
                        if (Object.prototype.hasOwnProperty.call(jsonDataToSave.mappings, buttonId)) {
                            const actionsInFile = jsonDataToSave.mappings[buttonId];
                            if (Array.isArray(actionsInFile)) {
                                if (actionsInFile.length > 0 && typeof actionsInFile[0] === 'object' && actionsInFile[0] !== null && 'type' in actionsInFile[0]) {
                                    this._addStoreLog('Detected UI action format in uploaded file, converting to device format.', 'info');
                                    jsonDataToSave.mappings[buttonId] = actionsInFile.map(this._convertUiActionToDeviceAction.bind(this));
                                } else {
                                    this._addStoreLog('Uploaded file actions appear to be in device format or unknown, attempting to send as-is.', 'info');
                                }
                            }
                        }
                    }
                }

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
});