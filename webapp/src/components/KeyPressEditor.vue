<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Key:</span>
    <input v-model="props.action.KeyPress.key" class="border rounded px-2 py-1 w-20" placeholder="Key" />
    <span class="font-mono">Mod:</span>
    <input v-model="props.action.KeyPress.modifier" class="border rounded px-2 py-1 w-20" placeholder="Modifier" />
    <button @click="emit('update', props.action)">✔</button>
  </div>
</template> 