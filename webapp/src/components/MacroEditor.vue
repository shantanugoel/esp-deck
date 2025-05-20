<template>
  <div v-if="open" class="w-full mx-auto p-4">
    <h3 class="text-lg font-bold mb-4">Macro Sequence Editor</h3>
    <div class="flex flex-col sm:flex-row gap-4">
      <!-- Sidebar: Action Palette -->
      <div class="flex flex-row sm:flex-col gap-2 min-w-[140px] max-w-[180px]">
        <button v-for="actionItem in actionPalette" :key="actionItem.type" @click="addAction(actionItem.type)"
          class="px-3 py-1 rounded bg-muted text-muted-foreground hover:bg-primary/10 border border-muted text-sm w-full">
          + {{ actionItem.label }}
        </button>
      </div>
      <!-- Macro Sequence List -->
      <div class="flex-1 min-w-0">
        <div class="relative">
          <div v-if="sequence.length === 0" class="text-muted-foreground text-sm mb-2">No actions yet. Add actions from the left.</div>
          <div class="max-h-[60vh] overflow-y-auto pr-1">
            <VueDraggable v-model="sequence" class="space-y-2" itemKey="type">
              <div v-for="(act, idx) in sequence" :key="idx">
                <li class="flex items-center gap-2 bg-card rounded border border-muted px-2 py-1 shadow-sm hover:shadow transition-all">
                  <span class="cursor-grab flex items-center pr-2 select-none text-muted-foreground">
                    <svg xmlns='http://www.w3.org/2000/svg' class='w-4 h-4' fill='none' viewBox='0 0 24 24' stroke='currentColor'><path stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M9 6h.01M9 12h.01M9 18h.01M15 6h.01M15 12h.01M15 18h.01'/></svg>
                  </span>
                  <div class="flex-1 min-w-0">
                    <div class="font-mono text-xs text-primary flex items-center gap-2">
                      <span v-html="getActionSummary(act)"></span>
                    </div>
                    <component :is="getActionEditor(act)" :action="act" @update="updateAction(idx, $event)" />
                  </div>
                  <span
                    @click="removeAction(idx)"
                    tabindex="0"
                    class="ml-2 w-8 h-8 flex items-center justify-center text-xl text-destructive hover:text-destructive/80 cursor-pointer focus-visible:ring focus-visible:ring-destructive/40 rounded"
                    role="button"
                    aria-label="Delete"
                    title="Delete"
                  >
                    🗑️
                  </span>
                </li>
              </div>
            </VueDraggable>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, defineProps, defineEmits } from 'vue'
import { VueDraggable } from 'vue-draggable-plus';
import KeyPressEditor from './KeyPressEditor.vue'
import MousePressEditor from './MousePressEditor.vue'
import MouseMoveEditor from './MouseMoveEditor.vue'
import MouseWheelEditor from './MouseWheelEditor.vue'
import ConsumerPressEditor from './ConsumerPressEditor.vue'
import DelayEditor from './DelayEditor.vue'
import SequenceEditor from './SequenceEditor.vue'
import UnknownEditor from './UnknownEditor.vue'
import KeyReleaseEditor from './KeyReleaseEditor.vue'
import SendStringEditor from './SendStringEditor.vue'
import { stringToKeyCodes, keyCodesToString } from '../keycodes'

// Import ConfigAction and potentially other action types if needed for casting
import type { ConfigAction, ConfigActionKeyPress, ConfigActionDelay, ConfigActionMousePress, ConfigActionMouseMove, ConfigActionMouseWheel, ConfigActionConsumerPress, ConfigActionSendString, ConfigActionSequence } from '@/types/protocol';

// Action type definitions
const actionPalette = [
  { type: 'KeyPress', label: 'KeyPress' },
  { type: 'KeyRelease', label: 'KeyRelease' },
  { type: 'MousePress', label: 'Mouse Press' },
  { type: 'MouseMove', label: 'Mouse Move' },
  { type: 'MouseWheel', label: 'Mouse Wheel' },
  { type: 'ConsumerPress', label: 'Media Key' },
  { type: 'Delay', label: 'Delay' },
  { type: 'Sequence', label: 'Nested Sequence' },
  { type: 'SendString', label: 'Send String' },
]

const props = defineProps<{
  modelValue: ConfigAction[],
  open: boolean
}>()
const emit = defineEmits<{ (e: 'update:modelValue', value: ConfigAction[]): void }>()

const sequence = ref<ConfigAction[]>([])

watch(() => props.modelValue, (val) => {
  sequence.value = JSON.parse(JSON.stringify(val || []));
}, { immediate: true, deep: true })

function emitSequence() {
  emit('update:modelValue', JSON.parse(JSON.stringify(sequence.value)));
}

