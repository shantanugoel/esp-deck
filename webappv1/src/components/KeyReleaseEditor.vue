<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
const props = defineProps<{ action: any }>()
const emit = defineEmits(['update'])
// No editable fields for KeyRelease
</script>
<template>
  <div class="text-muted-foreground text-sm py-2">
    <span class="font-mono">KeyRelease</span> (no editable fields)
  </div>
</template> 