<template>
  <Tabs :model-value="uiStore.activeTabId" @update:model-value="onTabChange" class="w-full">
    <TabsList class="grid w-full grid-cols-5">
      <TabsTrigger 
        v-for="tab in uiStore.availableTabs" 
        :key="tab.id" 
        :value="tab.id"
      >
        {{ tab.label }}
      </TabsTrigger>
    </TabsList>
    <!-- TabContent will be rendered by DefaultLayout.vue using activeViewComponent -->
  </Tabs>
</template>

<script setup lang="ts">
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useUiStore, type TabId } from '@/stores/uiStore';

const uiStore = useUiStore();

// Shadcn-Vue Tabs component emits string | number for the value
const onTabChange = (newTabId: string | number | undefined) => {
  if (newTabId !== undefined) {
    const newTabIdStr = String(newTabId); // Ensure it's a string for comparison and store
    if (uiStore.availableTabs.some(tab => tab.id === newTabIdStr)) {
      uiStore.setActiveTab(newTabIdStr as TabId);
    } else {
      console.warn(`TabNavigation: Attempted to switch to an invalid tab ID: ${newTabIdStr}`);
    }
  } 
};

</script>

<style scoped>
/* Add any specific styling for TabNavigation if needed */
</style> 