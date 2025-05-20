<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
const isEditing = ref(false)
const tempString = ref(props.action.String || '')
const inputRef = ref<HTMLInputElement | null>(null)

function startEdit() {
  tempString.value = props.action.String || ''
  isEditing.value = true
  nextTick(() => inputRef.value?.focus())
}
function saveEdit() {
  emit('update', { String: tempString.value })
  isEditing.value = false
}
watch(() => props.action.String, (val) => {
  if (!isEditing.value) tempString.value = val || ''
})
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">String:</span>
    <template v-if="!isEditing">
      <span>{{ props.action.String || '<empty>' }}</span>
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
        v-model="tempString"
        class="border rounded px-2 py-1 w-48"
        placeholder="Enter string"
        @keyup.enter="saveEdit"
        @blur="saveEdit"
      />
    </template>
  </div>
</template> 