<template>
  <div class="space-y-6 p-4 md:p-6">
    <div class="flex justify-between items-center">
      <h2 class="text-2xl font-semibold">Dashboard Widget Settings</h2>
    </div>

    <Card>
      <CardHeader>
        <CardTitle>Manage Widgets</CardTitle>
        <CardDescription>
          Add, edit, or remove widgets that appear on your device's dashboard.
          Changes will be staged and can be saved via the main "Save Settings" button in the header.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div class="mb-4">
          <Button @click="openAddWidgetDialog">
            <PlusCircle class="mr-2 h-4 w-4" /> Add New Widget
          </Button>
        </div>

        <div v-if="widgetSettings.widgetListForDisplay.value.length === 0 && !widgetSettings.hasPendingChanges.value && !initialLoadAttempted" class="flex items-center justify-center py-6">
          <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
          <p class="ml-2">Loading widget settings from device store...</p>
        </div>

        <template v-else-if="widgetSettings.widgetListForDisplay.value.length > 0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead class="w-[50px]">ID</TableHead>
                <TableHead>Title</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>URL</TableHead>
                <TableHead>Update (s)</TableHead>
                <TableHead class="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="widget in widgetSettings.widgetListForDisplay.value" :key="widget.id">
                <TableCell>{{ widget.id }}</TableCell>
                <TableCell class="font-medium">{{ widget.title }}</TableCell>
                <TableCell>{{ 'Text' in widget.kind ? 'Text' : 'Image' }}</TableCell>
                <TableCell class="truncate max-w-xs">
                  {{ 'Text' in widget.kind ? widget.kind.Text[0] : widget.kind.Image }}
                </TableCell>
                <TableCell>{{ widget.update_interval_seconds }}</TableCell>
                <TableCell class="text-right space-x-2">
                  <Button variant="outline" size="sm" @click="openEditWidgetDialog(widget.id)">
                    <Pencil class="mr-1 h-3 w-3" /> Edit
                  </Button>
                  <Button variant="destructive" size="sm" @click="deleteWidgetHandler(widget.id)">
                    <Trash2 class="mr-1 h-3 w-3" /> Delete
                  </Button>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </template>
        <p v-else class="text-muted-foreground text-center py-6">
          No widgets configured yet. Click "Add New Widget" to get started.
        </p>
      </CardContent>
    </Card>

    <!-- Add/Edit Widget Dialog -->
    <Dialog :open="isDialogVisible" @update:open="isDialogVisible = $event">
      <DialogContent class="sm:max-w-[525px]">
        <DialogHeader>
          <DialogTitle>{{ editingWidgetId === null ? 'Add New Widget' : 'Edit Widget' }}</DialogTitle>
          <DialogDescription>
            Configure the details for your widget. Click save when you're done.
          </DialogDescription>
        </DialogHeader>
        <form @submit.prevent="handleFormSubmit" class="space-y-4 py-2">
          <div>
            <Label for="widget-title">Title</Label>
            <Input id="widget-title" v-model="widgetForm.title" placeholder="e.g., Weather Update" required />
          </div>

          <div>
            <Label for="widget-type">Widget Type</Label>
            <RadioGroup id="widget-type" v-model="widgetForm.type" :disabled="editingWidgetId !== null" @update:model-value="onWidgetTypeChange">
              <div class="flex items-center space-x-2">
                <RadioGroupItem id="type-text" value="Text" />
                <Label for="type-text">Text</Label>
              </div>
              <div class="flex items-center space-x-2">
                <RadioGroupItem id="type-image" value="Image" />
                <Label for="type-image">Image</Label>
              </div>
            </RadioGroup>
          </div>

          <div>
            <Label for="widget-url">URL</Label>
            <Input id="widget-url" type="url" v-model="widgetForm.url" placeholder="https://example.com/data_or_image.jpg" required />
             <p class="text-sm text-muted-foreground mt-1" v-if="widgetForm.type === 'Image'">
              Supports JPG, PNG, WebP. Keep images small (e.g., &lt; 200x200px) for best performance.
            </p>
          </div>

          <div v-if="widgetForm.type === 'Text'">
            <div class="flex items-center space-x-2 mb-2">
              <Checkbox 
                id="widget-is-json" 
                :checked="widgetForm.isJson" 
                @update:checked="widgetForm.isJson = $event"
              />
              <Label for="widget-is-json">Response is JSON</Label>
            </div>
            <div v-if="widgetForm.isJson">
              <Label for="widget-json-pointer">JSON Pointer (optional)</Label>
              <Input id="widget-json-pointer" v-model="formJsonPointer" placeholder="e.g., /data/value or /results/0/message" />
              <p class="text-sm text-muted-foreground mt-1">
                Extract specific data. Example: <code>/path/to/array/0/item</code>. See <a href="https://datatracker.ietf.org/doc/html/rfc6901" target="_blank" class="underline">RFC6901</a>.
                Text will be truncated if too long on device.
              </p>
            </div>
          </div>
          
          <div>
            <Label for="widget-update-interval">Update Interval (seconds)</Label>
            <Input id="widget-update-interval" type="number" v-model.number="widgetForm.update_interval_seconds" placeholder="e.g., 60" min="0" required />
            <p class="text-sm text-muted-foreground mt-1">
              How often the widget polls for new data. Avoid very frequent updates.
            </p>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" @click="closeDialog">Cancel</Button>
            <Button type="submit">{{ editingWidgetId === null ? 'Add Widget' : 'Save Changes' }}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

  </div>
