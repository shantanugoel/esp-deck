import { defineStore } from 'pinia';

// Define more specific types as needed
export type ButtonConfig = {
    id: string | number;
    name?: string;
    actions?: any[]; // Replace 'any' with a more specific action type later
    color?: string;
    // other properties...
};

export const useMacroPadConfigStore = defineStore('macroPadConfig', {
    state: () => ({
        buttons: [] as ButtonConfig[],
        hasUnsavedChanges: false,
    }),
    getters: {
        getButtonConfig: (state) => (index: number): ButtonConfig | undefined => {
            return state.buttons[index];
        },
        isDirty: (state): boolean => state.hasUnsavedChanges,
        getConfigForSave: (state): ButtonConfig[] => {
            // In the future, this might filter only changed buttons or format data
            return state.buttons;
        },
    },
    actions: {
        loadConfig(config: ButtonConfig[]) {
            this.buttons = JSON.parse(JSON.stringify(config)); // Deep copy
            this.hasUnsavedChanges = false;
            console.log('MacroPad config loaded into store');
        },
        updateButton(index: number, newConfig: Partial<ButtonConfig>) {
            if (this.buttons[index]) {
                this.buttons[index] = { ...this.buttons[index], ...newConfig };
                this.hasUnsavedChanges = true;
            } else {
                console.warn(`Button at index ${index} not found for update.`);
            }
        },
        addButton(config: ButtonConfig) {
            // Example: ensure unique ID or handle as needed
            this.buttons.push(config);
            this.hasUnsavedChanges = true;
        },
        removeButton(buttonId: string | number) {
            this.buttons = this.buttons.filter(b => b.id !== buttonId);
            this.hasUnsavedChanges = true;
        },
        resetChanges() {
            // This might need to reload from a last known saved state or initial state
            // For now, it just clears the dirty flag. Revisit based on fetch/load strategy.
            console.log('MacroPad changes (simulated reset) - flag cleared, data might need reload from deviceStore');
            this.hasUnsavedChanges = false;
        },
        markAsSaved() {
            this.hasUnsavedChanges = false;
            console.log('MacroPad config marked as saved in store');
        },
    },
}); 