<template>
  <transition name="fade">
    <div v-if="show" :class="toastClass" class="fixed bottom-6 left-1/2 -translate-x-1/2 px-6 py-3 rounded-lg shadow-lg z-50 min-w-[200px] text-center">
      {{ message }}
    </div>
  </transition>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  message: string
  type: 'success' | 'error' | 'info'
  show: boolean
}>()

const toastClass = computed(() => {
  switch (props.type) {
    case 'success':
      return 'bg-green-600 text-white'
    case 'error':
      return 'bg-destructive text-destructive-foreground'
    default:
      return 'bg-muted text-muted-foreground'
  }
})
</script>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style> 