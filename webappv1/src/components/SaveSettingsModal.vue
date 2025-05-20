<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
    <div class="bg-card rounded-xl shadow-lg p-6 w-full max-w-md">
      <div class="font-semibold text-lg mb-2">Confirm Settings to Save</div>
      <div class="mb-4 text-sm text-muted-foreground">Select which settings you want to update. A summary of changes is shown below.</div>
      <form @submit.prevent="onConfirm">
        <div class="flex flex-col gap-3 mb-4">
          <label class="flex items-center gap-2">
            <input type="checkbox" v-model="selected.wifi" /> WiFi Settings
          </label>
          <label class="flex items-center gap-2">
            <input type="checkbox" v-model="selected.tz" /> Timezone Offset
          </label>
          <label class="flex items-center gap-2">
            <input type="checkbox" v-model="selected.mappings" /> Button Mappings
          </label>
          <label v-if="props.changedApiKey" class="flex items-center gap-2">
            <input type="checkbox" v-model="selected.apiKey" /> API Key
          </label>
          <label v-if="selected.mappings" class="flex items-center gap-2 ml-6 text-xs">
            <input type="checkbox" v-model="sendAllMappings" /> Send all button mappings and names (not just changed)
          </label>
        </div>
        <div class="mb-4">
          <div v-if="selected.wifi && changedWifi">
            <div class="font-medium">WiFi Changes:</div>
            <ul class="ml-4 list-disc">
              <li v-if="changedWifi.ssid">SSID: <span class="font-mono">{{ changedWifi.ssid.old }}</span> → <span class="font-mono">{{ changedWifi.ssid.new }}</span></li>
              <li v-if="changedWifi.password">Password: <span class="font-mono">••••••••</span> (changed)</li>
            </ul>
          </div>
          <div v-if="selected.tz && changedTz">
            <div class="font-medium">Timezone Offset:</div>
            <div class="ml-4">{{ changedTz.old }} → {{ changedTz.new }}</div>
          </div>
          <div v-if="selected.mappings && changedMappings && changedMappings.length">
            <div class="font-medium">Button Mappings Changed:</div>
            <div class="ml-4">Buttons: <span class="font-mono">{{ changedMappings.join(', ') }}</span></div>
          </div>
          <div v-if="selected.apiKey && props.changedApiKey" class="mt-2">
            <div class="font-medium">API Key Change:</div>
            <ul class="ml-4 list-disc">
              <li v-if="!props.changedApiKey.oldValue && props.changedApiKey.newValue">Will be set.</li>
              <li v-if="props.changedApiKey.oldValue && !props.changedApiKey.newValue">Will be cleared.</li>
              <li v-if="props.changedApiKey.oldValue && props.changedApiKey.newValue">Will be changed.</li>
            </ul>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button type="button" @click="$emit('cancel')" class="px-4 py-2 rounded bg-muted text-muted-foreground font-semibold hover:bg-muted/80">Cancel</button>
          <button type="submit" class="px-4 py-2 rounded bg-primary text-primary-foreground font-semibold shadow hover:bg-primary/80">Confirm</button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, defineProps, defineEmits } from 'vue'

type ApiKeyChange = { oldValue: string | null; newValue: string | null } | null;

const props = defineProps<{
  changedWifi: { ssid?: { old: string, new: string }, password?: boolean } | null,
  changedTz: { old: number|null, new: number|null } | null,
  changedMappings: number[],
  changedApiKey: ApiKeyChange
}>()
const emit = defineEmits(['confirm', 'cancel'])

const selected = ref({
  wifi: !!props.changedWifi,
  tz: !!props.changedTz,
  mappings: !!(props.changedMappings && props.changedMappings.length),
  apiKey: !!props.changedApiKey
})

const sendAllMappings = ref(false)
function onConfirm() {
  emit('confirm', { ...selected.value, sendAllMappings: sendAllMappings.value })
}
</script> 