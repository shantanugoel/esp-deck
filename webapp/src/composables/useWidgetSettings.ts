import { ref, computed } from 'vue';
import type {
    DeviceConfigFE,
    WidgetsState,
    WidgetItemConfigFE,
    WidgetKindConfigFE,
    WidgetFormState,
    WidgetsConfigPayload,
} from '@/types/deviceConfig';
import { deviceService } from '@/services/deviceService';
// import { useToast } from '@/components/ui/toast/use-toast'; // Assuming this will be available

export function useWidgetSettings() {
    // const { toast } = useToast(); // Comment out if toast is not yet set up
    const toast = (options: any) => { // Placeholder toast
        console.log('Toast:', options.title, options.description);
        if (options.variant === 'destructive') console.error('Error Toast:', options.title, options.description);
    };
    const isLoading = ref(false);
    const isSaving = ref(false);

    // Holds the full device configuration, including widgets
    const deviceConfig = ref<DeviceConfigFE | null>(null);

    // Reactive state for widgets being actively added/edited/deleted
    // This will be the payload sent to the backend
    const pendingWidgetChanges = ref<WidgetsConfigPayload>({});

    // The current state of widgets as known by the frontend (after fetch/save)
    const currentWidgets = computed<WidgetsState>(() => deviceConfig.value?.widgets || {});

    // A combined view of current widgets overlaid with pending changes for UI display
    const displayWidgets = computed<WidgetsState>(() => {
        const combined = { ...currentWidgets.value };
        for (const stringId in pendingWidgetChanges.value) {
            const id = Number(stringId);
            const change = pendingWidgetChanges.value[id];
            if (change === null) { // Marked for deletion
                delete combined[id];
            } else if (change) { // Added or updated
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

    const loadDeviceConfig = async () => {
        if (!deviceService.isConnected) {
            toast({ title: 'Device Not Connected', description: 'Please connect to the device first.', variant: 'destructive' });
            isLoading.value = false;
            return;
        }
        isLoading.value = true;
        pendingWidgetChanges.value = {};
        try {
            const response = await deviceService.fetchConfig();
            if (response.success && response.data) {
                // Assuming response.data is compatible with DeviceConfigFE
                // The backend now sends Option<HashMap<usize, WidgetItemConfig>> for widgets (cleaned)
                // So, deviceConfig.value.widgets should be WidgetsState (Record<number, WidgetItemConfigFE>)
                deviceConfig.value = response.data as DeviceConfigFE;
            } else {
                throw new Error(response.error || 'Failed to fetch configuration from device.');
            }
        } catch (error: any) {
            console.error('Failed to load device configuration:', error);
            toast({
                title: 'Error Loading Configuration',
                description: error.message || 'Could not fetch settings from the device.',
                variant: 'destructive',
            });
            deviceConfig.value = null;
        }
        isLoading.value = false;
    };

    const getNextWidgetId = (): number => {
        const currentIds = Object.keys(currentWidgets.value).map(Number);
        const pendingAddedIds = Object.keys(pendingWidgetChanges.value)
            .filter(id => pendingWidgetChanges.value[Number(id)] !== null && !(Number(id) in currentWidgets.value))
            .map(Number);
        const allKnownIds = [...currentIds, ...pendingAddedIds];
        return allKnownIds.length > 0 ? Math.max(...allKnownIds) + 1 : 0;
    };

    const stageWidgetChange = (id: number, item: WidgetItemConfigFE | null) => {
        pendingWidgetChanges.value = {
            ...pendingWidgetChanges.value,
            [id]: item,
        };
    };

    const addWidget = (formState: WidgetFormState): boolean => {
        if (formState.title.trim() === '' || formState.url.trim() === '') {
            toast({ title: 'Validation Error', description: 'Title and URL are required.', variant: 'destructive' });
            return false;
        }
        const newId = getNextWidgetId();
        const newWidgetItem = convertFormToWidgetItem(formState);
        stageWidgetChange(newId, newWidgetItem);
        toast({ title: 'Widget Staged', description: `Widget '${newWidgetItem.title}' will be added with ID ${newId} upon saving.` });
        return true;
    };

    const updateWidget = (id: number, formState: WidgetFormState): boolean => {
        if (!(id in displayWidgets.value)) { // Check against combined display state
            toast({ title: 'Error', description: `Widget with ID ${id} not found for update.`, variant: 'destructive' });
            return false;
        }
        if (formState.title.trim() === '' || formState.url.trim() === '') {
            toast({ title: 'Validation Error', description: 'Title and URL are required.', variant: 'destructive' });
            return false;
        }
        const updatedWidgetItem = convertFormToWidgetItem(formState);
        stageWidgetChange(id, updatedWidgetItem);
        toast({ title: 'Widget Update Staged', description: `Changes to widget '${updatedWidgetItem.title}' (ID: ${id}) are staged for saving.` });
        return true;
    };

    const deleteWidget = (id: number) => {
        const widgetToDelete = displayWidgets.value[id];
        if (!widgetToDelete) {
            toast({ title: 'Error', description: `Widget with ID ${id} not found for deletion.`, variant: 'destructive' });
            return;
        }
        stageWidgetChange(id, null); // Mark for deletion by setting to null
        toast({ title: 'Widget Deletion Staged', description: `Widget '${widgetToDelete.title}' (ID: ${id}) will be deleted upon saving.` });
    };

    const saveWidgetChanges = async () => {
        if (!deviceService.isConnected) {
            toast({ title: 'Device Not Connected', description: 'Please connect to the device to save changes.', variant: 'destructive' });
            isSaving.value = false;
            return;
        }
        if (!hasPendingChanges.value) {
            toast({ title: 'No Changes', description: 'There are no widget changes to save.' });
            return;
        }
        isSaving.value = true;

        // Construct the payload for the backend. It expects a map of widget ID to Option<WidgetItemConfig>.
        // The keys in WidgetsConfigPayload are numbers, but the backend custom deserializer handles string keys from JSON.
        // The deviceService.saveConfig likely expects a FullDeviceConfig compatible structure.
        // We need to merge pending changes into the current full config.

        const newConfigToSave: DeviceConfigFE = JSON.parse(JSON.stringify(deviceConfig.value || { settings: {}, mappings: {}, button_names: {}, widgets: {} }));
        if (!newConfigToSave.widgets) {
            newConfigToSave.widgets = {};
        }

        // Apply pending changes to a temporary full widget state that includes nulls for deletion
        let mergedWidgetsForPayload: Record<number, WidgetItemConfigFE | null> = { ...newConfigToSave.widgets };
        // First, incorporate all existing widgets from the potentially modified newConfigToSave.widgets
        // This ensures we respect any existing widgets not part of pending changes.
        for (const idStr in newConfigToSave.widgets) {
            const id = Number(idStr);
            if (!(id in pendingWidgetChanges.value)) { // If not in pending, keep it as is
                mergedWidgetsForPayload[id] = newConfigToSave.widgets[id];
            }
        }
        // Then, apply pending changes
        for (const idStr in pendingWidgetChanges.value) {
            const id = Number(idStr);
            mergedWidgetsForPayload[id] = pendingWidgetChanges.value[id];
        }

        // The backend's DeviceConfig.widgets is Option<HashMap<usize, Option<WidgetItemConfig>>>
        // So, the payload needs to be Record<string, WidgetItemConfigFE | null> for the `setConfig` on the device side if it does direct mapping.
        // Let's prepare the `widgets` part of the config to be sent.
        const widgetsPayloadForDevice: Record<string, WidgetItemConfigFE | null> = {};
        for (const numKey in mergedWidgetsForPayload) {
            widgetsPayloadForDevice[String(numKey)] = mergedWidgetsForPayload[numKey];
        }

        // Reconstruct the full config with the updated widgets map that includes nulls
        const finalConfigPayload = {
            ...newConfigToSave, // This includes settings, mappings, button_names
            widgets: widgetsPayloadForDevice // This is Record<string, WidgetItemConfigFE | null>
        };

        try {
            // deviceService.saveConfig expects FullDeviceConfig. Ensure compatibility.
            // The type casting here assumes FullDeviceConfig is structurally compatible with what we built.
            const response = await deviceService.saveConfig(finalConfigPayload as any);
            if (response.success) {
                toast({ title: 'Settings Saved', description: 'Widget changes have been sent to the device.' });
                await loadDeviceConfig(); // Reload to get fresh state and clear pending changes
            } else {
                throw new Error(response.error || 'Failed to save configuration to device.');
            }
        } catch (error: any) {
            console.error('Failed to save widget settings:', error);
            toast({
                title: 'Error Saving Settings',
                description: error.message || 'Could not save widget settings to the device.',
                variant: 'destructive',
            });
        }
        isSaving.value = false;
    };

    const discardWidgetChanges = () => {
        pendingWidgetChanges.value = {};
        toast({ title: 'Changes Discarded', description: 'Pending widget changes have been cleared.' });
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
        isSaving,
        deviceConfig, // Expose for other settings if needed
        currentWidgets, // Raw state from backend
        pendingWidgetChanges,
        displayWidgets, // Combined state for UI
        widgetListForDisplay, // Sorted array of displayWidgets
        hasPendingChanges,
        loadDeviceConfig,
        addWidget,
        updateWidget,
        deleteWidget,
        saveWidgetChanges,
        discardWidgetChanges,
        convertWidgetItemToForm,
        getNextWidgetId, // Expose if needed for form pre-population
    };
} 