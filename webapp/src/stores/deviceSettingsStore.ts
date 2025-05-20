import { defineStore } from 'pinia';

export type DeviceSettings = {
    wifiSsid?: string;
    wifiPassword?: string; // Consider security implications of storing/handling passwords
    timezoneOffset?: number | null;
    apiKey?: string | null;
    // other settings...
};

export const useDeviceSettingsStore = defineStore('deviceSettings', {
    state: () => ({
        settings: {} as DeviceSettings,
        hasUnsavedChanges: false,
    }),
    getters: {
        getCurrentSettings: (state): DeviceSettings => state.settings,
        isDirty: (state): boolean => state.hasUnsavedChanges,
        getSettingsForSave: (state): DeviceSettings => {
            // In the future, this might filter only changed settings
            return state.settings;
        },
    },
    actions: {
        loadSettings(newSettings: DeviceSettings) {
            this.settings = JSON.parse(JSON.stringify(newSettings)); // Deep copy
            this.hasUnsavedChanges = false;
            console.log('Device settings loaded into store');
        },
        updateWifiSsid(ssid: string) {
            this.settings.wifiSsid = ssid;
            this.hasUnsavedChanges = true;
        },
        updateWifiPassword(password: string) {
            this.settings.wifiPassword = password;
            this.hasUnsavedChanges = true;
        },
        updateTimezoneOffset(offset: number | null) {
            this.settings.timezoneOffset = offset;
            this.hasUnsavedChanges = true;
        },
        updateApiKey(key: string | null) {
            this.settings.apiKey = key;
            this.hasUnsavedChanges = true;
        },
        updateSetting(key: keyof DeviceSettings, value: any) {
            (this.settings as any)[key] = value;
            this.hasUnsavedChanges = true;
        },
        resetChanges() {
            // Similar to macroPadConfigStore, actual reset might need reloading from a base state.
            console.log('Device settings changes (simulated reset) - flag cleared, data might need reload');
            this.hasUnsavedChanges = false;
        },
        markAsSaved() {
            this.hasUnsavedChanges = false;
            console.log('Device settings marked as saved in store');
        },
    },
}); 