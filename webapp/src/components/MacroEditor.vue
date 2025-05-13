<template>
  <div v-if="open" class="w-full">
    <div class="mb-4">
      <h3 class="text-lg font-bold mb-2">Macro Sequence Editor</h3>
      <div class="flex flex-wrap gap-2 mb-4">
        <button v-for="action in actionPalette" :key="action.type" @click="addAction(action.type)"
          class="px-3 py-1 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm">
          + {{ action.label }}
        </button>
      </div>
      <div v-if="sequence.length === 0" class="text-muted-foreground text-sm mb-2">No actions yet. Add actions from above.</div>
      <ul class="space-y-2">
        <li v-for="(act, idx) in sequence" :key="idx" class="flex items-center gap-2 bg-card rounded p-2">
          <div class="flex-1">
            <component :is="getActionEditor(act, idx)" :action="act" @update="updateAction(idx, $event)" />
          </div>
          <div class="flex flex-col gap-1">
            <button @click="moveUp(idx)" :disabled="idx === 0" class="text-xs px-2 py-1 rounded bg-muted hover:bg-muted/80">↑</button>
            <button @click="moveDown(idx)" :disabled="idx === sequence.length - 1" class="text-xs px-2 py-1 rounded bg-muted hover:bg-muted/80">↓</button>
            <button @click="removeAction(idx)" class="text-xs px-2 py-1 rounded bg-destructive text-destructive-foreground hover:bg-destructive/80">✕</button>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, defineProps, defineEmits } from 'vue'

// Action type definitions
const actionPalette = [
  { type: 'KeyPress', label: 'Keyboard' },
  { type: 'MousePress', label: 'Mouse Press' },
  { type: 'MouseMove', label: 'Mouse Move' },
  { type: 'MouseWheel', label: 'Mouse Wheel' },
  { type: 'ConsumerPress', label: 'Media Key' },
  { type: 'Delay', label: 'Delay' },
  { type: 'Sequence', label: 'Nested Sequence' },
]

const props = defineProps<{
  modelValue: any[],
  open: boolean
}>()
const emit = defineEmits<{ (e: 'update:modelValue', value: any[]): void }>()

const sequence = ref<any[]>([])

watch(() => props.modelValue, (val) => {
  sequence.value = Array.isArray(val) ? JSON.parse(JSON.stringify(val)) : []
}, { immediate: true })

function addAction(type: string) {
  let action: any = {}
  if (type === 'KeyPress') action = { KeyPress: { key: '', modifier: '' } }
  else if (type === 'MousePress') action = { MousePress: { button: 1 } }
  else if (type === 'MouseMove') action = { MouseMove: { dx: 0, dy: 0 } }
  else if (type === 'MouseWheel') action = { MouseWheel: { amount: 1 } }
  else if (type === 'ConsumerPress') action = { ConsumerPress: { usage_id: 0xE9 } }
  else if (type === 'Delay') action = { Delay: { ms: 100 } }
  else if (type === 'Sequence') action = { Sequence: [] }
  sequence.value.push(action)
  emit('update:modelValue', sequence.value)
}
function removeAction(idx: number) {
  sequence.value.splice(idx, 1)
  emit('update:modelValue', sequence.value)
}
function moveUp(idx: number) {
  if (idx === 0) return
  const temp = sequence.value[idx - 1]
  sequence.value[idx - 1] = sequence.value[idx]
  sequence.value[idx] = temp
  emit('update:modelValue', sequence.value)
}
function moveDown(idx: number) {
  if (idx === sequence.value.length - 1) return
  const temp = sequence.value[idx + 1]
  sequence.value[idx + 1] = sequence.value[idx]
  sequence.value[idx] = temp
  emit('update:modelValue', sequence.value)
}
function updateAction(idx: number, newAction: any) {
  sequence.value[idx] = newAction
  emit('update:modelValue', sequence.value)
}

// Editor components for each action type
function getActionEditor(act: any, idx: number) {
  if (act.KeyPress) return KeyPressEditor
  if (act.MousePress) return MousePressEditor
  if (act.MouseMove) return MouseMoveEditor
  if (act.MouseWheel) return MouseWheelEditor
  if (act.ConsumerPress) return ConsumerPressEditor
  if (act.Delay) return DelayEditor
  if (act.Sequence) return SequenceEditor
  return UnknownEditor
}

// --- Inline editors for each action type ---
const KeyPressEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='flex gap-2 items-center'>
    <span class='font-mono'>Key:</span>
    <input v-model="action.KeyPress.key" class='border rounded px-2 py-1 w-20' placeholder='Key' />
    <span class='font-mono'>Mod:</span>
    <input v-model="action.KeyPress.modifier" class='border rounded px-2 py-1 w-20' placeholder='Modifier' />
    <button @click="$emit('update', action)">✔</button>
  </div>`
}
const MousePressEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='flex gap-2 items-center'>
    <span class='font-mono'>Button:</span>
    <select v-model.number="action.MousePress.button" class='border rounded px-2 py-1'>
      <option :value="1">Left</option>
      <option :value="2">Right</option>
      <option :value="4">Middle</option>
    </select>
    <button @click="$emit('update', action)">✔</button>
  </div>`
}
const MouseMoveEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='flex gap-2 items-center'>
    <span class='font-mono'>dx:</span>
    <input v-model.number="action.MouseMove.dx" type='number' class='border rounded px-2 py-1 w-14' />
    <span class='font-mono'>dy:</span>
    <input v-model.number="action.MouseMove.dy" type='number' class='border rounded px-2 py-1 w-14' />
    <button @click="$emit('update', action)">✔</button>
  </div>`
}
const MouseWheelEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='flex gap-2 items-center'>
    <span class='font-mono'>Amount:</span>
    <input v-model.number="action.MouseWheel.amount" type='number' class='border rounded px-2 py-1 w-14' />
    <button @click="$emit('update', action)">✔</button>
  </div>`
}
const ConsumerPressEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='flex gap-2 items-center'>
    <span class='font-mono'>Usage ID:</span>
    <input v-model.number="action.ConsumerPress.usage_id" type='number' class='border rounded px-2 py-1 w-20' />
    <button @click="$emit('update', action)">✔</button>
  </div>`
}
const DelayEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='flex gap-2 items-center'>
    <span class='font-mono'>Delay (ms):</span>
    <input v-model.number="action.Delay.ms" type='number' class='border rounded px-2 py-1 w-20' />
    <button @click="$emit('update', action)">✔</button>
  </div>`
}
const SequenceEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='flex flex-col gap-1'>
    <span class='font-mono'>Nested Sequence:</span>
    <MacroEditor :model-value="action.Sequence" :open="true" @update:modelValue="$emit('update', { Sequence: $event })" />
  </div>`
}
const UnknownEditor = {
  props: ['action'],
  emits: ['update'],
  template: `<div class='text-destructive'>Unknown action</div>`
}
</script> 