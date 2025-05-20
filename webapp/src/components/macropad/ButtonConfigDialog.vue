<script setup lang="ts">
import { ref, watch, computed } from 'vue';
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
import type { ButtonUIData } from './MacroPadSettingsView.vue';
import type { ConfigAction } from '@/types/protocol';
import MacroEditor from '@/components/MacroEditor.vue';

const props = defineProps({
  modelValue: { type: Boolean, required: true },
  buttonConfig: { type: Object as PropType<ButtonUIData | null>, default: null },
});

const emit = defineEmits(['update:modelValue', 'save']);

const localButtonName = ref('');
const localActions = ref<ConfigAction[]>([]);

watch(() => props.buttonConfig, (newConfig) => {
  if (newConfig) {
    localButtonName.value = newConfig.name;
    localActions.value = JSON.parse(JSON.stringify(newConfig.actions || []));
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
    const savedData: ButtonUIData = {
      id: props.buttonConfig.id,
      key: props.buttonConfig.key,
      name: localButtonName.value,
      actions: localActions.value,
    };
    emit('save', savedData);
    isOpen.value = false;
  }
};

const handleCancel = () => {
  isOpen.value = false;
};

</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-3xl">
      <DialogHeader>
        <DialogTitle>Configure Button {{ props.buttonConfig?.name || `ID: ${props.buttonConfig?.id}` }}</DialogTitle>
        <DialogDescription>
          Modify the name and actions for this macropad button.
        </DialogDescription>
      </DialogHeader>

      <div v-if="props.buttonConfig" class="grid gap-y-4 py-4">
        <div class="grid grid-cols-6 items-center gap-x-4">
          <Label for="button-name" class="text-right col-span-1">Name</Label>
          <Input id="button-name" v-model="localButtonName" class="col-span-5" />
        </div>

        <div class="grid grid-cols-6 items-start gap-x-4">
          <Label class="text-right col-span-1 pt-2">Actions</Label>
          <div class="col-span-5">
            <MacroEditor v-model="localActions" :open="true" />
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
