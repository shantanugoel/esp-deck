<template>
  <div class="w-full min-h-[calc(100vh-5rem)] flex flex-col items-center justify-center pt-8">
    <div class="w-full max-w-4xl bg-card/80 rounded-2xl shadow-lg p-4 flex flex-col items-center justify-center">
      <ButtonGrid :button-labels="buttonLabels" />
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-4">
      <DeviceStatus />
    </div>
    <div class="w-full max-w-4xl flex justify-center mt-2">
      <div class="text-xs text-muted-foreground font-mono">Time: Synced</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ButtonGrid from '../components/ButtonGrid.vue'
import DeviceStatus from '../components/DeviceStatus.vue'

const buttonLabels = [
  'A 🅰️', 'Ctrl+C 📋', 'Hello 👋', 'F5 🔄',
  'Left Click 🖱️', 'Move Square 🔲', 'Scroll Down 🖱️', 'Right Click 🖱️',
  'Vol Up 🔊', 'Vol Down 🔉', 'Mute 🔇', 'Play/Pause ⏯️',
  'Alt+Tab 🔀', 'Ctrl+Alt+Del 💻', 'Copy+Paste 📋', 'TaskMgr 🧑‍💻'
]
</script> 