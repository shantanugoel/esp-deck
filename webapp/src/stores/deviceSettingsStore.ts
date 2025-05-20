import { defineStore } from 'pinia';
import type { DeviceSettings, WifiSettings, FullDeviceConfig } from '@/types/protocol'; // Import FullDeviceConfig
import { useMacroPadConfigStore } from './macroPadConfigStore'; // Import macroPadConfigStore
import { useDeviceStore } from './deviceStore'; // Import deviceStore
import { isEqual } from 'lodash-es'; // For deep equality checks

// Local DeviceSettings type removed, using imported one.

const defaultDeviceSettings: () => DeviceSettings = () => ({ wifi: null, timezone_offset: null, api_key: null });

export const useDeviceSettingsStore = defineStore('deviceSettings', {
    state: () => ({
        settings: defaultDeviceSettings(),
        initialSettings: defaultDeviceSettings(), // Store the initial state for comparison
        hasUnsavedChanges: false, // Kept for now, but isDirty is the primary source of truth
    }),
    getters: {
        getCurrentSettings: (state): DeviceSettings => state.settings,
        getSettingsForSave: (state): DeviceSettings => state.settings, // Returns current potentially modified settings

        // Granular change detection getters
        isWifiChanged(state): boolean {
            const initialWifi = state.initialSettings.wifi;
            const currentWifi = state.settings.wifi;
            if (initialWifi === null && currentWifi === null) return false;
            if (initialWifi === null || currentWifi === null) return true; // One is null, other is not
            return !isEqual(initialWifi, currentWifi);
        },
        isTimezoneChanged(state): boolean {
            return state.initialSettings.timezone_offset !== state.settings.timezone_offset;
        },
        isApiKeyChanged(state): boolean {
            return state.initialSettings.api_key !== state.settings.api_key;
        },

        // Overall dirty flag based on granular changes
        isDirty(): boolean { // Removed state parameter, uses `this` context
            return this.isWifiChanged || this.isTimezoneChanged || this.isApiKeyChanged;
        },
    },
    actions: {
        loadSettings(newSettings: DeviceSettings) {
            const newSettingsCopy = JSON.parse(JSON.stringify(newSettings || defaultDeviceSettings()));
            this.settings = newSettingsCopy;
            this.initialSettings = JSON.parse(JSON.stringify(newSettingsCopy));
            this.hasUnsavedChanges = false; // isDirty will be false after this
            console.log('Device settings loaded and initial state set', this.settings);
        },
        // Update actions now just update settings; isDirty is reactive via getters
        updateWifiSsid(ssid: string) {
            if (!this.settings.wifi) {
                this.settings.wifi = { ssid: '', password: '' };
            }
            if (this.settings.wifi.ssid !== ssid) {
                this.settings.wifi.ssid = ssid;
            }
        },
        updateWifiPassword(password: string) {
            if (!this.settings.wifi) {
                this.settings.wifi = { ssid: '', password: '' };
            }
            if (this.settings.wifi.password !== password) {
                this.settings.wifi.password = password;
            }
        },
        clearWifiSettings() {
            if (this.settings.wifi !== null) {
                this.settings.wifi = null;
            }
        },
        updateTimezoneOffset(offset: number | null) {
            if (this.settings.timezone_offset !== offset) {
                this.settings.timezone_offset = offset;
            }
        },
        updateApiKey(key: string | null) {
            if (this.settings.api_key !== key) {
                this.settings.api_key = key;
            }
        },
        resetChanges() {
            this.settings = JSON.parse(JSON.stringify(this.initialSettings || defaultDeviceSettings()));
            console.log('Device settings reset to initial loaded state');
        },
        markAsSaved() {
            this.initialSettings = JSON.parse(JSON.stringify(this.settings));
            this.hasUnsavedChanges = false; // isDirty will be false after this
            console.log('Device settings current state marked as saved (initial state updated)');
        },

        // saveDeviceSettings remains largely the same but its success path calls markAsSaved
        async saveDeviceSettings(): Promise<boolean> {
            const macroPadConfigStore = useMacroPadConfigStore();
            const deviceStore = useDeviceStore();

            const currentDeviceSettings = this.getSettingsForSave;
            const currentMacroPadConfig = macroPadConfigStore.getMacroPadConfigForSave;

            const fullConfig: FullDeviceConfig = {
                settings: currentDeviceSettings,
                mappings: currentMacroPadConfig.mappings,
                button_names: currentMacroPadConfig.button_names,
            };

            console.log('Attempting to save full config from deviceSettingsStore', fullConfig);

            try {
                const success = await deviceStore.saveConfig(fullConfig);
                if (success) {
                    this.markAsSaved(); // Update initialSettings to reflect the saved state
                    // Note: deviceStore.saveConfig internally calls fetchConfig, which calls loadSettings.
                    // loadSettings ALREADY updates initialSettings. So this call to markAsSaved might be redundant
                    // if the flow through fetchConfig -> loadSettings is guaranteed.
                    // However, if saveDeviceSettings could be called without that full loop, this is a safeguard.
                    console.log('Device settings successfully saved (and marked as saved in deviceSettingsStore)');
                    return true;
                } else {
                    console.error('Failed to save device settings: deviceStore.saveConfig returned false');
                    return false;
                }
            } catch (error) {
                console.error('Error saving device settings:', error);
                return false;
            }
        },
    },
}); 