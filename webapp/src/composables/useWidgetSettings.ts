import { ref, computed, watch } from 'vue';
import type {
    // DeviceConfigFE, // No longer directly managing full device config here
    WidgetsState,
    WidgetItemConfigFE,
    WidgetKindConfigFE,
    WidgetFormState,
    WidgetsConfigPayload,
} from '@/types/deviceConfig';
import type { FullDeviceConfig } from '@/types/protocol'; // For type hint on init

// import { deviceService } from '@/services/deviceService'; // No longer using deviceService directly
// import { useToast } from '@/components/ui/toast/use-toast';

// --- Singleton State --- 
// Define state outside the function to make it singleton
const currentWidgets = ref<WidgetsState>({});
const pendingWidgetChanges = ref<WidgetsConfigPayload>({});

// Initial console logs to see when this module-level state is first created
console.log('[useWidgetSettings Module] Initial currentWidgets:', JSON.parse(JSON.stringify(currentWidgets.value)));
console.log('[useWidgetSettings Module] Initial pendingWidgetChanges:', JSON.parse(JSON.stringify(pendingWidgetChanges.value)));

// --- Exported Composable Function --- 
export function useWidgetSettings() {
    // Toast function can remain instance-specific if not using a global toast provider
    const toast = (options: any) => {
        console.log('Toast (from useWidgetSettings instance):', options.title, options.description);
        if (options.variant === 'destructive') console.error('Error Toast (from useWidgetSettings instance):', options.title, options.description);
    };

    // Computed properties now refer to the module-level state
    const displayWidgets = computed<WidgetsState>(() => {
        const combined = { ...currentWidgets.value };
        for (const stringId in pendingWidgetChanges.value) {
            const id = Number(stringId);
            const change = pendingWidgetChanges.value[stringId];
            if (change === null) {
                delete combined[id];
            } else if (change) {
                combined[id] = change;
            }
        }
        return combined;
    });

    const widgetListForDisplay = computed(() =>
        Object.entries(displayWidgets.value)
            .map(([id, config]) => ({ id: parseInt(id, 10), ...config }))
            .sort((a, b) => a.id - b.id)
    );

    const hasPendingChanges = computed(() => Object.keys(pendingWidgetChanges.value).length > 0);

    // Watchers can be set up here as well, they will be attached once
    // but react to changes in the singleton state.
    // However, for debugging, having them inside the function call path is also fine if they re-evaluate correctly.
    // Let's keep them here for now to see their behavior with singleton state.
    watch(widgetListForDisplay, (newList) => {
        console.log('[useWidgetSettings Singleton] widgetListForDisplay CHANGED:', JSON.parse(JSON.stringify(newList)));
    }, { deep: true });

    watch(currentWidgets, (newVal) => {
        console.log('[useWidgetSettings Singleton] currentWidgets CHANGED:', JSON.parse(JSON.stringify(newVal)));
    }, { deep: true });

    watch(pendingWidgetChanges, (newVal) => {
        console.log('[useWidgetSettings Singleton] pendingWidgetChanges CHANGED:', JSON.parse(JSON.stringify(newVal)));
    }, { deep: true });

    // This computed also needs to be tied to the singleton pendingWidgetChanges
    const loggingHasPendingChanges = computed(() => { // Renamed to avoid conflict if hasPendingChanges above is preferred
        const pending = Object.keys(pendingWidgetChanges.value).length > 0;
        console.log('[useWidgetSettings Singleton] hasPendingChanges computed:', pending, 'Pending obj:', JSON.parse(JSON.stringify(pendingWidgetChanges.value)));
        return pending;
    });


    // Methods now operate on the singleton state
    const initializeWidgets = (config: FullDeviceConfig | null) => {
        console.log('[useWidgetSettings Singleton] initializeWidgets called. Received config:', config ? JSON.parse(JSON.stringify(config)) : null);
        console.log('[useWidgetSettings Singleton] Config widgets part:', config?.widgets ? JSON.parse(JSON.stringify(config.widgets)) : null);
        pendingWidgetChanges.value = {}; // Reset pending changes for the singleton state
        const newCurrentWidgets: WidgetsState = {};
        if (config && config.widgets) {
            for (const stringId in config.widgets) {
                const widgetConf = config.widgets[stringId];
                if (widgetConf !== null) {
                    newCurrentWidgets[Number(stringId)] = widgetConf as WidgetItemConfigFE;
                }
            }
        }
        console.log('[useWidgetSettings Singleton] Parsed newCurrentWidgets:', JSON.parse(JSON.stringify(newCurrentWidgets)));
        currentWidgets.value = newCurrentWidgets; // Update the singleton state
        console.log('[useWidgetSettings Singleton] currentWidgets.value after assignment:', JSON.parse(JSON.stringify(currentWidgets.value)));
    };

    const getNextWidgetId = (): number => {
        const currentIds = Object.keys(currentWidgets.value).map(Number);
        const pendingAddedIds = Object.entries(pendingWidgetChanges.value)
            .filter(([id, item]) => item !== null && !(Number(id) in currentWidgets.value))
            .map(([id]) => Number(id));
        const allKnownIds = [...currentIds, ...pendingAddedIds];
        const displayWidgetIds = Object.keys(displayWidgets.value).map(Number);
        if (displayWidgetIds.length === 0) return 0;
        return Math.max(...displayWidgetIds) + 1;
    };

    const stageWidgetChange = (id: number, item: WidgetItemConfigFE | null) => {
        pendingWidgetChanges.value = {
            ...pendingWidgetChanges.value,
            [String(id)]: item,
        };
        console.log(`[useWidgetSettings Singleton] stageWidgetChange: ID ${id}, Item:`, item ? JSON.parse(JSON.stringify(item)) : null, 'Pending:', JSON.parse(JSON.stringify(pendingWidgetChanges.value)));
    };

    const addWidget = (formState: WidgetFormState): boolean => {
        if (formState.title.trim() === '' || formState.url.trim() === '') {
            toast({ title: 'Validation Error', description: 'Title and URL are required.', variant: 'destructive' });
            return false;
        }
        const newId = getNextWidgetId();
        const newWidgetItem = convertFormToWidgetItem(formState);
        stageWidgetChange(newId, newWidgetItem);
        toast({ title: 'Widget Staged', description: `Widget '${newWidgetItem.title}' (ID ${newId}) is staged.` });
        console.log('[useWidgetSettings Singleton] addWidget. Pending:', JSON.parse(JSON.stringify(pendingWidgetChanges.value)));
        // Trigger the logging computed to see its value
        console.log('[useWidgetSettings Singleton] addWidget: loggingHasPendingChanges.value:', loggingHasPendingChanges.value);
        return true;
    };

    const updateWidget = (id: number, formState: WidgetFormState): boolean => {
        if (!(id in displayWidgets.value)) {
            toast({ title: 'Error', description: `Widget with ID ${id} not found for update.`, variant: 'destructive' });
            return false;
        }
        if (formState.title.trim() === '' || formState.url.trim() === '') {
            toast({ title: 'Validation Error', description: 'Title and URL are required.', variant: 'destructive' });
            return false;
        }
        const updatedWidgetItem = convertFormToWidgetItem(formState);
        stageWidgetChange(id, updatedWidgetItem);
        toast({ title: 'Widget Update Staged', description: `Changes to widget '${updatedWidgetItem.title}' (ID ${id}) are staged.` });
        return true;
    };

    const deleteWidget = (id: number) => {
        const widgetToDelete = displayWidgets.value[id];
        if (!widgetToDelete) {
            toast({ title: 'Error', description: `Widget with ID ${id} not found.`, variant: 'destructive' });
            return;
        }
        stageWidgetChange(id, null);
        toast({ title: 'Widget Deletion Staged', description: `Widget '${widgetToDelete.title}' (ID ${id}) will be deleted upon saving.` });
    };

    const getPendingWidgetConfigForSave = (): WidgetsConfigPayload | null => {
        if (!hasPendingChanges.value) return null;
        return JSON.parse(JSON.stringify(pendingWidgetChanges.value));
    };

    const markWidgetsAsSaved = (successfullySavedWidgetsPayload: WidgetsConfigPayload) => {
        console.log('[useWidgetSettings Singleton] markWidgetsAsSaved called with:', JSON.parse(JSON.stringify(successfullySavedWidgetsPayload)));
        const newCurrentWidgetsState = { ...currentWidgets.value };
        const remainingPendingChanges = { ...pendingWidgetChanges.value };
        for (const stringId in successfullySavedWidgetsPayload) {
            const id = Number(stringId);
            const savedItem = successfullySavedWidgetsPayload[stringId];
            if (savedItem === null) {
                delete newCurrentWidgetsState[id];
            } else {
                newCurrentWidgetsState[id] = savedItem;
            }
            delete remainingPendingChanges[stringId];
        }
        currentWidgets.value = newCurrentWidgetsState;
        pendingWidgetChanges.value = remainingPendingChanges;
        toast({ title: 'Widget Settings Synced', description: 'Local widget state updated after save.' });
        console.log('[useWidgetSettings Singleton] markWidgetsAsSaved finished. currentWidgets:', JSON.parse(JSON.stringify(currentWidgets.value)), 'pendingChanges:', JSON.parse(JSON.stringify(pendingWidgetChanges.value)));
    };

    const revertWidgetChanges = () => {
        pendingWidgetChanges.value = {};
        toast({ title: 'Widget Changes Discarded', description: 'Pending widget modifications cleared.' });
    };

    // Helper to convert form state to API-compatible widget item
    function convertFormToWidgetItem(formState: WidgetFormState): WidgetItemConfigFE {
        let kind: WidgetKindConfigFE;
        if (formState.type === 'Text') {
            kind = { Text: [formState.url, formState.isJson ? (formState.jsonPointer?.trim() || null) : null] };
        } else { // Image
            kind = { Image: formState.url };
        }
        return {
            title: formState.title.trim(),
            kind,
            update_interval_seconds: Number(formState.update_interval_seconds) || 0,
        };
    }

    // Helper to convert API widget item to form state for editing
    function convertWidgetItemToForm(id: number, widgetItem: WidgetItemConfigFE): WidgetFormState {
        let type: 'Text' | 'Image' = 'Text';
        let url = '';
        let jsonPointer: string | null = null;
        let isJson = false;

        if ('Text' in widgetItem.kind) {
            type = 'Text';
            url = widgetItem.kind.Text[0];
            jsonPointer = widgetItem.kind.Text[1];
            // Ensure isJson is true only if jsonPointer is a non-empty string
            isJson = typeof jsonPointer === 'string' && jsonPointer.trim() !== '';
        } else if ('Image' in widgetItem.kind) {
            type = 'Image';
            url = widgetItem.kind.Image;
        }
        // console.log(`[useWidgetSettings] Converting widget ID ${id} to form:`, { id, title: widgetItem.title, type, url, jsonPointer, update_interval_seconds: widgetItem.update_interval_seconds, isJson });
        return {
            id, // Keep the original ID for context, though forms might not always need it displayed
            title: widgetItem.title,
            type,
            url,
            jsonPointer: jsonPointer, // Keep null if it was null
            update_interval_seconds: widgetItem.update_interval_seconds,
            isJson,
        };
    }

    // Return the singleton refs and methods
    return {
        currentWidgets, // Expose the raw ref for direct observation if needed, though displayWidgets is primary for UI list
        pendingWidgetChanges, // Expose for observation
        widgetListForDisplay,
        hasPendingChanges: loggingHasPendingChanges, // Use the logging version for debug
        displayWidgets,
        addWidget,
        updateWidget,
        deleteWidget,
        convertWidgetItemToForm,
        initializeWidgets,
        getPendingWidgetConfigForSave,
        markWidgetsAsSaved,
        revertWidgetChanges,
    };
} 