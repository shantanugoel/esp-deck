<template>
  <div class="flex flex-col min-h-screen">
    <TopBar />

    <!-- Action Buttons Row -->
    <div class="w-full p-2 bg-background shadow-sm">
      <div class="flex flex-wrap gap-2 justify-center items-center">
        <button
          @click="handleConnectToggle"
          :disabled="deviceStore.isConnecting || deviceStore.isLoading"
          class="px-3 py-1 rounded hover:bg-secondary/80 min-w-[110px] text-sm"
          :class="deviceStore.isConnected ? 'bg-destructive text-destructive-foreground hover:bg-destructive/80' : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'"
        >
          {{ deviceStore.isConnecting ? 'Connecting...' : (deviceStore.isConnected ? 'Disconnect' : 'Connect') }}
        </button>
        <span class="text-sm text-muted-foreground w-full sm:w-auto text-center sm:text-left order-first sm:order-none mb-2 sm:mb-0 sm:ml-4">
          Status: {{ deviceStore.getFormattedStatus }}
        </span>
      </div>
      <div class="flex flex-wrap gap-2 justify-center items-center mt-2">
        <button
          @click="handleSaveSettings"
          :disabled="!deviceStore.isConnected || deviceStore.isLoading || !isSaveRelevant"
          class="px-3 py-1 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/80 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Save Settings
        </button>
        <button
          @click="handleReloadSettings"
          :disabled="!deviceStore.isConnected || deviceStore.isLoading"
          class="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80 border border-input disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Reload Settings
        </button>
        <button
          @click="handleResetSettings"
          :disabled="!deviceStore.isConnected || deviceStore.isLoading"
          class="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80 border border-input disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Reset Settings
        </button>
        <button
          @click="handleRebootDevice"
          :disabled="!deviceStore.isConnected || deviceStore.isLoading"
          class="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80 border border-input disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Reboot Device
        </button>
        <button
          @click="handleBackupDeviceConfig"
          :disabled="!deviceStore.isConnected || deviceStore.isLoading"
          class="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80 border border-input disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Backup Device Config
        </button>
        <button
          @click="handleBackupCurrentConfig"
          :disabled="deviceStore.isLoading"
          class="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80 border border-input disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Backup Current Config
        </button>
        <button
          @click="triggerUpload"
          :disabled="deviceStore.isLoading"
          class="px-3 py-1 text-sm bg-muted text-muted-foreground rounded hover:bg-muted/80 border border-input disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Upload Config
        </button>
      </div>
    </div>
    <input type="file" ref="fileInputRef" @change="handleFileSelected" accept=".json,application/json" style="display: none" />

    <main class="flex-grow w-full px-4 py-8">
      <TabNavigation />
      <div class="mt-4 p-4 border rounded-lg shadow bg-card min-h-[300px]">
        <component :is="uiStore.activeViewComponent" />
      </div>
    </main>
    <BottomBar />
    <DebugLogPanel />

    <ConfirmChangesDialog
      v-model="isConfirmChangesDialogVisible"
      :is-wifi-changed="pendingIsWifiChanged"
      :is-timezone-changed="pendingIsTimezoneChanged"
      :is-api-key-changed="pendingIsApiKeyChanged"
      :is-widgets-changed="pendingIsWidgetsChanged"
      :changed-button-keys="pendingChangedButtonKeys"
      @confirm-save="executeSaveWithSelection"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import TopBar from '@/components/core/TopBar.vue';
import BottomBar from '@/components/core/BottomBar.vue';
import TabNavigation from '@/components/core/TabNavigation.vue';
import ConfirmChangesDialog from '@/components/core/ConfirmChangesDialog.vue';
import DebugLogPanel from '@/components/core/DebugLogPanel.vue';
import type { SaveSelection } from '@/components/core/ConfirmChangesDialog.vue';
import { useDeviceStore } from '@/stores/deviceStore';
import { useMacroPadConfigStore } from '@/stores/macroPadConfigStore';
import { useDeviceSettingsStore } from '@/stores/deviceSettingsStore';
import { useUiStore } from '@/stores/uiStore';
import { useWidgetSettings } from '@/composables/useWidgetSettings';
import type { FullDeviceConfig, MappingConfiguration, DeviceSettings, ConfigAction } from '@/types/protocol';
import type { WidgetsConfigPayload } from '@/types/deviceConfig';

