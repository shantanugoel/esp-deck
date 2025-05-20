<template>
  <RouterView />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
// Removed useRoute and isActiveRoute as the nav is removed.
</script>

<style>
/* Ensure html, body, and #app take full height if not already set globally */
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
  background-color: hsl(var(--background)); /* Example from shadcn-vue */
  color: hsl(var(--foreground)); /* Example from shadcn-vue */
}
</style>
