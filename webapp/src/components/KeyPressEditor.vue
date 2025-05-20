<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { keyCodes } from '../keycodes'
import type { ConfigActionKeyPress } from '@/types/protocol';

const props = defineProps<{ action: ConfigActionKeyPress }>()
const emit = defineEmits<{(e: 'update', value: ConfigActionKeyPress): void}>()
const isEditingKey = ref<number | null>(null)
const isEditingMod = ref(false)
const tempKey = ref('')
const tempMod = ref(props.action.modifier || '')
const keyInputRef = ref<HTMLInputElement | null>(null)
const modInputRef = ref<HTMLInputElement | null>(null)

const isValidKeyPress = (
  props.action &&
  props.action.type === 'KeyPress' &&
  Array.isArray(props.action.keys)
)

function startEditKey(idx: number) {
  tempKey.value = props.action.keys[idx] || ''
  isEditingKey.value = idx
}
function saveEditKey(idx: number) {
  const keys = [...props.action.keys]
  keys[idx] = tempKey.value
  emit('update', { type: 'KeyPress', keys, modifier: props.action.modifier || null })
  isEditingKey.value = null
}
function addKey() {
  if (props.action.keys.length < 6) {
    emit('update', { type: 'KeyPress', keys: [...props.action.keys, ''], modifier: props.action.modifier || null })
  }
}
function removeKey(idx: number) {
  if (props.action.keys.length > 1) {
    const keys = [...props.action.keys]
    keys.splice(idx, 1)
    emit('update', { type: 'KeyPress', keys, modifier: props.action.modifier || null })
    if (isEditingKey.value === idx) isEditingKey.value = null
  }
}
function startEditMod() {
  tempMod.value = props.action.modifier || ''
  isEditingMod.value = true
}
function saveEditMod() {
  emit('update', { type: 'KeyPress', keys: props.action.keys, modifier: tempMod.value || null })
  isEditingMod.value = false
}
function toggleModifier(mod: string) {
  let currentModifier = props.action.modifier || '';
  let mods = currentModifier.split(' ').filter(Boolean);

  if (mods.includes(mod)) {
    mods = mods.filter((m: string) => m !== mod);
  } else {
    mods.push(mod);
  }
  const newModifier = mods.length > 0 ? mods.join(' ') : null;
  emit('update', { type: 'KeyPress', keys: props.action.keys, modifier: newModifier });
}

watch(isEditingKey, (val) => {
  if (val !== null) nextTick(() => keyInputRef.value?.focus())
})
watch(isEditingMod, (val) => {
  if (val) nextTick(() => modInputRef.value?.focus())
})
</script>
<template>
  <div v-if="isValidKeyPress" class="flex flex-col gap-2">
    <div v-for="(k, idx) in props.action.keys" :key="idx" class="flex gap-2 items-center">
      <span class="font-mono">Key{{ props.action.keys.length > 1 ? ` ${idx + 1}` : '' }}:</span>
      <select
        v-model="props.action.keys[idx]"
        class="border rounded px-2 py-1 w-36"
        @change="emit('update', { type: 'KeyPress', keys: props.action.keys, modifier: props.action.modifier })"
      >
        <option v-for="kc in keyCodes" :key="kc.code" :value="kc.code">{{ kc.label }}</option>
      </select>
      <span v-if="props.action.keys.length > 1" @click="removeKey(idx)" class="ml-1 cursor-pointer text-destructive hover:text-destructive/80" title="Remove" tabindex="0" role="button" aria-label="Remove">🗑️</span>
    </div>
    <button
      v-if="props.action.keys.length < 6"
      @click="addKey"
      class="mt-1 px-2 py-1 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-xs self-start"
      type="button"
    >
      + Add Key
    </button>
    <div class="flex gap-2 items-center mt-2">
      <span class="font-mono">Modifiers:</span>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.modifier?.includes('ControlLeft')" @change="toggleModifier('ControlLeft')" /> Ctrl
      </label>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.modifier?.includes('ShiftLeft')" @change="toggleModifier('ShiftLeft')" /> Shift
      </label>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.modifier?.includes('AltLeft')" @change="toggleModifier('AltLeft')" /> Alt
      </label>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.modifier?.includes('MetaLeft')" @change="toggleModifier('MetaLeft')" /> Meta
      </label>
    </div>
  </div>
  <div v-else class="text-destructive text-sm p-2 border border-destructive rounded bg-destructive/10">
    Invalid KeyPress action structure.
  </div>
</template> 