const deviceStore = useDeviceStore();
const macroPadStore = useMacroPadConfigStore();
const deviceSettingsStore = useDeviceSettingsStore();
const uiStore = useUiStore();
const widgetSettings = useWidgetSettings();

const fileInputRef = ref<HTMLInputElement | null>(null);

const isConfirmChangesDialogVisible = ref(false);
const pendingIsWifiChanged = ref(false);
const pendingIsTimezoneChanged = ref(false);
const pendingIsApiKeyChanged = ref(false);
const pendingIsWidgetsChanged = ref(false);
const pendingChangedButtonKeys = ref<string[]>([]);
const MAX_BUTTONS = 16;

const handleConnectToggle = () => {
  if (deviceStore.isConnected) {
    deviceStore.disconnect();
  } else {
    deviceStore.connect();
  }
};

const isSaveRelevant = computed(() => {
  return deviceSettingsStore.isDirty || macroPadStore.isDirty || widgetSettings.hasPendingChanges.value;
});

const handleSaveSettings = async () => {
  if (!isSaveRelevant.value || !deviceStore.isConnected || deviceStore.isLoading) return;

  pendingIsWifiChanged.value = deviceSettingsStore.isWifiChanged;
  pendingIsTimezoneChanged.value = deviceSettingsStore.isTimezoneChanged;
  pendingIsApiKeyChanged.value = deviceSettingsStore.isApiKeyChanged;
  pendingIsWidgetsChanged.value = widgetSettings.hasPendingChanges.value;
  pendingChangedButtonKeys.value = macroPadStore.getChangedButtonKeys;

  isConfirmChangesDialogVisible.value = true;
};

const executeSaveWithSelection = async (selection: SaveSelection) => {
  console.log('Executing save with selection:', selection);
  const lastFetchedConfig = deviceStore.lastFetchedConfig;

  const finalDeviceSettings: Partial<DeviceSettings> = {};
  if (selection.applyWifi) {
    finalDeviceSettings.wifi = deviceSettingsStore.settings.wifi;
  }
  if (selection.applyTimezone) {
    finalDeviceSettings.timezone_offset = deviceSettingsStore.settings.timezone_offset;
  }
  if (selection.applyApiKey) {
    finalDeviceSettings.api_key = deviceSettingsStore.settings.api_key;
  }

  const finalMappingsToSend: MappingConfiguration = {};
  let finalButtonNamesToSend: Record<number, string> | undefined = undefined;

  if (selection.applyMappings) {
    const selectedButtonKeys = Object.keys(selection.buttonsToSave).filter(key => selection.buttonsToSave[key]);
    if (selectedButtonKeys.length > 0) {
      const tempButtonNames: Record<number, string> = {};
      selectedButtonKeys.forEach(buttonKey => {
        finalMappingsToSend[buttonKey] = macroPadStore.getButtonActions(buttonKey) || [];
        const numericId = parseInt(buttonKey, 10);
        const name = macroPadStore.getButtonName(numericId);
        if (name) {
          tempButtonNames[numericId] = name;
        }
      });
      if (Object.keys(tempButtonNames).length > 0) {
        finalButtonNamesToSend = tempButtonNames;
      }
    }
  }

  const payload: { settings: DeviceSettings; mappings: MappingConfiguration; button_names?: Record<number, string>; widgets?: WidgetsConfigPayload } = {
    settings: finalDeviceSettings as DeviceSettings,
    mappings: finalMappingsToSend,
  };

  if (finalButtonNamesToSend !== undefined) {
    payload.button_names = finalButtonNamesToSend;
  }

  if (selection.applyWidgets && widgetSettings.hasPendingChanges.value) {
    const pendingWidgets = widgetSettings.getPendingWidgetConfigForSave();
    if (pendingWidgets) {
      payload.widgets = pendingWidgets as WidgetsConfigPayload;
    }
  }

  console.log("Final payload for save:", JSON.parse(JSON.stringify(payload)));
  const success = await deviceStore.saveConfig(payload as FullDeviceConfig);
  if (success) {
    deviceSettingsStore.markAsSaved({
      wifi: selection.applyWifi,
      timezone: selection.applyTimezone,
      apiKey: selection.applyApiKey,
    });

    const savedButtonKeys = selection.applyMappings 
      ? Object.keys(selection.buttonsToSave).filter(key => selection.buttonsToSave[key]) 
      : [];

    macroPadStore.markAsSaved({
      applyAllButtonChanges: selection.applyMappings,
      buttonKeys: savedButtonKeys,
    });

    if (selection.applyWidgets && payload.widgets) {
      widgetSettings.markWidgetsAsSaved(payload.widgets as WidgetsConfigPayload);
    }

    console.log('Settings saved via dialog, stores selectively marked as saved.');
  } else {
    console.error('Failed to save settings from DefaultLayout via dialog.');
  }
  isConfirmChangesDialogVisible.value = false;
};

