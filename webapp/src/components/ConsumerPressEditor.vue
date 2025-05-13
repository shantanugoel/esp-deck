<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Usage ID:</span>
    <input v-model.number="props.action.ConsumerPress.usage_id" type="number" class="border rounded px-2 py-1 w-20" />
    <button @click="emit('update', props.action)">✔</button>
  </div>
</template> 