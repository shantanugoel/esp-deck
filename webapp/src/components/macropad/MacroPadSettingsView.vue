<template>
  <div>
    <h2 class="text-xl font-semibold mb-4">MacroPad Configuration</h2>
    <p class="text-sm text-muted-foreground mb-6">
      Configure the actions for each button on your MacroPad. Click on a button to edit its settings.
    </p>

    <div v-if="!deviceStore.isConnected" class="text-center text-muted-foreground">
      <p>Please connect to a device first.</p>
    </div>
    <div v-else-if="deviceStore.isLoading" class="text-center text-muted-foreground">
      <p>Loading configuration...</p>
    </div>
    <div v-else-if="buttons.length === 0" class="text-center text-muted-foreground">
      <p>No buttons configured or loaded.</p>
    </div>
    <div v-else class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
      <Button
        v-for="button in buttons"
        :key="button.id"
        @click="handleButtonClick(button)"
        variant="outline"
        class="p-4 h-24 flex flex-col items-center justify-center text-center aspect-square focus:ring-2 focus:ring-primary focus:ring-offset-2 transition-all duration-150 ease-in-out hover:shadow-lg hover:scale-105 active:scale-95"
      >
        <span class="text-sm font-medium truncate w-full">{{ button.name || `Button ${button.id}` }}</span>
        <!-- Future: Maybe show a small icon or summary of actions -->
      </Button>
    </div>

    <!-- Button Configuration Dialog -->
    <ButtonConfigDialog
      v-if="selectedButtonConfig" 
      v-model="isDialogVisible"
      :button-config="selectedButtonConfig"
      @save="handleSaveButtonConfig"
    />

  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useMacroPadConfigStore, type ButtonConfig } from '@/stores/macroPadConfigStore';
import { useDeviceStore } from '@/stores/deviceStore';
import { Button } from '@/components/ui/button';
import ButtonConfigDialog from '@/components/macropad/ButtonConfigDialog.vue';

const macroPadStore = useMacroPadConfigStore();
const deviceStore = useDeviceStore();

const buttons = computed(() => macroPadStore.buttons);
const isLoading = computed(() => deviceStore.isLoading);
const isConnected = computed(() => deviceStore.isConnected);

const isDialogVisible = ref(false);
const selectedButtonConfig = ref<ButtonConfig | null>(null);

const handleButtonClick = (button: ButtonConfig) => {
  console.log('Button clicked:', button);
  selectedButtonConfig.value = button;
  isDialogVisible.value = true;
};

const handleSaveButtonConfig = (updatedConfig: ButtonConfig) => {
  console.log('Save button config from dialog:', updatedConfig);
  macroPadStore.updateButtonConfig(updatedConfig.id, updatedConfig);
  // isDialogVisible will be set to false by ButtonConfigDialog's internal v-model:open logic after save
  // selectedButtonConfig will be reset by the watcher below
};

// Watch for the dialog closing to reset the selected button config
watch(isDialogVisible, (newValue) => {
  if (newValue === false) {
    selectedButtonConfig.value = null;
  }
});

</script>

<style scoped>
/* Ensure buttons in the grid are somewhat square if content varies */
.aspect-square {
  aspect-ratio: 1 / 1;
}
</style> 