<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { defineProps, defineEmits } from 'vue'
import { keyCodes } from '../keycodes'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
const isEditingKey = ref<number | null>(null)
const isEditingMod = ref(false)
const tempKey = ref('')
const tempMod = ref(props.action.KeyPress.modifier || '')
const keyInputRef = ref<HTMLInputElement | null>(null)
const modInputRef = ref<HTMLInputElement | null>(null)

const isValidKeyPress = (
  props.action &&
  typeof props.action === 'object' &&
  props.action.KeyPress &&
  Array.isArray(props.action.KeyPress.keys)
)

function startEditKey(idx: number) {
  tempKey.value = props.action.KeyPress.keys[idx] || ''
  isEditingKey.value = idx
}
function saveEditKey(idx: number) {
  const keys = [...props.action.KeyPress.keys]
  keys[idx] = tempKey.value
  emit('update', { ...props.action, KeyPress: { keys, modifier: props.action.KeyPress.modifier || '' } })
  isEditingKey.value = null
}
function addKey() {
  if (props.action.KeyPress.keys.length < 6) {
    emit('update', { ...props.action, KeyPress: { keys: [...props.action.KeyPress.keys, ''], modifier: props.action.KeyPress.modifier || '' } })
  }
}
function removeKey(idx: number) {
  if (props.action.KeyPress.keys.length > 1) {
    const keys = [...props.action.KeyPress.keys]
    keys.splice(idx, 1)
    emit('update', { ...props.action, KeyPress: { keys, modifier: props.action.KeyPress.modifier || '' } })
    if (isEditingKey.value === idx) isEditingKey.value = null
  }
}
function startEditMod() {
  tempMod.value = props.action.KeyPress.modifier || ''
  isEditingMod.value = true
}
function saveEditMod() {
  emit('update', { ...props.action, KeyPress: { keys: props.action.KeyPress.keys, modifier: tempMod.value } })
  isEditingMod.value = false
}
function toggleModifier(mod: string) {
  let mods = (props.action.KeyPress.modifier || '').split(' ').filter(Boolean)
  if (mods.includes(mod)) {
    mods = mods.filter((m: string) => m !== mod)
  } else {
    mods.push(mod)
  }
  emit('update', { ...props.action, KeyPress: { keys: props.action.KeyPress.keys, modifier: mods.join(' ') } })
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
    <div v-for="(k, idx) in props.action.KeyPress.keys" :key="idx" class="flex gap-2 items-center">
      <span class="font-mono">Key{{ props.action.KeyPress.keys.length > 1 ? ` ${idx + 1}` : '' }}:</span>
      <select
        v-model="props.action.KeyPress.keys[idx]"
        class="border rounded px-2 py-1 w-36"
        @change="emit('update', { ...props.action, KeyPress: { keys: props.action.KeyPress.keys, modifier: props.action.KeyPress.modifier } })"
      >
        <option v-for="kc in keyCodes" :key="kc.code" :value="kc.code">{{ kc.label }}</option>
      </select>
      <span v-if="props.action.KeyPress.keys.length > 1" @click="removeKey(idx)" class="ml-1 cursor-pointer text-destructive hover:text-destructive/80" title="Remove" tabindex="0" role="button" aria-label="Remove">🗑️</span>
    </div>
    <button
      v-if="props.action.KeyPress.keys.length < 6"
      @click="addKey"
      class="mt-1 px-2 py-1 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-xs self-start"
      type="button"
    >
      + Add Key
    </button>
    <div class="flex gap-2 items-center mt-2">
      <span class="font-mono">Modifiers:</span>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.KeyPress.modifier?.includes('ControlLeft')" @change="toggleModifier('ControlLeft')" /> Ctrl
      </label>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.KeyPress.modifier?.includes('ShiftLeft')" @change="toggleModifier('ShiftLeft')" /> Shift
      </label>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.KeyPress.modifier?.includes('AltLeft')" @change="toggleModifier('AltLeft')" /> Alt
      </label>
      <label class="flex items-center gap-1">
        <input type="checkbox" :checked="props.action.KeyPress.modifier?.includes('MetaLeft')" @change="toggleModifier('MetaLeft')" /> Meta
      </label>
    </div>
  </div>
  <div v-else class="text-destructive text-sm p-2 border border-destructive rounded bg-destructive/10">
    Invalid KeyPress action structure.
  </div>
</template> 