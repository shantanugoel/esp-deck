<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { ConfigActionDelay } from '@/types/protocol';

const props = defineProps<{ action: ConfigActionDelay }>()
const emit = defineEmits<{(e: 'update', value: ConfigActionDelay): void}>()
const isEditing = ref(false)
const tempValue = ref(props.action.ms)
const inputRef = ref<HTMLInputElement | null>(null)

function startEdit() {
  tempValue.value = props.action.ms
  isEditing.value = true
}
function saveEdit() {
  emit('update', { type: 'Delay', ms: tempValue.value })
  isEditing.value = false
}

watch(isEditing, (val) => {
  if (val) nextTick(() => inputRef.value?.focus())
})
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Delay (ms):</span>
    <template v-if="!isEditing">
      <span>{{ props.action.ms }}</span>
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
      <input
        ref="inputRef"
        v-model.number="tempValue"
        type="number"
        class="border rounded px-2 py-1 w-20"
        @blur="saveEdit"
        @keyup.enter="saveEdit"
      />
    </template>
  </div>
</template> 