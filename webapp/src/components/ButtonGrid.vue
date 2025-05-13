<template>
  <div class="grid grid-cols-4 gap-3 w-full">
    <button
      v-for="(label, idx) in buttonLabels"
      :key="idx"
      class="h-12 sm:h-14 md:h-16 w-full rounded-lg bg-card text-card-foreground font-semibold text-base flex items-center justify-center shadow hover:bg-muted transition"
    >
      {{ label }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { defineProps } from 'vue'

const props = defineProps<{ buttonLabels: string[] }>()
</script> 