<script setup lang="ts">
import { defineProps, defineEmits, ref, watch, computed } from 'vue';
import type { PropType } from 'vue';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { ButtonConfig } from '@/stores/macroPadConfigStore';
import type { ConfigAction, ConfigActionSequence } from '@/types/protocol';

const props = defineProps({
  modelValue: { type: Boolean, required: true },
  buttonConfig: { type: Object as PropType<ButtonConfig | null>, default: null },
});

const emit = defineEmits(['update:modelValue', 'save']);

const localButtonName = ref('');
const localActions = ref<ConfigAction[]>([]);

watch(() => props.buttonConfig, (newConfig) => {
  if (newConfig) {
    localButtonName.value = newConfig.name || `Button ${newConfig.id}`;
    localActions.value = JSON.parse(JSON.stringify(newConfig.actions || [])); // Deep copy for potential modification
  } else {
    localButtonName.value = '';
    localActions.value = [];
  }
}, { immediate: true });

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const handleSave = () => {
  if (props.buttonConfig) {
    emit('save', { ...props.buttonConfig, name: localButtonName.value, actions: localActions.value });
    isOpen.value = false;
  }
};

const handleCancel = () => {
  isOpen.value = false;
};

// Updated formatAction function
const formatAction = (action: ConfigAction, indentLevel = 0): string => {
  const indent = '  '.repeat(indentLevel);
  if (!action || !action.type) return `${indent}Invalid Action`;

  switch (action.type) {
    case 'KeyPress':
      return `${indent}Key Press: ${action.keys.join(', ')}${action.modifier ? ' (Mod: ' + action.modifier + ')' : ''}`;
    case 'KeyRelease':
      return `${indent}Key Release`;
    case 'MouseMove':
      return `${indent}Mouse Move: dx=${action.dx}, dy=${action.dy}`;
    case 'MousePress':
      return `${indent}Mouse Press: button=${action.button}`;
    case 'MouseRelease':
      return `${indent}Mouse Release`;
    case 'MouseWheel':
      return `${indent}Mouse Wheel: amount=${action.amount}`;
    case 'ConsumerPress':
      return `${indent}Consumer Press: id=${action.usage_id}`;
    case 'ConsumerRelease':
      return `${indent}Consumer Release`;
    case 'Delay':
      return `${indent}Delay: ${action.ms}ms`;
    case 'SendString':
      return `${indent}Send String: "${action.keys.join('')}"${action.modifiers.length > 0 ? ' (Mods: ' + action.modifiers.join(', ') + ')' : ''}`;
    case 'Sequence':
      // Explicitly cast to ConfigActionSequence to satisfy TypeScript about the 'actions' property
      const sequenceAction = action as ConfigActionSequence;
      const subActions = sequenceAction.actions.map(sub => formatAction(sub, indentLevel + 1)).join('\n');
      return `${indent}Sequence:\n${subActions}`;
    default:
      // Handle cases where action.type might not be recognized by the switch (shouldn't happen with proper types)
      const exhaustiveCheck: never = action;
      return `${indent}Unknown Action: ${JSON.stringify(exhaustiveCheck)}`;
  }
};

</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[600px]"> <!-- Increased width for better action display -->
      <DialogHeader>
        <DialogTitle>Configure Button {{ props.buttonConfig?.id }}</DialogTitle>
        <DialogDescription>
          Modify the name and actions for this macropad button.
        </DialogDescription>
      </DialogHeader>

      <div v-if="props.buttonConfig" class="grid gap-4 py-4">
        <div class="grid grid-cols-4 items-center gap-4">
          <Label for="button-name" class="text-right">
            Name
          </Label>
          <Input id="button-name" v-model="localButtonName" class="col-span-3" />
        </div>

        <div class="grid grid-cols-4 items-start gap-4">
          <Label class="text-right col-span-1 pt-2">Actions</Label>
          <div class="col-span-3">
            <div v-if="localActions.length === 0" class="text-sm text-muted-foreground">
              No actions configured.
            </div>
            <!-- Use pre-wrap for better formatting of multi-line sequences -->
            <ul v-else class="list-none p-0 m-0 space-y-1">
              <li v-for="(action, index) in localActions" :key="index" class="text-sm bg-slate-100 dark:bg-slate-800 p-2 rounded whitespace-pre-wrap">
                {{ formatAction(action) }}
              </li>
            </ul>
            <!-- TODO: Add UI for editing/adding/removing actions -->
            <div class="text-center text-sm text-muted-foreground col-span-4 pt-2">
              (Action editing UI will be here)
            </div>
          </div>
        </div>
      </div>
      <div v-else class="py-4">
        <p>No button selected.</p>
      </div>

      <DialogFooter>
        <Button variant="outline" @click="handleCancel">Cancel</Button>
        <Button type="submit" @click="handleSave" :disabled="!props.buttonConfig">Save Changes</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
