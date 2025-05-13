<script setup lang="ts">
import { useRoute } from 'vue-router'
import { RouterLink, RouterView } from 'vue-router'

const route = useRoute()
const isActiveRoute = (path: string) => route.path === path
</script>

<template>
  <div class="w-screen min-h-screen bg-zinc-950 text-zinc-100">
    <!-- Top Navigation Bar -->
    <nav class="flex items-center justify-between px-8 py-4 bg-red-700 shadow-md">
      <div class="flex items-center gap-6">
        <span class="font-bold text-xl tracking-wide text-white">ESP Deck</span>
        <RouterLink
          to="/"
          class="px-4 py-2 rounded transition hover:bg-red-600 text-white"
          :class="{ 'bg-red-600 font-semibold': isActiveRoute('/') }"
        >
          Dashboard
        </RouterLink>
        <RouterLink
          to="/settings"
          class="px-4 py-2 rounded transition hover:bg-red-600 text-white"
          :class="{ 'bg-red-600 font-semibold': isActiveRoute('/settings') }"
        >
          Settings
        </RouterLink>
      </div>
      <div class="text-xs text-red-200">v0.1.x</div>
    </nav>

    <!-- Main Content -->
    <main class="w-full max-w-7xl mx-auto py-10 px-2 sm:px-4 lg:px-8">
      <RouterView />
    </main>
  </div>
</template>
