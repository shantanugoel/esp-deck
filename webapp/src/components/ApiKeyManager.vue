<template>
  <div class="space-y-2">
    <div class="flex items-center justify-between">
      <span class="text-sm font-medium">API Key:</span>
      <span v-if="props.modelValue" class="text-sm font-mono bg-muted px-2 py-1 rounded">**********{{ props.modelValue.slice(-4) }} (Click to copy)</span>
      <span v-else class="text-sm text-muted-foreground">Not Set</span>
    </div>
    <div class="flex items-center space-x-2">
      <Button @click="generateApiKey" size="sm">
        Generate New
      </Button>
      <Button @click="clearApiKey" size="sm" :disabled="!props.modelValue">
        Clear
      </Button>
      <Button @click="copyApiKey" size="sm" :disabled="!props.modelValue">
        Copy
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, computed } from 'vue'
import { Button } from '@/components/ui/button'
import { toast } from 'vue-sonner'

const props = defineProps<{
  modelValue: string | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | null): void
}>()

const generateApiKey = () => {
  const newKey = crypto.randomUUID()
  emit('update:modelValue', newKey)
  toast.success('New API key generated. Save settings to apply.')
}

const clearApiKey = () => {
  emit('update:modelValue', null)
  toast.info('API key cleared. Save settings to apply.')
}

const copyApiKey = async () => {
  if (!props.modelValue) {
    toast.warning('No API Key to copy.')
    return
  }
  try {
    await navigator.clipboard.writeText(props.modelValue)
    toast.success('API Key copied to clipboard.')
  } catch (err) {
    toast.error('Failed to copy API Key.')
    console.error('Failed to copy API key:', err)
  }
}
</script> 