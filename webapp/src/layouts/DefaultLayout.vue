<template>
  <div class="flex flex-col min-h-screen">
    <TopBar />
    <!-- Action Buttons Row - Centered below TopBar -->
    <div class="w-full flex flex-wrap gap-2 justify-center py-2 bg-background shadow-sm">
      <button class="px-3 py-1 bg-secondary text-secondary-foreground rounded hover:bg-secondary/80">Connect</button>
      <button class="px-3 py-1 bg-secondary text-secondary-foreground rounded hover:bg-secondary/80">Save Settings</button>
      <!-- Other buttons will be added here in Phase 2 -->
    </div>

    <main class="flex-grow container mx-auto px-4 py-8">
      <TabNavigation />
      <div class="mt-4 p-4 border rounded-lg shadow bg-card">
        <!-- Content for the active tab will go here -->
        <p>Selected Tab Content Area</p>
        <slot></slot> <!-- Or router-view if tabs control routes -->
      </div>
    </main>
    <BottomBar />
  </div>
</template>

<script setup lang="ts">
import TopBar from '@/components/core/TopBar.vue'
import BottomBar from '@/components/core/BottomBar.vue'
import TabNavigation from '@/components/core/TabNavigation.vue'
</script>

<style scoped>
/* Add any layout-specific styles here */
</style> 