<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { ConfigActionConsumerPress } from '@/types/protocol';

const props = defineProps<{ action: ConfigActionConsumerPress }>()
const emit = defineEmits<{(e: 'update', value: ConfigActionConsumerPress): void}>()
const isEditing = ref(false)
const tempUsageId = ref(props.action.usage_id)
const inputRef = ref<HTMLInputElement | null>(null)

function startEdit() {
  tempUsageId.value = props.action.usage_id
  isEditing.value = true
}
function saveEdit() {
  emit('update', { type: 'ConsumerPress', usage_id: tempUsageId.value })
  isEditing.value = false
}

watch(isEditing, (val) => {
  if (val) nextTick(() => inputRef.value?.focus())
})
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Usage ID:</span>
    <template v-if="!isEditing">
      <span>{{ props.action.usage_id }}</span>
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
        v-model.number="tempUsageId"
        type="number"
        class="border rounded px-2 py-1 w-20"
        @keyup.enter="saveEdit"
        @blur="saveEdit"
      />
    </template>
  </div>
</template> 