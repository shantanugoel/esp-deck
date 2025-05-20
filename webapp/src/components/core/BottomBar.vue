<template>
  <div class="border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 text-sm">
    <div class="container mx-auto px-4 h-12 flex items-center justify-between">
      <div class="flex items-center space-x-2">
        <Badge :variant="isConnected ? 'default' : 'destructive'" class="capitalize">
          {{ isConnected ? 'Online' : 'Offline' }}
        </Badge>
        <span class="text-muted-foreground">{{ formattedStatus }}</span>
      </div>
      <div class="flex items-center space-x-2">
        <Button variant="outline" size="sm" @click="uiStore.toggleDebugLogVisibility()">
          <ListTreeIcon class="h-4 w-4 mr-1.5" />
          {{ uiStore.isDebugLogVisible ? 'Hide' : 'Show' }} Logs
        </Button>
        <span class="text-muted-foreground text-xs">ESP Deck UI</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDeviceStore } from '@/stores/deviceStore';
import { useUiStore } from '@/stores/uiStore';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ListTreeIcon } from 'lucide-vue-next';

const deviceStore = useDeviceStore();
const uiStore = useUiStore();

const deviceInfo = computed(() => deviceStore.deviceInfo);
const isConnected = computed(() => deviceStore.isConnected);

const formattedStatus = computed(() => {
  if (!isConnected.value) {
    return "Disconnected";
  }
  if (deviceInfo.value?.productName) {
    let status = `Connected to ${deviceInfo.value.productName}`;
    if (deviceInfo.value.serialNumber) {
      status += ` (S/N: ${deviceInfo.value.serialNumber})`;
    }
    // Firmware version can be added if available and desired
    // if (deviceInfo.value.firmwareVersion) {
    //   status += ` - FW: ${deviceInfo.value.firmwareVersion}`;
    // }
    return status;
  }
  return "Connected";
});

// TODO: Add debug log toggle and display
</script>

<style scoped>
/* Add any specific styles for BottomBar here */
</style> 