<script setup lang="ts">
import { ref, watch, computed } from 'vue';
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
  applyWifi: boolean;
  applyTimezone: boolean;
  applyApiKey: boolean;
  applyMappings: boolean; // Master switch for all mappings
  // buttonsToSave will only be populated if applyMappings is true
  // and will contain { [buttonKey]: true } for selected changed buttons
  buttonsToSave: Record<string, boolean>; 
  applyWidgets: boolean; // Added for widgets
};

const props = defineProps({
  modelValue: { type: Boolean, required: true }, 
  isWifiChanged: { type: Boolean, default: false },
  isTimezoneChanged: { type: Boolean, default: false },
  isApiKeyChanged: { type: Boolean, default: false },
  isWidgetsChanged: { type: Boolean, default: false }, // Added for widgets
  changedButtonKeys: { type: Array as PropType<string[]>, default: () => [] },
});

const emit = defineEmits(['update:modelValue', 'confirm-save']);

const macroPadStore = useMacroPadConfigStore();

// Local state for user selections
const selectWifi = ref(false);
const selectTimezone = ref(false);
const selectApiKey = ref(false);
const selectWidgets = ref(false); // Added for widgets
const selectMappingsMaster = ref(false); // Master checkbox for all mappings
const individualButtonSelections = ref<Record<string, boolean>>({});

// Function to initialize/reset local state when dialog opens or props change
const initializeSelections = () => {
  selectWifi.value = props.isWifiChanged;
  selectTimezone.value = props.isTimezoneChanged;
  selectApiKey.value = props.isApiKeyChanged;
  selectWidgets.value = props.isWidgetsChanged; // Added for widgets
  
  // If any button has changed, default the master mappings switch to true
  selectMappingsMaster.value = props.changedButtonKeys.length > 0;

  const newButtonSelection: Record<string, boolean> = {};
  // Only create selections for buttons that actually changed
  props.changedButtonKeys.forEach(key => {
    newButtonSelection[key] = true; // Default changed buttons to selected
  });
  individualButtonSelections.value = newButtonSelection;
};

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    initializeSelections();
  }
}, { immediate: true }); // Immediate to run on component mount if dialog is initially open

// Also re-initialize if the underlying change props change while dialog might be open
watch(() => [
  props.isWifiChanged,
  props.isTimezoneChanged,
  props.isApiKeyChanged,
  props.isWidgetsChanged, // Added for widgets
  props.changedButtonKeys
], () => {
  if (props.modelValue) { // Only if dialog is open
    initializeSelections();
  }
}, { deep: true });


const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const getButtonDisplayName = (key: string): string => {
  const numericId = parseInt(key, 10);
  if (!isNaN(numericId)) {
    return macroPadStore.getButtonName(numericId) || `Button ${numericId + 1}`;
  }
  return `Button ${key}`;
};

const canSave = computed(() => {
  if (selectWifi.value || selectTimezone.value || selectApiKey.value || selectWidgets.value) return true; // Added widgets
  if (selectMappingsMaster.value) {
    // If master mappings is selected, check if any individual button is selected
    return props.changedButtonKeys.some(key => individualButtonSelections.value[key]);
  }
  return false;
});

const handleConfirmSave = () => {
  const buttonsToSaveOutput: Record<string, boolean> = {};
  if (selectMappingsMaster.value) {
    props.changedButtonKeys.forEach(key => {
      if (individualButtonSelections.value[key]) {
        buttonsToSaveOutput[key] = true;
      }
    });
  }

  const selection: SaveSelection = {
    applyWifi: selectWifi.value,
    applyTimezone: selectTimezone.value,
    applyApiKey: selectApiKey.value,
    applyWidgets: selectWidgets.value, // Added widgets
    applyMappings: selectMappingsMaster.value && Object.keys(buttonsToSaveOutput).length > 0,
    buttonsToSave: buttonsToSaveOutput,
  };
  emit('confirm-save', selection);
  isOpen.value = false;
};

const handleCancel = () => {
  isOpen.value = false;
};

</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-lg"> <!-- Slightly wider for more content -->
      <DialogHeader>
        <DialogTitle>Confirm Changes to Save</DialogTitle>
        <DialogDescription>
          Select which settings and MacroPad button configurations you want to apply to the device.
        </DialogDescription>
      </DialogHeader>

      <div class="py-4 space-y-3 max-h-[60vh] overflow-y-auto text-sm">
        <h4 class="font-semibold mb-1">Device Settings:</h4>
        <div class="flex items-center space-x-3 pl-2">
          <Checkbox id="selectWifi" v-model:checked="selectWifi" />
          <Label for="selectWifi" :class="{'text-muted-foreground': !props.isWifiChanged}">
            WiFi Configuration <span v-if="props.isWifiChanged" class="text-xs text-blue-500">(changed)</span>
          </Label>
        </div>
        <div class="flex items-center space-x-3 pl-2">
          <Checkbox id="selectTimezone" v-model:checked="selectTimezone" />
          <Label for="selectTimezone" :class="{'text-muted-foreground': !props.isTimezoneChanged}">
            Timezone Offset <span v-if="props.isTimezoneChanged" class="text-xs text-blue-500">(changed)</span>
          </Label>
        </div>
        <div class="flex items-center space-x-3 pl-2">
          <Checkbox id="selectApiKey" v-model:checked="selectApiKey" />
          <Label for="selectApiKey" :class="{'text-muted-foreground': !props.isApiKeyChanged}">
            API Key <span v-if="props.isApiKeyChanged" class="text-xs text-blue-500">(changed)</span>
          </Label>
        </div>
        <div class="flex items-center space-x-3 pl-2">
          <Checkbox id="selectWidgets" v-model:checked="selectWidgets" />
          <Label for="selectWidgets" :class="{'text-muted-foreground': !props.isWidgetsChanged}">
            Widgets Configuration <span v-if="props.isWidgetsChanged" class="text-xs text-blue-500">(changed)</span>
          </Label>
        </div>

        <h4 class="font-semibold pt-3 mb-1">MacroPad Button Configuration:</h4>
        <div class="flex items-center space-x-3 pl-2">
          <Checkbox id="selectMappingsMaster" v-model:checked="selectMappingsMaster" />
          <Label for="selectMappingsMaster" :class="{'text-muted-foreground': props.changedButtonKeys.length === 0}">
            Apply changes to selected buttons <span v-if="props.changedButtonKeys.length > 0" class="text-xs text-blue-500">({{ props.changedButtonKeys.length }} changed)</span>
          </Label>
        </div>
        
        <div v-if="selectMappingsMaster && props.changedButtonKeys.length > 0" class="space-y-2 pl-6 border-l-2 ml-3 border-muted">
            <div v-for="key in props.changedButtonKeys" :key="key" class="flex items-center space-x-3 pt-1">
              <Checkbox :id="`button-${key}`" v-model:checked="individualButtonSelections[key]" />
              <Label :for="`button-${key}`" class="font-normal">{{ getButtonDisplayName(key) }}</Label>
            </div>
        </div>
        <div v-else-if="props.changedButtonKeys.length === 0" class="pl-6 text-xs text-muted-foreground">
            No button changes detected.
        </div>

      </div>

      <DialogFooter>
        <Button variant="outline" @click="handleCancel">Cancel</Button>
        <Button 
          type="submit" 
          @click="handleConfirmSave" 
          :disabled="!canSave"
        >
          Save to Device
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template> 