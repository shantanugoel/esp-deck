<script setup lang="ts">
import { useUiStore } from '@/stores/uiStore';
import { deviceService } from '@/services/deviceService'; // Accessing directly for USB logs
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { XIcon } from 'lucide-vue-next';

const uiStore = useUiStore();

const formatTimestamp = (date: Date) => {
  return date.toLocaleTimeString() + '.' + String(date.getMilliseconds()).padStart(3, '0');
};

// The data can be an object or string. We'll stringify objects.
const formatData = (data: any) => {
  if (typeof data === 'string') return data;
  try {
    return JSON.stringify(data, null, 2);
  } catch (e) {
    return '[Unserializable Data]';
  }
};

</script>

<template>
  <div 
    v-if="uiStore.isDebugLogVisible"
    class="fixed bottom-0 left-0 right-0 h-1/3 bg-background/95 backdrop-blur-sm border-t border-border shadow-lg z-50 flex flex-col p-2 max-h-[50vh]"
  >
    <div class="flex justify-between items-center mb-2 px-2">
      <h3 class="text-lg font-semibold">USB Debug Log</h3>
      <Button variant="ghost" size="icon" @click="uiStore.toggleDebugLogVisibility()">
        <XIcon class="h-5 w-5" />
      </Button>
    </div>
    <ScrollArea class="flex-grow h-full border rounded-md bg-muted/40 p-1">
      <div class="p-2 space-y-1 text-xs font-mono">
        <div v-if="deviceService.debugLogs.length === 0" class="text-muted-foreground text-center py-4">
          No USB messages logged yet.
        </div>
        <div 
          v-for="(log, index) in deviceService.debugLogs" 
          :key="index" 
          class="whitespace-pre-wrap break-all p-1 rounded-sm"
          :class="{
            'text-blue-500': log.type === 'sent',
            'text-green-500': log.type === 'received'
          }"
        >
          <span class="font-semibold">[{{ formatTimestamp(log.timestamp) }}] [{{ log.type.toUpperCase() }}]</span>
          <pre class="ml-2 inline-block align-top">{{ formatData(log.data) }}</pre>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>

<style scoped>
/* Ensure pre preserves formatting but also wraps if needed */
pre {
  white-space: pre-wrap; /* Allows wrapping long lines */
  word-break: break-all;   /* Breaks long words/strings if they would overflow */
  display: inline; /* to keep it from taking full block width if short */
}
</style> 