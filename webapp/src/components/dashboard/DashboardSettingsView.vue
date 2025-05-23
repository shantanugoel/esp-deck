<template>
  <div class="space-y-6 p-4 md:p-6">
    <div class="flex justify-between items-center">
      <h2 class="text-2xl font-semibold">Dashboard Widget Settings</h2>
      <div class="flex space-x-2">
        <Button 
          variant="outline" 
          @click="discardWidgetChangesHandler"
          :disabled="!hasPendingChanges || isSaving"
        >
          Discard Changes
        </Button>
        <Button @click="saveWidgetChangesHandler" :disabled="!hasPendingChanges || isSaving">
          <Loader2 class="mr-2 h-4 w-4 animate-spin" v-if="isSaving" />
          Save to Device
        </Button>
      </div>
    </div>

    <Card>
      <CardHeader>
        <CardTitle>Manage Widgets</CardTitle>
        <CardDescription>
          Add, edit, or remove widgets that appear on your device's dashboard.
          Remember to save your changes to the device.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div class="mb-4">
          <Button @click="openAddWidgetDialog">
            <PlusCircle class="mr-2 h-4 w-4" /> Add New Widget
          </Button>
        </div>

        <div v-if="isLoading" class="flex items-center justify-center py-6">
          <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
          <p class="ml-2">Loading widget settings...</p>
        </div>

        <template v-else-if="widgetListForDisplay.length > 0">
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
              <TableRow v-for="widget in widgetListForDisplay" :key="widget.id">
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
                v-model="widgetForm.isJson" 
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
import { ref, onMounted, watch, reactive, computed } from 'vue';
import { useWidgetSettings } from '@/composables/useWidgetSettings';
import type { WidgetFormState, WidgetItemConfigFE } from '@/types/deviceConfig';

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
// Ensure useToast is set up or placeholder is used in composable
// import { useToast } from '@/components/ui/toast/use-toast';

const {
  isLoading,
  isSaving,
  widgetListForDisplay,
  hasPendingChanges,
  loadDeviceConfig,
  addWidget,
  updateWidget,
  deleteWidget,
  saveWidgetChanges,
  discardWidgetChanges,
  convertWidgetItemToForm,
  getNextWidgetId,
  displayWidgets, // Used to get widget for editing
} = useWidgetSettings();

// const { toast } = useToast(); // If using toast directly here
const toast = (options: any) => { // Placeholder toast if not using global
    console.log('Toast:', options.title, options.description);
    if (options.variant === 'destructive') console.error('Error Toast:', options.title, options.description);
  };

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

// Computed property to interface with v-model for jsonPointer
const formJsonPointer = computed({
  get: () => widgetForm.jsonPointer === null ? undefined : widgetForm.jsonPointer,
  set: (val) => {
    widgetForm.jsonPointer = val === undefined || val === '' ? null : val;
  }
});

onMounted(() => {
  loadDeviceConfig();
});

const resetForm = () => {
  Object.assign(widgetForm, initialFormState);
  editingWidgetId.value = null;
};

const onWidgetTypeChange = (newType: string) => {
  // Validate and cast the input from RadioGroup
  if (newType === 'Text' || newType === 'Image') {
    widgetForm.type = newType as 'Text' | 'Image'; // Cast to the specific union type
    if (widgetForm.type === 'Image') {
      widgetForm.isJson = false;
      widgetForm.jsonPointer = null;
    }
  } else {
    // Handle unexpected value if necessary, though RadioGroup should limit to defined values
    console.warn('Unexpected widget type from RadioGroup:', newType);
    // Optionally reset to a default or handle error
    widgetForm.type = 'Text'; // Default fallback
  }
};

const openAddWidgetDialog = () => {
  resetForm();
  widgetForm.id = null; // Explicitly for clarity, though resetForm does it
  isDialogVisible.value = true;
};

const openEditWidgetDialog = (widgetId: number) => {
  // Find from displayWidgets as it reflects pending changes too
  const widgetToEdit = displayWidgets.value[widgetId];
  if (widgetToEdit) {
    const formValues = convertWidgetItemToForm(widgetId, widgetToEdit);
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
    // Update existing widget
    const success = updateWidget(editingWidgetId.value, { ...widgetForm });
    if (success) closeDialog();
  } else {
    // Add new widget
    const success = addWidget({ ...widgetForm });
    if (success) closeDialog();
  }
  // Changes are staged; user still needs to click "Save to Device"
};

const deleteWidgetHandler = (id: number) => {
  // Optional: Add a confirmation dialog here
  deleteWidget(id);
  // Widget is now marked for deletion, user needs to save.
};

const saveWidgetChangesHandler = async () => {
  await saveWidgetChanges();
  // State should be reloaded within saveWidgetChanges, pending changes cleared.
};

const discardWidgetChangesHandler = () => {
  discardWidgetChanges();
};

// Watch for dialog close to reset form if it wasn't submitted
watch(isDialogVisible, (newValue) => {
  if (!newValue) {
    // Ensure form is reset if dialog is closed via overlay click or Esc key
    // but only if it wasn't a successful submit (which calls closeDialog -> resetForm)
    // This check might be tricky; for simplicity, resetForm is often called in closeDialog directly.
    // If a submit didn't happen, resetForm() hasn't been called yet via handleFormSubmit -> closeDialog.
  }
});

</script> 