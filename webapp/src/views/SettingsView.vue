<template>
  <div class="w-full min-h-[calc(100vh-5rem)] flex flex-col items-center justify-center pt-8">
    <div class="w-full max-w-xl bg-card/80 rounded-2xl shadow-lg p-6 flex flex-col items-center justify-center relative">
      <h1 class="text-2xl font-bold mb-4 text-center text-foreground">Settings</h1>
      <p class="text-muted-foreground text-center mb-2">WiFi and device settings will appear here.</p>
      <!-- Future: Add settings forms here -->
      <div v-if="deviceStore.loading" class="absolute inset-0 flex items-center justify-center bg-background/80 rounded-2xl z-10">
        <span class="text-muted-foreground">Loading...</span>
      </div>
      <FeedbackToast
        v-if="deviceStore.error"
        :message="deviceStore.error"
        type="error"
        :show="!!deviceStore.error"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDeviceStore } from '../stores/deviceStore'
import FeedbackToast from '../components/FeedbackToast.vue'

const deviceStore = useDeviceStore()
</script> 