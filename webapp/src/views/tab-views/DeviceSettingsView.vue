<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useDeviceSettingsStore } from '@/stores/deviceSettingsStore';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

const deviceSettingsStore = useDeviceSettingsStore();

const localWifiSsid = ref(deviceSettingsStore.settings.wifi?.ssid || '');
const localWifiPassword = ref(deviceSettingsStore.settings.wifi?.password || '');
const localTimezoneOffsetInput = ref<string>(deviceSettingsStore.settings.timezone_offset?.toString() || '');
const localApiKey = ref<string>(deviceSettingsStore.settings.api_key || '');

const isWifiConfigured = computed(() => !!deviceSettingsStore.settings.wifi);

watch(localWifiSsid, (newSsid) => {
  deviceSettingsStore.updateWifiSsid(newSsid);
});

watch(localWifiPassword, (newPassword) => {
  deviceSettingsStore.updateWifiPassword(newPassword);
});

watch(localTimezoneOffsetInput, (newValStr) => {
  const trimmedVal = newValStr.trim();
  if (trimmedVal === '') {
    deviceSettingsStore.updateTimezoneOffset(null);
  } else {
    const numVal = parseFloat(trimmedVal);
    if (!isNaN(numVal)) {
      deviceSettingsStore.updateTimezoneOffset(numVal);
    } else {
      // Optional: handle invalid number input, e.g., by not updating or setting error
    }
  }
});

watch(localApiKey, (newKey: string) => {
  deviceSettingsStore.updateApiKey(newKey.trim() === '' ? null : newKey);
});

watch(() => deviceSettingsStore.settings, (newSettings) => {
  localWifiSsid.value = newSettings.wifi?.ssid || '';
  localWifiPassword.value = newSettings.wifi?.password || '';
  localTimezoneOffsetInput.value = newSettings.timezone_offset?.toString() || '';
  localApiKey.value = newSettings.api_key || '';
}, { deep: true });

const handleClearWifi = () => {
  deviceSettingsStore.clearWifiSettings();
};

</script>

<template>
  <div class="p-4 space-y-6">
    <div class="p-4 border rounded-md">
      <h3 class="text-lg font-semibold mb-2">WiFi Configuration</h3>
      <p class="text-sm text-muted-foreground mb-4">Connect your ESP Deck to a WiFi network. Leave empty to disable WiFi.</p>
      <div class="space-y-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 items-center">
          <Label for="wifi-ssid">SSID (Network Name)</Label>
          <Input id="wifi-ssid" v-model="localWifiSsid" placeholder="Your WiFi SSID" />
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 items-center">
          <Label for="wifi-password">Password</Label>
          <Input id="wifi-password" type="password" v-model="localWifiPassword" placeholder="Your WiFi Password" />
        </div>
      </div>
      <div v-if="isWifiConfigured" class="mt-4">
        <Button variant="destructive" @click="handleClearWifi">Clear WiFi Settings</Button>
      </div>
    </div>

    <div class="p-4 border rounded-md">
      <h3 class="text-lg font-semibold mb-2">Timezone</h3>
      <p class="text-sm text-muted-foreground mb-4">Set the timezone offset from UTC in hours (e.g., -5 for EST, 2 for EET).</p>
      <div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 items-center">
          <Label for="timezone-offset">Timezone Offset (hours)</Label>
          <Input id="timezone-offset" type="text" v-model="localTimezoneOffsetInput" placeholder="e.g., -5 or 2 (empty for none)" />
        </div>
      </div>
    </div>

    <div class="p-4 border rounded-md">
      <h3 class="text-lg font-semibold mb-2">API Key</h3>
      <p class="text-sm text-muted-foreground mb-4">Optional API key for accessing external services (if implemented by a feature).</p>
      <div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 items-center">
          <Label for="api-key">API Key</Label>
          <Input id="api-key" type="text" v-model="localApiKey" placeholder="Optional API Key (empty for none)" />
        </div>
      </div>
    </div>

    <div class="text-sm text-muted-foreground pt-4">
      Note: Changes made here are applied to the current session. Click "Save Settings" in the top bar to persist them to the device.
      The `hasUnsavedChanges` flag in the store is: {{ deviceSettingsStore.hasUnsavedChanges }}
    </div>
  </div>
</template> 