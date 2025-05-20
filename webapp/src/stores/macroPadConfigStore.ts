import { defineStore } from 'pinia';
import type { ConfigAction, MappingConfiguration } from '@/types/protocol'; // Import ConfigAction and MappingConfiguration

// ButtonConfig type is now effectively represented by the combination of
// a key in mappings (e.g., "0", "1") and an entry in button_names (e.g., 0: "My Button")
// For UI purposes, if a combined view is needed, it can be a computed property or a local component structure.
// We remove the old ButtonConfig type from here as the store's state will directly use MappingConfiguration and Record<number, string>.

export const useMacroPadConfigStore = defineStore('macroPadConfig', {
    state: () => ({
        mappings: {} as MappingConfiguration,
        button_names: {} as Record<number, string>,
        hasUnsavedChanges: false,
    }),
    getters: {
        getMappings: (state): MappingConfiguration => state.mappings,
        getButtonName: (state) => (buttonId: number): string => state.button_names[buttonId] || '',
        // buttonKey is typically the string representation of buttonId, e.g., "0", "1", ... "11"
        getButtonActions: (state) => (buttonKey: string): ConfigAction[] | undefined => state.mappings[buttonKey],
        isDirty: (state): boolean => state.hasUnsavedChanges,

        // Provides the parts of the config this store is responsible for, ready for saving.
        getMacroPadConfigForSave: (state): { mappings: MappingConfiguration; button_names: Record<number, string> } => {
            return {
                mappings: state.mappings,
                button_names: state.button_names,
            };
        },

        // If a component needs a list of "button-like" objects for display (e.g., for v-for)
        // it can compute them. This getter provides an example if needed, but components
        // might be better off using getButtonName and getButtonActions directly with an index.
        // For MacroPadSettingsView, it iterates from 0 to MAX_BUTTONS - 1 and uses those.
        /*
        getButtonListForDisplay: (state) => {
            const maxButtonId = Math.max(...Object.keys(state.button_names).map(Number), ...Object.keys(state.mappings).map(Number).filter(k => !isNaN(k)));
            const list = [];
            for (let i = 0; i <= maxButtonId; i++) {
                const key = String(i);
                if (state.mappings[key] || state.button_names[i]) {
                    list.push({
                        id: i,
                        key: key,
                        name: state.button_names[i] || `Button ${i + 1}`,
                        actions: state.mappings[key] || []
                    });
                }
            }
            return list;
        }
        */
    },
    actions: {
        loadConfig(newMappings: MappingConfiguration, newButtonNames?: Record<number, string> | null) {
            this.mappings = JSON.parse(JSON.stringify(newMappings || {}));
            this.button_names = JSON.parse(JSON.stringify(newButtonNames || {}));
            this.hasUnsavedChanges = false;
            console.log('MacroPad config (mappings and button_names) loaded into store', this.mappings, this.button_names);
        },

        updateButtonName(buttonId: number, name: string) {
            if (this.button_names[buttonId] !== name) {
                this.button_names[buttonId] = name;
                this.hasUnsavedChanges = true;
            }
        },

        updateButtonActions(buttonKey: string, actions: ConfigAction[]) {
            // Simple equality check might not be deep enough for arrays of objects.
            // For robust change detection, a deep comparison or versioning might be needed.
            // For now, assume any call to this intends an update.
            this.mappings[buttonKey] = JSON.parse(JSON.stringify(actions)); // Deep copy actions
            this.hasUnsavedChanges = true;
        },

        // If a button is "removed", it means clearing its name and actions.
        // The device config always expects all 12 button slots in mappings (even if actions are empty)
        // and corresponding names.
        clearButtonConfig(buttonId: number) {
            const buttonKey = String(buttonId);
            this.button_names[buttonId] = ''; // Set name to empty or a default
            this.mappings[buttonKey] = [];    // Set actions to an empty array
            this.hasUnsavedChanges = true;
        },

        resetChanges() {
            // This should trigger a reload from deviceStore, which holds the last fetched config.
            // For now, it just clears the flag. A proper implementation would involve
            // deviceStore re-calling loadConfig on this store.
            console.log('MacroPad changes reset (simulated) - flag cleared, data needs reload from deviceStore');
            this.hasUnsavedChanges = false;
        },

        markAsSaved() {
            this.hasUnsavedChanges = false;
            console.log('MacroPad config marked as saved in store');
        },
    },
}); 