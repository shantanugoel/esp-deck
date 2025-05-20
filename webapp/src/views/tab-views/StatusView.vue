<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { useDeviceStore } from '@/stores/deviceStore';
import { useDeviceSettingsStore } from '@/stores/deviceSettingsStore';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';

const deviceStore = useDeviceStore();
const deviceSettingsStore = useDeviceSettingsStore();

const currentTime = ref(new Date().toLocaleTimeString());
let timerInterval: number | undefined;

onMounted(() => {
  timerInterval = window.setInterval(() => {
    currentTime.value = new Date().toLocaleTimeString();
  }, 1000);
});

onUnmounted(() => {
  if (timerInterval) {
    clearInterval(timerInterval);
  }
});

const connectedWifiSsid = computed(() => {
  if (deviceStore.isConnected && deviceSettingsStore.settings.wifi?.ssid) {
    return deviceSettingsStore.settings.wifi.ssid;
  }
  return 'N/A';
});

// Placeholder for IP Address - this would typically come from the device if it reports it
const deviceIpAddress = computed(() => {
  if (deviceStore.isConnected && deviceSettingsStore.settings.wifi?.ssid) {
    // This is a placeholder. Actual IP would need to be fetched from the device.
    // For demonstration, we can show a mock IP or a message.
    return '192.168.1.100 (Example)'; 
  }
  return 'N/A';
});

const deviceUptime = computed(() => {
  // Placeholder for device uptime
  return 'N/A (Not Implemented)';
});

const deviceFreeMemory = computed(() => {
  // Placeholder for device free memory
  return 'N/A (Not Implemented)';
});

</script>

<template>
  <div class="p-4 space-y-6">
    <h2 class="text-2xl font-semibold tracking-tight">Device Status</h2>
    <p class="text-sm text-muted-foreground">
      Current status information from your ESP Deck.
    </p>

    <div class="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Clock</CardTitle>
          <CardDescription>Current local time.</CardDescription>
        </CardHeader>
        <CardContent>
          <p class="text-3xl font-bold">{{ currentTime }}</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Network Status</CardTitle>
          <CardDescription>WiFi connection details.</CardDescription>
        </CardHeader>
        <CardContent class="space-y-2">
          <div class="flex justify-between">
            <span class="text-muted-foreground">Status:</span>
            <span :class="{'text-green-500': deviceStore.isConnected && deviceSettingsStore.settings.wifi?.ssid, 'text-red-500': !deviceStore.isConnected || !deviceSettingsStore.settings.wifi?.ssid}">
              {{ deviceStore.isConnected && deviceSettingsStore.settings.wifi?.ssid ? 'Connected' : 'Not Connected' }}
            </span>
          </div>
          <Separator />
          <div class="flex justify-between">
            <span class="text-muted-foreground">SSID:</span>
            <span>{{ connectedWifiSsid }}</span>
          </div>
          <Separator />
          <div class="flex justify-between">
            <span class="text-muted-foreground">IP Address:</span>
            <span>{{ deviceIpAddress }}</span>
          </div>
        </CardContent>
      </Card>

      <Card class="md:col-span-2">
        <CardHeader>
          <CardTitle>Device Diagnostics</CardTitle>
          <CardDescription>Real-time device information (placeholders).</CardDescription>
        </CardHeader>
        <CardContent class="space-y-2">
          <div class="flex justify-between">
            <span class="text-muted-foreground">Device Uptime:</span>
            <span>{{ deviceUptime }}</span>
          </div>
          <Separator />
           <div class="flex justify-between">
            <span class="text-muted-foreground">Free Memory:</span>
            <span>{{ deviceFreeMemory }}</span>
          </div>
          <Separator />
          <div class="flex justify-between">
            <span class="text-muted-foreground">Firmware Version:</span>
            <!-- Assuming deviceStore.deviceInfo might hold this in future -->
            <span>{{ deviceStore.deviceInfo?.firmwareVersion || 'N/A' }}</span>
          </div>
           <Separator />
           <div class="flex justify-between">
            <span class="text-muted-foreground">Device Name:</span>
            <span>{{ deviceStore.deviceInfo?.productName || 'N/A' }}</span>
          </div>
        </CardContent>
      </Card>
    </div>

  </div>
</template> 