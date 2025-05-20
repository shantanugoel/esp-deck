<script setup lang="ts">
import type { ConfigActionKeyRelease } from '@/types/protocol'; // Import the specific type

// Props: Expect the specific ConfigActionKeyRelease type
const props = defineProps<{ action: ConfigActionKeyRelease }>();

// No emits needed if there are no editable fields and the action object itself doesn't change structurally by this editor.
// If an @update was strictly required by MacroEditor even for no-op, it would emit props.action or { type: 'KeyRelease' }.

</script>
<template>
  <div class="text-muted-foreground text-sm py-1 pl-1">
    <!-- Displaying props.action.type to be explicit, though it's known -->
    <span class="font-mono text-xs">{{ props.action.type }}</span>: No parameters to edit.
  </div>
</template> 