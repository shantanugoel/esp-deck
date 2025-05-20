import { defineStore } from 'pinia';
import type { DeviceSettings, WifiSettings, FullDeviceConfig } from '@/types/protocol'; // Import FullDeviceConfig
import { useMacroPadConfigStore } from './macroPadConfigStore'; // Import macroPadConfigStore
import { useDeviceStore } from './deviceStore'; // Import deviceStore

// Local DeviceSettings type removed, using imported one.

export const useDeviceSettingsStore = defineStore('deviceSettings', {
    state: () => ({
        // Initialize with an empty object or default structure according to imported DeviceSettings
        settings: { wifi: null, timezone_offset: null, api_key: null } as DeviceSettings,
        hasUnsavedChanges: false,
    }),
    getters: {
        getCurrentSettings: (state): DeviceSettings => state.settings,
        isDirty: (state): boolean => state.hasUnsavedChanges,
        // This getter should return the structure expected by deviceService.saveConfig
        getSettingsForSave: (state): DeviceSettings => {
            return state.settings;
        },
    },
    actions: {
        loadSettings(newSettings: DeviceSettings) {
            // Ensure wifi is initialized if it's null/undefined in newSettings but we want to allow updates to its properties
            // The default state now initializes wifi to null, so this explicit check might be less critical
            // but doesn't hurt if newSettings could potentially omit it.
            if (newSettings.wifi === undefined) {
                newSettings.wifi = null;
            }
            this.settings = JSON.parse(JSON.stringify(newSettings)); // Deep copy
            this.hasUnsavedChanges = false;
            console.log('Device settings loaded into store', this.settings);
        },
        // Actions now reflect the structure of DeviceSettings from protocol.ts
        updateWifiSsid(ssid: string) {
            if (this.settings.wifi === null || this.settings.wifi === undefined) { // Check for null or undefined explicitly
                this.settings.wifi = { ssid: '', password: '' };
            }
            this.settings.wifi.ssid = ssid;
            this.hasUnsavedChanges = true;
        },
        updateWifiPassword(password: string) {
            if (this.settings.wifi === null || this.settings.wifi === undefined) { // Check for null or undefined explicitly
                this.settings.wifi = { ssid: '', password: '' };
            }
            this.settings.wifi.password = password;
            this.hasUnsavedChanges = true;
        },
        clearWifiSettings() {
            this.settings.wifi = null;
            this.hasUnsavedChanges = true;
        },
        updateTimezoneOffset(offset: number | null) {
            this.settings.timezone_offset = offset;
            this.hasUnsavedChanges = true;
        },
        updateApiKey(key: string | null) {
            this.settings.api_key = key;
            this.hasUnsavedChanges = true;
        },
        // Generic updateSetting might be complex with nested structures.
        // Specific updaters are preferred for type safety.
        // updateSetting(key: keyof DeviceSettings, value: any) { ... }

        resetChanges() {
            // This should ideally reload from _lastFetchedConfig in deviceStore via an action
            // or use a local copy of the last loaded settings if that's the desired behavior.
            console.log('Device settings changes (simulated reset) - flag cleared, data needs reload from source');
            // For a true reset to last loaded state, you might do:
            // if (this.initialSettings) this.settings = JSON.parse(JSON.stringify(this.initialSettings));
            // Where initialSettings is populated in loadSettings.
            this.hasUnsavedChanges = false;
        },
        markAsSaved() {
            this.hasUnsavedChanges = false;
            console.log('Device settings marked as saved in store');
        },

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
                // Assume deviceStore.saveConfig returns a boolean or throws an error
                const success = await deviceStore.saveConfig(fullConfig);
                if (success) {
                    this.markAsSaved();
                    // Optionally, macroPadConfigStore could also mark itself as saved if its data was part of this save.
                    // However, its state is managed via hasUnsavedChanges which is set by its own actions.
                    // If saveConfig in deviceStore is the single point of truth for saving FullDeviceConfig,
                    // it might be better for deviceStore.saveConfig to also notify macroPadConfigStore.
                    // For now, this store marks its own part as saved.
                    console.log('Device settings successfully saved via deviceStore.saveConfig');
                    return true;
                } else {
                    console.error('Failed to save device settings: deviceStore.saveConfig returned false');
                    // TODO: Propagate error to UI
                    return false;
                }
            } catch (error) {
                console.error('Error saving device settings:', error);
                // TODO: Propagate error to UI
                return false;
            }
        },
    },
}); 