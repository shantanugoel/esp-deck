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
      <VueDraggable v-model="sequence" class="space-y-2">
        <div v-for="(act, idx) in sequence" :key="idx">
          <li class="flex items-center gap-2 bg-card rounded border border-muted px-2 py-1 shadow-sm hover:shadow transition-all">
            <span class="cursor-grab flex items-center pr-2 select-none text-muted-foreground">
              <svg xmlns='http://www.w3.org/2000/svg' class='w-4 h-4' fill='none' viewBox='0 0 24 24' stroke='currentColor'><path stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M9 6h.01M9 12h.01M9 18h.01M15 6h.01M15 12h.01M15 18h.01'/></svg>
            </span>
            <div class="flex-1 min-w-0">
              <div class="font-mono text-xs text-primary flex items-center gap-2">
                <span v-html="getActionSummary(act)"></span>
              </div>
              <component :is="getActionEditor(act, idx)" :action="act" @update="updateAction(idx, $event)" />
            </div>
            <button @click="removeAction(idx)" class="ml-2 p-1 rounded text-destructive hover:text-destructive/80" title="Delete">
              <svg xmlns='http://www.w3.org/2000/svg' class='w-4 h-4' fill='none' viewBox='0 0 24 24' stroke='currentColor'><path stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M3 6h18M8 6v12a2 2 0 002 2h4a2 2 0 002-2V6m-6 0V4a2 2 0 012-2h0a2 2 0 012 2v2'/></svg>
            </button>
          </li>
        </div>
      </VueDraggable>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, defineProps, defineEmits } from 'vue'
import { VueDraggable } from 'vue-draggable-plus';
import KeyPressEditor from './KeyPressEditor.vue'
import MousePressEditor from './MousePressEditor.vue'
import MouseMoveEditor from './MouseMoveEditor.vue'
import MouseWheelEditor from './MouseWheelEditor.vue'
import ConsumerPressEditor from './ConsumerPressEditor.vue'
import DelayEditor from './DelayEditor.vue'
import SequenceEditor from './SequenceEditor.vue'
import UnknownEditor from './UnknownEditor.vue'

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

// Move getActionSummary above the template so it is available for template usage
function getActionSummary(act: any): string {
  if (act.KeyPress) {
    const key = act.KeyPress.key || '<key>'
    const mod = act.KeyPress.modifier ? ` + ${act.KeyPress.modifier}` : ''
    return `<b>KeyPress:</b> ${key}${mod}`
  }
  if (act.MousePress) {
    const btn = act.MousePress.button === 1 ? 'Left' : act.MousePress.button === 2 ? 'Right' : 'Middle'
    return `<b>MousePress:</b> ${btn}`
  }
  if (act.MouseMove) {
    return `<b>MouseMove:</b> dx=${act.MouseMove.dx}, dy=${act.MouseMove.dy}`
  }
  if (act.MouseWheel) {
    return `<b>MouseWheel:</b> amount=${act.MouseWheel.amount}`
  }
  if (act.ConsumerPress) {
    return `<b>Media Key:</b> usage_id=0x${act.ConsumerPress.usage_id.toString(16).toUpperCase()}`
  }
  if (act.Delay) {
    return `<b>Delay:</b> ${act.Delay.ms}ms`
  }
  if (act.Sequence) {
    return `<b>Nested Sequence</b>`
  }
  if (typeof act === 'string') {
    return `<b>${act}</b>`
  }
  return '<b>Unknown</b>'
}
</script> 