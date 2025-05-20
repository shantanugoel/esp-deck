<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import MacroEditor from './MacroEditor.vue'
import type { ConfigActionSequence, ConfigAction } from '@/types/protocol';

const props = defineProps<{ action: ConfigActionSequence }>()
const emit = defineEmits<{(e: 'update', value: ConfigActionSequence): void}>()
</script>
<template>
  <div class="flex flex-col gap-1">
    <span class="font-mono">Nested Sequence:</span>
    <MacroEditor :model-value="props.action.actions" :open="true" @update:modelValue="(val: ConfigAction[]) => emit('update', { type: 'Sequence', actions: val })" />
  </div>
</template> 