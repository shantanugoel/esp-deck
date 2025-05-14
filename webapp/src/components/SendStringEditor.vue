<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'

const props = defineProps<{
  action: { SendString: { text: string } }
}>()
const emit = defineEmits<{ (e: 'update', value: any): void }>()

const updateText = (event: Event) => {
  const target = event.target as HTMLInputElement | null
  if (!target) return
  emit('update', { SendString: { text: target.value } })
}
</script>

<template>
  <div class="flex flex-col gap-2 mt-2">
    <input
      type="text"
      :value="action.SendString.text"
      @input="updateText"
      class="border rounded px-2 py-1 w-full"
      placeholder="Type the string to send..."
      maxlength="64"
    />
  </div>
</template> 