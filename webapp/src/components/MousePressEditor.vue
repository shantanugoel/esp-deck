<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { ConfigActionMousePress } from '@/types/protocol';

const props = defineProps<{ action: ConfigActionMousePress }>()
const emit = defineEmits<{(e: 'update', value: ConfigActionMousePress): void}>()
const isEditing = ref(false)
const tempButton = ref(props.action.button)
const selectRef = ref<HTMLSelectElement | null>(null)

function startEdit() {
  tempButton.value = props.action.button
  isEditing.value = true
}
function saveEdit() {
  emit('update', { type: 'MousePress', button: tempButton.value })
  isEditing.value = false
}
function buttonLabel(val: number) {
  if (val === 1) return 'Left'
  if (val === 2) return 'Right'
  if (val === 4) return 'Middle'
  return val
}

watch(isEditing, (val) => {
  if (val) nextTick(() => selectRef.value?.focus())
})
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Button:</span>
    <template v-if="!isEditing">
      <span>{{ buttonLabel(props.action.button) }}</span>
      <span
        class="ml-1 cursor-pointer text-muted-foreground hover:text-primary"
        @click="startEdit"
        title="Edit"
        tabindex="0"
        role="button"
        aria-label="Edit"
      >✏️</span>
    </template>
    <template v-else>
      <select
        ref="selectRef"
        v-model.number="tempButton"
        class="border rounded px-2 py-1"
        @blur="saveEdit"
        @change="saveEdit"
      >
        <option :value="1">Left</option>
        <option :value="2">Right</option>
        <option :value="4">Middle</option>
      </select>
    </template>
  </div>
</template> 