const handleReloadSettings = () => {
  if (!deviceStore.isConnected || deviceStore.isLoading) return;
  deviceStore.fetchConfig();
};

const handleResetSettings = () => {
  if (!deviceStore.isConnected || deviceStore.isLoading) return;
  if (confirm('Are you sure you want to reset the device to factory defaults? This will reload settings afterwards.')) {
    deviceStore.resetConfig();
  }
};

const handleRebootDevice = () => {
  if (!deviceStore.isConnected || deviceStore.isLoading) return;
  if (confirm('Are you sure you want to reboot the device?')) {
    deviceStore.rebootDevice();
  }
};

const handleBackupDeviceConfig = () => {
  if (!deviceStore.isConnected || deviceStore.isLoading) return;
  deviceStore.backupDeviceConfig();
};

const handleBackupCurrentConfig = () => {
  if (deviceStore.isLoading) return;
  const settingsForBackup: DeviceSettings = deviceSettingsStore.settings;
  const macroPadConfigForBackup = macroPadStore.getMacroPadConfigForSave;
  const fullConfigForBackup: FullDeviceConfig = {
    settings: settingsForBackup,
    mappings: macroPadConfigForBackup.mappings,
    button_names: Object.keys(macroPadConfigForBackup.button_names).length > 0 ? macroPadConfigForBackup.button_names : null,
  };
  deviceStore.backupCurrentUiConfig(fullConfigForBackup);
};

const triggerUpload = () => {
  if (deviceStore.isLoading) return;
  fileInputRef.value?.click();
};

const handleFileSelected = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files[0]) {
    const file = target.files[0];
    const reader = new FileReader();
    reader.onload = async (e) => {
      const content = e.target?.result;
      if (content && typeof content === 'string') {
        await deviceStore.uploadConfig(content);
      } else {
        alert('Could not read file content or content is not text.');
      }
      if (fileInputRef.value) fileInputRef.value.value = ''; 
    };
    reader.onerror = () => {
      alert('Error reading file.');
      if (fileInputRef.value) fileInputRef.value.value = ''; 
    };
    reader.readAsText(file);
  } else {
    console.log('No file selected for upload.');
  }
};

watch(() => deviceStore.isConnected, (newVal, oldVal) => {
  if (newVal === true && oldVal === false) {
    console.log("Device connected. Fetching initial config...");
    deviceStore.fetchConfig();
  }
});

</script>

<style scoped>
.min-w-\[110px\] { min-width: 110px; }
.min-h-\[300px\] { min-height: 300px; }
</style> 