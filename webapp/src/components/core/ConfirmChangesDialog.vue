<script setup lang="ts">
import { defineProps, defineEmits, ref, watch, computed } from 'vue';
import type { PropType } from 'vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useMacroPadConfigStore } from '@/stores/macroPadConfigStore';

export type SaveSelection = {
  applyWifiChanges: boolean;
  applyTimezoneChanges: boolean;
  applyApiKeyChanges: boolean;
  // For buttons, a map of buttonKey (string) to boolean (true if save, false if revert/omit)
  buttonsToSave: Record<string, boolean>; 
};

const props = defineProps({
  modelValue: { type: Boolean, required: true }, // For v-model:open
  isWifiChanged: { type: Boolean, default: false },
  isTimezoneChanged: { type: Boolean, default: false },
  isApiKeyChanged: { type: Boolean, default: false },
  changedButtonKeys: { type: Array as PropType<string[]>, default: () => [] },
});

const emit = defineEmits(['update:modelValue', 'confirm-save']);

const macroPadStore = useMacroPadConfigStore();

// Local state for user selections, initialized based on incoming changed props
const applyWifi = ref(props.isWifiChanged);
const applyTimezone = ref(props.isTimezoneChanged);
const applyApiKey = ref(props.isApiKeyChanged);
const buttonsSelection = ref<Record<string, boolean>>({});

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    // Reset selections when dialog opens based on current changes
    applyWifi.value = props.isWifiChanged;
    applyTimezone.value = props.isTimezoneChanged;
    applyApiKey.value = props.isApiKeyChanged;
    
    const newButtonSelection: Record<string, boolean> = {};
    props.changedButtonKeys.forEach(key => {
      newButtonSelection[key] = true; // Default to true (save change) if button is listed as changed
    });
    buttonsSelection.value = newButtonSelection;
  }
}, { immediate: true });

// Watch individual props too, in case they change while dialog is already open (less likely for this modal)
watch(() => props.isWifiChanged, (newVal) => applyWifi.value = newVal);
watch(() => props.isTimezoneChanged, (newVal) => applyTimezone.value = newVal);
watch(() => props.isApiKeyChanged, (newVal) => applyApiKey.value = newVal);
watch(() => props.changedButtonKeys, (newKeys) => {
  const newButtonSelection: Record<string, boolean> = {};
  newKeys.forEach(key => {
    // Preserve existing selection if key still exists, otherwise default to true
    newButtonSelection[key] = buttonsSelection.value[key] === undefined ? true : buttonsSelection.value[key];
  });
  buttonsSelection.value = newButtonSelection;
}, { deep: true });

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const anyDeviceSettingChanged = computed(() => props.isWifiChanged || props.isTimezoneChanged || props.isApiKeyChanged);
const anyButtonChanged = computed(() => props.changedButtonKeys.length > 0);
const noChangesDetected = computed(() => !anyDeviceSettingChanged.value && !anyButtonChanged.value);

const getButtonDisplayName = (key: string): string => {
  const numericId = parseInt(key, 10);
  if (!isNaN(numericId)) {
    return macroPadStore.getButtonName(numericId) || `Button ${numericId + 1}`;
  }
  return `Button ${key}`;
};

const handleConfirmSave = () => {
  const selection: SaveSelection = {
    applyWifiChanges: props.isWifiChanged ? applyWifi.value : false,
    applyTimezoneChanges: props.isTimezoneChanged ? applyTimezone.value : false,
    applyApiKeyChanges: props.isApiKeyChanged ? applyApiKey.value : false,
    buttonsToSave: { ...buttonsSelection.value }, // Send a copy
  };
  // Ensure buttonsToSave only contains keys that were initially marked as changed
  for (const key in selection.buttonsToSave) {
    if (!props.changedButtonKeys.includes(key)) {
      delete selection.buttonsToSave[key];
    }
  }
  emit('confirm-save', selection);
  isOpen.value = false;
};

const handleCancel = () => {
  isOpen.value = false;
};

</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Confirm Changes</DialogTitle>
        <DialogDescription>
          Select the changes you want to save to the device.
        </DialogDescription>
      </DialogHeader>

      <div v-if="noChangesDetected" class="py-4">
        <p class="text-muted-foreground">No changes detected to save.</p>
      </div>
      <div v-else class="py-4 space-y-4 max-h-[60vh] overflow-y-auto">
        <template v-if="anyDeviceSettingChanged">
          <h4 class="font-semibold text-sm mb-2">Device Settings:</h4>
          <div v-if="props.isWifiChanged" class="flex items-center space-x-2">
            <Checkbox id="wifiChanges" v-model:checked="applyWifi" />
            <Label for="wifiChanges" class="font-normal">WiFi Configuration</Label>
          </div>
          <div v-if="props.isTimezoneChanged" class="flex items-center space-x-2">
            <Checkbox id="tzChanges" v-model:checked="applyTimezone" />
            <Label for="tzChanges" class="font-normal">Timezone Offset</Label>
          </div>
          <div v-if="props.isApiKeyChanged" class="flex items-center space-x-2">
            <Checkbox id="apiKeyChanges" v-model:checked="applyApiKey" />
            <Label for="apiKeyChanges" class="font-normal">API Key</Label>
          </div>
        </template>

        <template v-if="anyButtonChanged">
          <h4 class="font-semibold text-sm mt-4 mb-2">MacroPad Buttons:</h4>
          <div v-for="key in props.changedButtonKeys" :key="key" class="flex items-center space-x-2 ml-2">
            <Checkbox :id="`button-${key}`" v-model:checked="buttonsSelection[key]" />
            <Label :for="`button-${key}`" class="font-normal">{{ getButtonDisplayName(key) }}</Label>
          </div>
        </template>
      </div>

      <DialogFooter>
        <Button variant="outline" @click="handleCancel">Cancel</Button>
        <Button 
          type="submit" 
          @click="handleConfirmSave" 
          :disabled="noChangesDetected"
        >
          Save Selected
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template> 