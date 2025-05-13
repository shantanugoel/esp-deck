<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Amount:</span>
    <input v-model.number="props.action.MouseWheel.amount" type="number" class="border rounded px-2 py-1 w-14" />
    <button @click="emit('update', props.action)">✔</button>
  </div>
</template> 