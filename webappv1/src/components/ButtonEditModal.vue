<template>
  <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
    <div class="bg-card rounded-xl shadow-xl w-full max-w-md p-6 relative">
      <button class="absolute top-3 right-3 text-muted-foreground hover:text-foreground" @click="emitClose" aria-label="Close">
        <span aria-hidden="true">&times;</span>
      </button>
      <h2 class="text-xl font-bold mb-4 text-center">Edit Button {{ buttonIndex + 1 }}</h2>
      <form @submit.prevent="onSave">
        <div class="mb-4">
          <label class="block text-sm font-medium mb-1" for="buttonName">Name (with emoji/icon)</label>
          <input
            id="buttonName"
            v-model="form.name"
            type="text"
            maxlength="20"
            class="border border-muted rounded px-3 py-2 bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-primary w-full"
            placeholder="Button name or emoji"
            required
          />
        </div>
        <div class="mb-4">
          <label class="block text-sm font-medium mb-1" for="actionType">Action Type</label>
          <select id="actionType" v-model="form.actionType" class="border border-muted rounded px-3 py-2 bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-primary w-full">
            <option value="keyboard">Keyboard</option>
            <option value="mouse">Mouse</option>
            <option value="media">Media</option>
            <option value="macro">Macro</option>
          </select>
        </div>
        <div class="mb-4" v-if="form.actionType === 'macro'">
          <label class="block text-sm font-medium mb-1">Macro Sequence</label>
          <MacroEditor v-model="macroSequence" :open="true" />
        </div>
        <div class="mb-4" v-else>
          <label class="block text-sm font-medium mb-1" for="actionDetail">Action Details</label>
          <textarea
            id="actionDetail"
            v-model="form.actionDetail"
            class="border border-muted rounded px-3 py-2 bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-primary w-full"
            rows="2"
            placeholder="e.g. Ctrl+C, Play/Pause, macro script..."
          ></textarea>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button type="button" class="px-4 py-2 rounded font-semibold transition bg-muted text-muted-foreground hover:bg-muted/80" @click="emitClose">Cancel</button>
          <button type="submit" class="px-4 py-2 rounded font-semibold transition bg-primary text-primary-foreground hover:bg-primary/80">Save</button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, defineProps, defineEmits } from 'vue'
import MacroEditor from './MacroEditor.vue'

type ButtonData = {
  name: string
  actionType: string
  actionDetail: string
}

const props = defineProps<{
  open: boolean
  buttonIndex: number
  buttonData: ButtonData
}>()

const emit = defineEmits<{
  (e: 'save', data: ButtonData): void
  (e: 'close'): void
}>()

const form = ref<ButtonData>({
  name: '',
  actionType: 'keyboard',
  actionDetail: ''
})

// Macro sequence state for macro editor
const macroSequence = ref<any[]>([])

watch(() => props.buttonData, (val) => {
  if (val) {
    form.value = { ...val }
    if (val.actionType === 'macro') {
      try {
        macroSequence.value = JSON.parse(val.actionDetail)
      } catch {
        macroSequence.value = []
      }
    } else {
      macroSequence.value = []
    }
  }
}, { immediate: true })

watch(macroSequence, (val) => {
  if (form.value.actionType === 'macro') {
    form.value.actionDetail = JSON.stringify(val)
  }
}, { deep: true })

function onSave() {
  emit('save', { ...form.value })
}
function emitClose() {
  emit('close')
}
</script>