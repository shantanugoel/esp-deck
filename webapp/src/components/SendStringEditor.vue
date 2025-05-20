<script setup lang="ts">
import { defineProps, defineEmits, ref, watch } from 'vue'
import type { ConfigActionSendString } from '@/types/protocol';
import { stringToKeyCodes, keyCodesToString } from '../keycodes';

const props = defineProps<{
  action: ConfigActionSendString
}>()
const emit = defineEmits<{(e: 'update', value: ConfigActionSendString): void }>()

const displayText = ref('');

// Initialize displayText when props.action changes
watch(() => props.action, (newAction) => {
  const currentTextFromAction = keyCodesToString(newAction.keys || [], newAction.modifiers || []);
  // Only update displayText if it's different, to avoid loops if user is typing
  if (displayText.value !== currentTextFromAction) {
    displayText.value = currentTextFromAction;
  }
}, { immediate: true, deep: true });

// Watch displayText (which is bound with v-model) for changes
// and emit the updated action structure
watch(displayText, (newText) => {
  const { keys, modifiers } = stringToKeyCodes(newText);
  // Check if the generated keys/modifiers are different from what's in props.action
  // to prevent emitting an update if props.action was the source of the change.
  const currentKeys = props.action.keys || [];
  const currentModifiers = props.action.modifiers || [];
  if (JSON.stringify(keys) !== JSON.stringify(currentKeys) || JSON.stringify(modifiers) !== JSON.stringify(currentModifiers)) {
    emit('update', { type: 'SendString', keys, modifiers });
  }
});

</script>

<template>
  <div class="flex flex-col gap-2 mt-2">
    <input
      type="text"
      v-model="displayText"
      class="border rounded px-2 py-1 w-full"
      placeholder="Type the string to send..."
      maxlength="64"
    />
    <div v-if="action.keys.length > 0" class="text-xs text-muted-foreground">
      Parsed: {{ action.keys.map((k, i) => `${action.modifiers[i] ? action.modifiers[i].concat('+') : ''}${k}`).join(', ') }}
    </div>
  </div>
</template> 