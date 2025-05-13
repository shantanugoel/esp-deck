<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { defineProps, defineEmits } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
const isEditing = ref(false)
const tempDx = ref(props.action.MouseMove.dx)
const tempDy = ref(props.action.MouseMove.dy)
const dxInputRef = ref<HTMLInputElement | null>(null)

function startEdit() {
  tempDx.value = props.action.MouseMove.dx
  tempDy.value = props.action.MouseMove.dy
  isEditing.value = true
}
function saveEdit() {
  emit('update', { ...props.action, MouseMove: { dx: tempDx.value, dy: tempDy.value } })
  isEditing.value = false
}

watch(isEditing, (val) => {
  if (val) nextTick(() => dxInputRef.value?.focus())
})
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">dx:</span>
    <template v-if="!isEditing">
      <span>{{ props.action.MouseMove.dx }}</span>
      <span class="font-mono ml-2">dy:</span>
      <span>{{ props.action.MouseMove.dy }}</span>
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
        ref="dxInputRef"
        v-model.number="tempDx"
        type="number"
        class="border rounded px-2 py-1 w-14"
        @keyup.enter="saveEdit"
        @blur="saveEdit"
      />
      <span class="font-mono ml-2">dy:</span>
      <input
        v-model.number="tempDy"
        type="number"
        class="border rounded px-2 py-1 w-14"
        @keyup.enter="saveEdit"
        @blur="saveEdit"
      />
    </template>
  </div>
</template> 