function addAction(type: string) {
  let newAction: ConfigAction | null = null;
  switch (type) {
    case 'KeyPress':
      newAction = { type: 'KeyPress', keys: [''], modifier: null };
      break;
    case 'KeyRelease':
      newAction = { type: 'KeyRelease' };
      break;
    case 'MousePress':
      newAction = { type: 'MousePress', button: 1 };
      break;
    case 'MouseMove':
      newAction = { type: 'MouseMove', dx: 0, dy: 0 };
      break;
    case 'MouseWheel':
      newAction = { type: 'MouseWheel', amount: 1 };
      break;
    case 'ConsumerPress':
      newAction = { type: 'ConsumerPress', usage_id: 0xE9 };
      break;
    case 'Delay':
      newAction = { type: 'Delay', ms: 100 };
      break;
    case 'Sequence':
      newAction = { type: 'Sequence', actions: [] };
      break;
    case 'SendString':
      newAction = { type: 'SendString', keys: [], modifiers: [] };
      break;
    default:
      console.warn('Unknown action type to add:', type);
      return;
  }
  if (newAction) {
    sequence.value.push(newAction);
    emitSequence();
  }
}
function removeAction(idx: number) {
  sequence.value.splice(idx, 1)
  emitSequence()
}
function moveUp(idx: number) {
  if (idx === 0) return
  const temp = sequence.value[idx - 1]
  sequence.value[idx - 1] = sequence.value[idx]
  sequence.value[idx] = temp
  emitSequence()
}
function moveDown(idx: number) {
  if (idx === sequence.value.length - 1) return
  const temp = sequence.value[idx + 1]
  sequence.value[idx + 1] = sequence.value[idx]
  sequence.value[idx] = temp
  emitSequence()
}
function updateAction(idx: number, newAction: ConfigAction) {
  sequence.value[idx] = newAction
  emitSequence()
}

// Editor components for each action type
function getActionEditor(act: ConfigAction) {
  switch (act.type) {
    case 'KeyPress': return KeyPressEditor
    case 'KeyRelease': return KeyReleaseEditor
    case 'MousePress': return MousePressEditor
    case 'MouseMove': return MouseMoveEditor
    case 'MouseWheel': return MouseWheelEditor
    case 'ConsumerPress': return ConsumerPressEditor
    case 'Delay': return DelayEditor
    case 'Sequence': return SequenceEditor
    case 'SendString': return SendStringEditor
    default: return UnknownEditor
  }
}

// Move getActionSummary above the template so it is available for template usage
function getActionSummary(act: ConfigAction): string {
  switch (act.type) {
    case 'KeyPress':
      const kpAction = act as ConfigActionKeyPress;
      const keyLabel = kpAction.keys.length > 0 ? kpAction.keys.map((k: string) => k || '<key>').join(' + ') : '<key>';
      const mod = kpAction.modifier ? ` + ${kpAction.modifier}` : '';
      return `<b>KeyPress:</b> ${keyLabel}${mod}`;
    case 'SendString':
      const ssAction = act as ConfigActionSendString;
      if (ssAction.keys.length > 0) {
        const keys = ssAction.keys.map((k: string, i: number) => {
          const m = ssAction.modifiers && ssAction.modifiers[i] ? `${ssAction.modifiers[i]} + ` : '';
          return `${m}${k}`;
        }).join(', ');
        return `<b>SendString:</b> ${keys}`;
      }
      return '<b>SendString:</b> <empty>';
    case 'MousePress':
      const mpAction = act as ConfigActionMousePress;
      const btn = mpAction.button === 1 ? 'Left' : mpAction.button === 2 ? 'Right' : mpAction.button === 4 ? 'Middle' : `Button ${mpAction.button}`;
      return `<b>MousePress:</b> ${btn}`;
    case 'MouseMove':
      const mmAction = act as ConfigActionMouseMove;
      return `<b>MouseMove:</b> dx=${mmAction.dx}, dy=${mmAction.dy}`;
    case 'MouseWheel':
      const mwAction = act as ConfigActionMouseWheel;
      return `<b>MouseWheel:</b> amount=${mwAction.amount}`;
    case 'ConsumerPress':
      const cpAction = act as ConfigActionConsumerPress;
      return `<b>Media Key:</b> usage_id=0x${cpAction.usage_id.toString(16).toUpperCase()}`;
    case 'Delay':
      const dAction = act as ConfigActionDelay;
      return `<b>Delay:</b> ${dAction.ms}ms`;
    case 'Sequence':
      const seqAction = act as ConfigActionSequence;
      return `<b>Nested Sequence</b> (${(seqAction.actions.length)} actions)`;
    case 'KeyRelease':
      return `<b>KeyRelease</b>`;
    case 'MouseRelease':
      return `<b>MouseRelease</b>`;
    case 'ConsumerRelease':
      return `<b>ConsumerRelease</b>`;
    default:
      const _exhaustiveCheck: never = act;
      return `<b>Unknown Action:</b> ${JSON.stringify(_exhaustiveCheck)}`;
  }
}
</script> 