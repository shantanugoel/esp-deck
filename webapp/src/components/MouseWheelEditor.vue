<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { defineProps, defineEmits } from 'vue'
import type { ConfigActionMouseWheel } from '@/types/protocol';

const props = defineProps<{ action: ConfigActionMouseWheel }>()
const emit = defineEmits<{(e: 'update', value: ConfigActionMouseWheel): void}>()
const isEditing = ref(false)
const tempAmount = ref(props.action.amount)
const inputRef = ref<HTMLInputElement | null>(null)

function startEdit() {
  tempAmount.value = props.action.amount
  isEditing.value = true
}
function saveEdit() {
  emit('update', { type: 'MouseWheel', amount: tempAmount.value })
  isEditing.value = false
}

watch(isEditing, (val) => {
  if (val) nextTick(() => inputRef.value?.focus())
})
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Amount:</span>
    <template v-if="!isEditing">
      <span>{{ props.action.amount }}</span>
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
        v-model.number="tempAmount"
        type="number"
        class="border rounded px-2 py-1 w-14"
        @keyup.enter="saveEdit"
        @blur="saveEdit"
      />
    </template>
  </div>
</template> 