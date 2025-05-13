<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import MacroEditor from './MacroEditor.vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
</script>
<template>
  <div class="flex flex-col gap-1">
    <span class="font-mono">Nested Sequence:</span>
    <MacroEditor :model-value="props.action.Sequence" :open="true" @update:modelValue="val => emit('update', { Sequence: val })" />
  </div>
</template> 