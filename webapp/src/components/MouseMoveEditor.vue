<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { defineProps, defineEmits } from 'vue'
import type { ConfigActionMouseMove } from '@/types/protocol';

const props = defineProps<{ action: ConfigActionMouseMove }>()
const emit = defineEmits<{(e: 'update', value: ConfigActionMouseMove): void}>()
const isEditingDx = ref(false)
const isEditingDy = ref(false)
const tempDx = ref(props.action.dx)
const tempDy = ref(props.action.dy)
const dxInputRef = ref<HTMLInputElement | null>(null)
const dyInputRef = ref<HTMLInputElement | null>(null)

function startEditDx() {
  tempDx.value = props.action.dx
  isEditingDx.value = true
  nextTick(() => dxInputRef.value?.focus())
}
function saveEditDx() {
  emit('update', { type: 'MouseMove', dx: tempDx.value, dy: props.action.dy })
  isEditingDx.value = false
}
function startEditDy() {
  tempDy.value = props.action.dy
  isEditingDy.value = true
  nextTick(() => dyInputRef.value?.focus())
}
function saveEditDy() {
  emit('update', { type: 'MouseMove', dx: props.action.dx, dy: tempDy.value })
  isEditingDy.value = false
}
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">dx:</span>
    <template v-if="!isEditingDx">
      <span>{{ props.action.dx }}</span>
      <span
        class="ml-1 cursor-pointer text-muted-foreground hover:text-primary"
        @click="startEditDx"
        title="Edit dx"
        tabindex="0"
        role="button"
        aria-label="Edit dx"
      >✏️</span>
    </template>
    <template v-else>
      <input
        ref="dxInputRef"
        v-model.number="tempDx"
        type="number"
        class="border rounded px-2 py-1 w-14"
        @keyup.enter="saveEditDx"
        @blur="saveEditDx"
      />
    </template>
    <span class="font-mono ml-2">dy:</span>
    <template v-if="!isEditingDy">
      <span>{{ props.action.dy }}</span>
      <span
        class="ml-1 cursor-pointer text-muted-foreground hover:text-primary"
        @click="startEditDy"
        title="Edit dy"
        tabindex="0"
        role="button"
        aria-label="Edit dy"
      >✏️</span>
    </template>
    <template v-else>
      <input
        ref="dyInputRef"
        v-model.number="tempDy"
        type="number"
        class="border rounded px-2 py-1 w-14"
        @keyup.enter="saveEditDy"
        @blur="saveEditDy"
      />
    </template>
  </div>
</template> 