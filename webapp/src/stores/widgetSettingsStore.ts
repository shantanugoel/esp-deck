import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type {
    DeviceConfigFE,
    WidgetsState,
    WidgetItemConfigFE,
    WidgetKindConfigFE,
    WidgetFormState,
    WidgetsConfigPayload,
} from '@/types/deviceConfig';
// import { deviceService } from '@/services/deviceService'; // No longer directly uses deviceService

// Placeholder for toast, replace with actual implementation when available
const toast = (options: { title: string, description: string, variant?: string }) => {
    console.log(`Toast [${options.variant || 'info'}]: ${options.title} - ${options.description}`);
    if (options.variant === 'destructive') console.error(`Error Toast: ${options.title} - ${options.description}`);
};

export const useWidgetSettingsStore = defineStore('widgetSettings', () => {
    const isLoading = ref(true); // Start as true, will be false after config is set
    const deviceConfig = ref<DeviceConfigFE | null>(null);
    const pendingWidgetChanges = ref<WidgetsConfigPayload>({});

    const currentWidgets = computed<WidgetsState>(() => deviceConfig.value?.widgets || {});

    const displayWidgets = computed<WidgetsState>(() => {
        const combined = { ...currentWidgets.value };
        for (const stringId in pendingWidgetChanges.value) {
            const id = Number(stringId);
            const change = pendingWidgetChanges.value[id];
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

    // Action to be called by deviceStore or another central orchestrator
    function setDeviceConfig(newConfig: DeviceConfigFE | null) {
        deviceConfig.value = newConfig;
        pendingWidgetChanges.value = {}; // Reset pending changes as we have a new baseline
        isLoading.value = false;
        if (newConfig) {
            toast({ title: 'Widget Settings Initialized', description: 'Widget configuration has been set.' });
        } else {
            toast({ title: 'Widget Settings Cleared', description: 'Widget configuration is not available.', variant: 'info' });
        }
    }

    function getNextWidgetId(): number {
        const currentIds = Object.keys(currentWidgets.value).map(Number);
        const pendingAddedIds = Object.keys(pendingWidgetChanges.value)
            .filter(id => pendingWidgetChanges.value[Number(id)] !== null && !(Number(id) in currentWidgets.value))
            .map(Number);
        const allKnownIds = [...currentIds, ...pendingAddedIds];
        return allKnownIds.length > 0 ? Math.max(...allKnownIds) + 1 : 0;
    }

    function stageWidgetChange(id: number, item: WidgetItemConfigFE | null) {
        pendingWidgetChanges.value = {
            ...pendingWidgetChanges.value,
            [id]: item,
        };
    }

    function addWidget(formState: WidgetFormState): boolean {
        if (formState.title.trim() === '' || formState.url.trim() === '') {
            toast({ title: 'Validation Error', description: 'Title and URL are required.', variant: 'destructive' });
            return false;
        }
        const newId = getNextWidgetId();
        const newWidgetItem = convertFormToWidgetItem(formState);
        stageWidgetChange(newId, newWidgetItem);
        toast({ title: 'Widget Staged', description: `Widget '${newWidgetItem.title}' will be added with ID ${newId}. Save via main "Save Settings".` });
        return true;
    }

    function updateWidget(id: number, formState: WidgetFormState): boolean {
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
        toast({ title: 'Widget Update Staged', description: `Changes to widget '${updatedWidgetItem.title}' (ID: ${id}) are staged. Save via main "Save Settings".` });
        return true;
    }

    function deleteWidget(id: number) {
        const widgetToDelete = displayWidgets.value[id];
        if (!widgetToDelete) {
            toast({ title: 'Error', description: `Widget with ID ${id} not found for deletion.`, variant: 'destructive' });
            return;
        }
        stageWidgetChange(id, null);
        toast({ title: 'Widget Deletion Staged', description: `Widget '${widgetToDelete.title}' (ID: ${id}) will be deleted. Save via main "Save Settings".` });
    }

    function getPendingWidgetPayloadForSave(): WidgetsConfigPayload | null {
        if (!hasPendingChanges.value) return null;
        return { ...pendingWidgetChanges.value };
    }

    function discardChanges() {
        pendingWidgetChanges.value = {};
        toast({ title: 'Widget Changes Discarded', description: 'Pending widget changes have been cleared.' });
    }

    // Called after global save operation succeeds for widgets
    // This store no longer reloads itself; deviceStore will provide new config via setDeviceConfig
    function markAsSaved() {
        pendingWidgetChanges.value = {}; // Assume changes were successful, new config will arrive
        toast({ title: 'Widget Save Processed', description: 'Pending widget changes cleared. Expecting updated config.' });
        // isLoading can remain false, or be set to true if we want to show loading until new config arrives
        // For now, keep it simple: new config will arrive via setDeviceConfig which handles isLoading.
    }

    // Helper functions (internal to the store)
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

    function convertWidgetItemToForm(id: number, widgetItem: WidgetItemConfigFE): WidgetFormState {
        let type: 'Text' | 'Image' = 'Text';
        let url = '';
        let jsonPointer: string | null = null;
        let isJson = false;

        if ('Text' in widgetItem.kind) {
            type = 'Text';
            url = widgetItem.kind.Text[0];
            jsonPointer = widgetItem.kind.Text[1];
            isJson = jsonPointer !== null && jsonPointer !== '';
        } else if ('Image' in widgetItem.kind) {
            type = 'Image';
            url = widgetItem.kind.Image;
        }

        return {
            id,
            title: widgetItem.title,
            type,
            url,
            jsonPointer,
            update_interval_seconds: widgetItem.update_interval_seconds,
            isJson,
        };
    }

    return {
        isLoading,
        deviceConfig,
        currentWidgets,
        // pendingWidgetChanges, // Exposed only via hasPendingChanges and getPendingWidgetPayloadForSave
        displayWidgets,
        widgetListForDisplay,
        hasPendingChanges,
        setDeviceConfig, // New action
        addWidget,
        updateWidget,
        deleteWidget,
        discardChanges,
        getPendingWidgetPayloadForSave,
        markAsSaved, // Renamed from markAsSavedAndReload
        convertWidgetItemToForm,
        getNextWidgetId,
    };
}); 