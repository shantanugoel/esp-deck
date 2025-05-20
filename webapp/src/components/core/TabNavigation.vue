<template>
  <nav class="flex space-x-1 border-b">
    <!-- Using Shadcn-Vue like styling for tabs -->
    <button class="px-4 py-2 -mb-px border-b-2 border-primary font-semibold text-primary focus:outline-none">
      MacroPad
    </button>
    <button class="px-4 py-2 border-b-2 border-transparent hover:border-gray-400 text-muted-foreground focus:outline-none">
      Dashboard
    </button>
    <button class="px-4 py-2 border-b-2 border-transparent hover:border-gray-400 text-muted-foreground focus:outline-none">
      Now
    </button>
    <button class="px-4 py-2 border-b-2 border-transparent hover:border-gray-400 text-muted-foreground focus:outline-none">
      Device Settings
    </button>
  </nav>
</template>

<script setup lang="ts">
// Logic for TabNavigation will go here in later phases
</script>

<style scoped>
/* Add any TabNavigation-specific styles here */
</style> 