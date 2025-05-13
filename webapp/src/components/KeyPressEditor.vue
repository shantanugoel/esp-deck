<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { defineProps, defineEmits } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
const isEditing = ref(false)
const tempKey = ref(props.action.KeyPress.key)
const tempMod = ref(props.action.KeyPress.modifier)
const keyInputRef = ref<HTMLInputElement | null>(null)

function startEdit() {
  tempKey.value = props.action.KeyPress.key
  tempMod.value = props.action.KeyPress.modifier
  isEditing.value = true
}
function saveEdit() {
  emit('update', { ...props.action, KeyPress: { key: tempKey.value, modifier: tempMod.value } })
  isEditing.value = false
}

watch(isEditing, (val) => {
  if (val) nextTick(() => keyInputRef.value?.focus())
})
</script>
<template>
  <div class="flex gap-2 items-center">
    <span class="font-mono">Key:</span>
    <template v-if="!isEditing">
      <span>{{ props.action.KeyPress.key || '<key>' }}</span>
      <span class="font-mono ml-2">Mod:</span>
      <span>{{ props.action.KeyPress.modifier || '-' }}</span>
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
        ref="keyInputRef"
        v-model="tempKey"
        class="border rounded px-2 py-1 w-20"
        placeholder="Key"
        @keyup.enter="saveEdit"
        @blur="saveEdit"
      />
      <span class="font-mono ml-2">Mod:</span>
      <input
        v-model="tempMod"
        class="border rounded px-2 py-1 w-20"
        placeholder="Modifier"
        @keyup.enter="saveEdit"
        @blur="saveEdit"
      />
    </template>
  </div>
</template> 