import { defineStore } from 'pinia';
import type { ConfigAction, MappingConfiguration } from '@/types/protocol'; // Import ConfigAction and MappingConfiguration
import { isEqual } from 'lodash-es'; // For deep equality checks

// ButtonConfig type is now effectively represented by the combination of
// a key in mappings (e.g., "0", "1") and an entry in button_names (e.g., 0: "My Button")
// For UI purposes, if a combined view is needed, it can be a computed property or a local component structure.
// We remove the old ButtonConfig type from here as the store's state will directly use MappingConfiguration and Record<number, string>.

const defaultMappings: () => MappingConfiguration = () => ({});
const defaultButtonNames: () => Record<number, string> = () => ({});

export const useMacroPadConfigStore = defineStore('macroPadConfig', {
    state: () => ({
        mappings: defaultMappings(),
        button_names: defaultButtonNames(),
        initialMappings: defaultMappings(),      // Store initial state for comparison
        initialButtonNames: defaultButtonNames(), // Store initial state for comparison
        hasUnsavedChanges: false, // Kept for now, but isDirty is the primary source of truth
    }),
    getters: {
        getMappings: (state): MappingConfiguration => state.mappings,
        getButtonName: (state) => (buttonId: number): string => state.button_names[buttonId] || '',
        // buttonKey is typically the string representation of buttonId, e.g., "0", "1", ... "11"
        getButtonActions: (state) => (buttonKey: string): ConfigAction[] | undefined => state.mappings[buttonKey],

        getMacroPadConfigForSave: (state): { mappings: MappingConfiguration; button_names: Record<number, string> } => {
            return {
                mappings: state.mappings,
                button_names: state.button_names,
            };
        },

        // Granular change detection getters
        isButtonNameChanged(state) {
            return (buttonId: number): boolean => {
                const initialName = state.initialButtonNames[buttonId] || '';
                const currentName = state.button_names[buttonId] || '';
                return initialName !== currentName;
            };
        },
        areButtonActionsChanged(state) {
            return (buttonKey: string): boolean => {
                const initialActions = state.initialMappings[buttonKey] || [];
                const currentActions = state.mappings[buttonKey] || [];
                return !isEqual(initialActions, currentActions);
            };
        },
        getChangedButtonKeys(state): string[] {
            const changedKeys: string[] = [];
            const allKeys = new Set([...Object.keys(state.mappings), ...Object.keys(state.initialMappings)]);

            for (const key of allKeys) {
                const numericId = parseInt(key, 10);
                let nameChanged = false;
                if (!isNaN(numericId)) {
                    nameChanged = (state.initialButtonNames[numericId] || '') !== (state.button_names[numericId] || '');
                }
                const actionsChanged = !isEqual(state.initialMappings[key] || [], state.mappings[key] || []);

                if (nameChanged || actionsChanged) {
                    changedKeys.push(key);
                }
            }
            return changedKeys;
        },

        isDirty(): boolean { // Removed state parameter, uses `this` context
            return this.getChangedButtonKeys.length > 0;
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
            const mappingsCopy = JSON.parse(JSON.stringify(newMappings || defaultMappings()));
            const buttonNamesCopy = JSON.parse(JSON.stringify(newButtonNames || defaultButtonNames()));

            this.mappings = mappingsCopy;
            this.initialMappings = JSON.parse(JSON.stringify(mappingsCopy));

            this.button_names = buttonNamesCopy;
            this.initialButtonNames = JSON.parse(JSON.stringify(buttonNamesCopy));

            this.hasUnsavedChanges = false; // isDirty will be false after this
            console.log('MacroPad config loaded and initial state set', this.mappings, this.button_names);
        },

        updateButtonName(buttonId: number, name: string) {
            if ((this.button_names[buttonId] || '') !== name) {
                this.button_names[buttonId] = name;
            }
        },

        updateButtonActions(buttonKey: string, actions: ConfigAction[]) {
            // Use isEqual for a more robust check if needed, or assume update implies change
            this.mappings[buttonKey] = JSON.parse(JSON.stringify(actions));
        },

        clearButtonConfig(buttonId: number) {
            const buttonKey = String(buttonId);
            this.button_names[buttonId] = '';
            this.mappings[buttonKey] = [];
        },

        resetChanges() {
            this.mappings = JSON.parse(JSON.stringify(this.initialMappings || defaultMappings()));
            this.button_names = JSON.parse(JSON.stringify(this.initialButtonNames || defaultButtonNames()));
            console.log('MacroPad config reset to initial loaded state');
        },

        markAsSaved() {
            this.initialMappings = JSON.parse(JSON.stringify(this.mappings));
            this.initialButtonNames = JSON.parse(JSON.stringify(this.button_names));
            this.hasUnsavedChanges = false; // isDirty will be false after this
            console.log('MacroPad config current state marked as saved (initial state updated)');
        },
    },
}); 