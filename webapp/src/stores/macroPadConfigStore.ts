import { defineStore } from 'pinia';
import type { ConfigAction } from '@/types/protocol'; // Import ConfigAction

// Define more specific types as needed
export type ButtonConfig = {
    id: string | number; // Keep as is, string keys from MappingConfiguration will be used
    name?: string;
    actions: ConfigAction[]; // Use imported ConfigAction type
    color?: string; // Assuming this is a UI-specific property managed by this store
    // other properties...
};

export const useMacroPadConfigStore = defineStore('macroPadConfig', {
    state: () => ({
        buttons: [] as ButtonConfig[],
        hasUnsavedChanges: false,
    }),
    getters: {
        getButtonConfig: (state) => (id: string | number): ButtonConfig | undefined => {
            // Find button by id, as index might not be reliable if array is sparse or reordered
            return state.buttons.find(b => b.id === id);
        },
        isDirty: (state): boolean => state.hasUnsavedChanges,
        // This should return the data structure expected for saving (MappingConfiguration and button_names)
        // For now, it returns ButtonConfig[], deviceStore will transform it.
        // OR, this store could be responsible for creating the MappingConfiguration and button_names directly.
        // Let's assume for now deviceStore handles the transformation back for saving.
        getConfigForSave: (state): ButtonConfig[] => {
            return state.buttons;
        },
    },
    actions: {
        loadConfig(config: ButtonConfig[]) { // Expects the transformed ButtonConfig array
            this.buttons = JSON.parse(JSON.stringify(config)); // Deep copy
            this.hasUnsavedChanges = false;
            console.log('MacroPad config loaded into store');
        },
        updateButtonConfig(buttonId: string | number, newConfig: Partial<ButtonConfig>) {
            const buttonIndex = this.buttons.findIndex(b => b.id === buttonId);
            if (buttonIndex !== -1) {
                this.buttons[buttonIndex] = { ...this.buttons[buttonIndex], ...newConfig };
                this.hasUnsavedChanges = true;
            } else {
                // Option to add if not found, or log error
                console.warn(`Button with id ${buttonId} not found for update.`);
                // To add if not found (example):
                // this.buttons.push({ id: buttonId, ...newConfig } as ButtonConfig);
                // this.hasUnsavedChanges = true;
            }
        },
        addButton(config: ButtonConfig) {
            // Ensure unique ID if necessary, or let an external process handle ID generation.
            // For now, assumes config comes with a valid ID.
            if (this.buttons.some(b => b.id === config.id)) {
                console.warn(`Button with id ${config.id} already exists. Update instead?`);
                return; // Or update if that's the desired behavior
            }
            this.buttons.push(config);
            this.hasUnsavedChanges = true;
        },
        removeButton(buttonId: string | number) {
            this.buttons = this.buttons.filter(b => b.id !== buttonId);
            this.hasUnsavedChanges = true;
        },
        resetChanges() {
            console.log('MacroPad changes (simulated reset) - flag cleared, data needs reload from deviceStore');
            this.hasUnsavedChanges = false;
        },
        markAsSaved() {
            this.hasUnsavedChanges = false;
            console.log('MacroPad config marked as saved in store');
        },
    },
}); 