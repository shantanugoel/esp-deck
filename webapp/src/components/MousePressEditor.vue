<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Button:</span>
    <select v-model.number="props.action.MousePress.button" class="border rounded px-2 py-1">
      <option :value="1">Left</option>
      <option :value="2">Right</option>
      <option :value="4">Middle</option>
    </select>
    <button @click="emit('update', props.action)">✔</button>
  </div>
</template> 