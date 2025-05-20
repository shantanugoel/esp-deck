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
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import TopBar from '@/components/core/TopBar.vue';
import BottomBar from '@/components/core/BottomBar.vue';
import TabNavigation from '@/components/core/TabNavigation.vue';
import { useDeviceStore } from '@/stores/deviceStore';
import { useMacroPadConfigStore, type ButtonConfig as MacroPadButtonConfig } from '@/stores/macroPadConfigStore';
import { useDeviceSettingsStore } from '@/stores/deviceSettingsStore';
import { useUiStore } from '@/stores/uiStore';
import type { FullDeviceConfig, MappingConfiguration, DeviceSettings, ConfigAction } from '@/types/protocol';

const deviceStore = useDeviceStore();
const macroPadStore = useMacroPadConfigStore();
const deviceSettingsStore = useDeviceSettingsStore();
const uiStore = useUiStore();

const fileInputRef = ref<HTMLInputElement | null>(null);

const handleConnectToggle = () => {
  if (deviceStore.isConnected) {
    deviceStore.disconnect();
  } else {
    deviceStore.connect();
  }
};

const isSaveRelevant = computed(() => {
  return macroPadStore.isDirty || deviceSettingsStore.isDirty;
});

const handleSaveSettings = async () => {
  if (!isSaveRelevant.value || !deviceStore.isConnected || deviceStore.isLoading) return;

  const settingsToSave: DeviceSettings = deviceSettingsStore.getSettingsForSave;
  const buttonConfigsToSave: MacroPadButtonConfig[] = macroPadStore.getConfigForSave;

  const mappings: MappingConfiguration = {};
  const buttonNames: Record<number, string> = {};

  for (const btn of buttonConfigsToSave) {
    mappings[btn.id.toString()] = btn.actions as ConfigAction[];
    if (btn.name) {
      const numId = parseInt(btn.id.toString(), 10);
      if (!isNaN(numId) && numId > 0) {
        buttonNames[numId - 1] = btn.name;
      }
    }
  }

  const payload: FullDeviceConfig = {
    settings: settingsToSave,
    mappings: mappings,
    button_names: Object.keys(buttonNames).length > 0 ? buttonNames : null,
  };

  await deviceStore.saveConfig(payload);
  if (!deviceStore.error) { 
      macroPadStore.markAsSaved();
      deviceSettingsStore.markAsSaved();
  }
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
  const buttonConfigsForBackup: MacroPadButtonConfig[] = macroPadStore.buttons;

  const mappingsForBackup: MappingConfiguration = {};
  const buttonNamesForBackup: Record<number, string> = {};

  for (const btn of buttonConfigsForBackup) {
    mappingsForBackup[btn.id.toString()] = btn.actions as ConfigAction[];
    if (btn.name) {
      const numId = parseInt(btn.id.toString(), 10);
      if (!isNaN(numId) && numId > 0) {
        buttonNamesForBackup[numId - 1] = btn.name;
      }
    }
  }

  const fullConfigForBackup: FullDeviceConfig = {
    settings: settingsForBackup,
    mappings: mappingsForBackup,
    button_names: Object.keys(buttonNamesForBackup).length > 0 ? buttonNamesForBackup : null,
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