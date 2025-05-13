<template>
  <div class="w-full min-h-screen flex flex-col items-center bg-background py-8">
    <div class="w-full bg-card rounded-2xl shadow-lg p-6 flex flex-col items-center">
      <h1 class="text-2xl font-bold mb-4 text-center">Edit Macro for Button {{ buttonIndexDisplay }}</h1>
      <div class="w-full mb-4">
        <label class="block text-sm font-medium mb-1">Button Name</label>
        <input
          v-model="buttonName"
          type="text"
          maxlength="20"
          class="border border-muted rounded px-3 py-2 bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-primary w-full"
          placeholder="Button name or emoji"
        />
      </div>
      <div class="w-full flex-1 mb-4">
        <label class="block text-sm font-medium mb-1">Macro Sequence</label>
        <MacroEditor v-model="macroSequence" :open="true" />
      </div>
      <div class="w-full flex justify-end gap-2 mt-6">
        <button type="button" class="px-4 py-2 rounded font-semibold transition bg-muted text-muted-foreground hover:bg-muted/80" @click="goBack">Cancel</button>
        <button type="button" class="px-4 py-2 rounded font-semibold transition bg-primary text-primary-foreground hover:bg-primary/80" @click="onSave">Save</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useDeviceStore } from '../stores/deviceStore'
import MacroEditor from '../components/MacroEditor.vue'

const route = useRoute()
const router = useRouter()
const deviceStore = useDeviceStore()

const buttonIndex = computed(() => {
  const idx = parseInt(route.params.buttonIndex as string, 10)
  return isNaN(idx) ? null : idx
})
const buttonIndexDisplay = computed(() => (buttonIndex.value != null ? buttonIndex.value + 1 : '?'))

const buttonName = ref('')
const macroSequence = ref<any[]>([])

onMounted(() => {
  if (buttonIndex.value == null) return
  const config = deviceStore.deviceConfig?.config
  const buttonNames = config?.button_names || {}
  const mappings = config?.mappings || {}
  buttonName.value = buttonNames[buttonIndex.value] || `Button ${buttonIndexDisplay.value}`
  const mappingKey = String(buttonIndexDisplay.value)
  macroSequence.value = Array.isArray(mappings[mappingKey]) ? JSON.parse(JSON.stringify(mappings[mappingKey])) : []
})

function onSave() {
  if (buttonIndex.value == null) return
  deviceStore.stageButtonName(buttonIndex.value, buttonName.value)
  deviceStore.stageButtonMacro(buttonIndex.value, macroSequence.value)
  goBack()
}
function goBack() {
  router.push({ name: 'Dashboard' })
}
</script> 