</template>

<script setup lang="ts">
import { ref, watch, reactive, computed, onMounted } from 'vue';
import { useWidgetSettings } from '@/composables/useWidgetSettings';
import type { WidgetFormState } from '@/types/deviceConfig';

import { Button } from '@/components/ui/button';
import {
  Card, CardContent, CardDescription, CardHeader, CardTitle,
} from '@/components/ui/card';
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table';
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Checkbox } from '@/components/ui/checkbox';
import { Loader2, PlusCircle, Pencil, Trash2 } from 'lucide-vue-next';

const toast = (options: any) => { 
    console.log('Toast:', options.title, options.description);
    if (options.variant === 'destructive') console.error('Error Toast:', options.title, options.description);
  };

const widgetSettings = useWidgetSettings();
const initialLoadAttempted = ref(false);

onMounted(() => {
  setTimeout(() => {
    initialLoadAttempted.value = true;
  }, 200);
});

const isDialogVisible = ref(false);
const editingWidgetId = ref<number | null>(null);

const initialFormState: WidgetFormState = {
  id: null,
  title: '',
  type: 'Text',
  url: '',
  jsonPointer: null,
  update_interval_seconds: 60,
  isJson: false,
};
const widgetForm = reactive<WidgetFormState>({ ...initialFormState });

const formJsonPointer = computed({
  get: () => widgetForm.jsonPointer === null ? undefined : widgetForm.jsonPointer,
  set: (val) => {
    widgetForm.jsonPointer = val === undefined || val === '' ? null : val;
  }
});

const resetForm = () => {
  Object.assign(widgetForm, initialFormState);
  editingWidgetId.value = null;
};

const onWidgetTypeChange = (newType: string) => {
  if (newType === 'Text' || newType === 'Image') {
    widgetForm.type = newType as 'Text' | 'Image';
    if (widgetForm.type === 'Image') {
      widgetForm.isJson = false;
      widgetForm.jsonPointer = null;
    }
  } else {
    console.warn('Unexpected widget type from RadioGroup:', newType);
    widgetForm.type = 'Text';
  }
};

const openAddWidgetDialog = () => {
  resetForm();
  isDialogVisible.value = true;
};

const openEditWidgetDialog = (widgetId: number) => {
  const widgetToEdit = widgetSettings.displayWidgets.value[widgetId];
  if (widgetToEdit) {
    const formValues = widgetSettings.convertWidgetItemToForm(widgetId, widgetToEdit);
    Object.assign(widgetForm, formValues);
    editingWidgetId.value = widgetId;
    isDialogVisible.value = true;
  } else {
    toast({ title: 'Error', description: `Widget with ID ${widgetId} not found for editing.`, variant: 'destructive' });
  }
};

const closeDialog = () => {
  isDialogVisible.value = false;
  resetForm();
};

const handleFormSubmit = () => {
  if (editingWidgetId.value !== null) {
    const success = widgetSettings.updateWidget(editingWidgetId.value, { ...widgetForm });
    if (success) closeDialog();
  } else {
    const success = widgetSettings.addWidget({ ...widgetForm });
    if (success) closeDialog();
  }
};

const deleteWidgetHandler = (id: number) => {
  widgetSettings.deleteWidget(id);
};

watch(isDialogVisible, (newValue) => {
  if (!newValue) {
  }
});

watch(() => widgetForm.isJson, (isJson) => {
  if (!isJson) {
    widgetForm.jsonPointer = null;
  }
});

</script> 