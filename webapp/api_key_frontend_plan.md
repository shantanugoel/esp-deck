# Frontend API Key Management Plan

This document outlines the plan for implementing API key management in the frontend application, adhering to the existing pattern for settings like Wi-Fi and Timezone.

## 1. Requirements

-   **View API Key:** Users should be able to see the current API key (or its status, e.g., "Not Set").
-   **Generate API Key:** Users should be able to generate a new API key (UUID).
-   **Clear API Key:** Users should be able to clear/remove an existing API key.
-   **Copy API Key:** A button to copy the current API key to the clipboard.
-   **Local Staging:** All API key modifications (generation, clearing) must be staged locally within `DashboardView.vue`'s context, not immediately saved to the device or directly to `deviceStore`.
-   **Save Workflow Integration:** Staged API key changes must be part of the existing "Save Settings" workflow in `DashboardView.vue`, managed via `SaveSettingsModal.vue`.
-   **Selective Save:** Users must be able to selectively include or exclude API key changes during the save operation.
-   **User Feedback:** Clear feedback should be provided for actions (e.g., using toasts).

## 2. Architecture (Revised)

The implementation will leverage the existing Vue 3 (Composition API, TypeScript, Pinia) architecture.

-   **State Management (`deviceStore.ts`):**
    -   **No API-key-specific changes.** The API key (e.g., `api_key: string | null`) is expected to be a standard field within the device configuration's `settings` object (e.g., `deviceConfig.config.settings.api_key`).
    -   `deviceStore` will continue to load and save the entire device configuration, including the `settings` object, without specific knowledge of the `api_key` field beyond its type if defined in `DeviceConfig`.

-   **UI Components:**
    -   **`ApiKeyManager.vue` (New Component):**
        -   A UI component responsible for displaying the API key and providing "Generate", "Clear", and "Copy" buttons.
        -   Props: `modelValue: string | null` (for v-model binding with `DashboardView.vue`, representing the current staged API key).
        -   Local state: Manages its display based on `modelValue`.
        -   Emits: `@update:modelValue (newValue: string | null)` when "Generate" or "Clear" is clicked.
    -   **`DashboardView.vue`:**
        -   Will instantiate and use `ApiKeyManager.vue`.
        -   Local State:
            -   `originalApiKey = ref<string | null>(null)`: Populated from `deviceStore.deviceConfig.config.settings.api_key` (or the actual path) upon loading configuration. This is the last saved state.
            -   `stagedApiKey = ref<string | null>(null)`: Bound to `ApiKeyManager.vue` using `v-model`. Initialized with `originalApiKey.value`.
        -   Computed: `changedApiKey = computed(() => { ... })`: Compares `stagedApiKey.value` with `originalApiKey.value`. Returns `{ oldValue: string | null, newValue: string | null } | null`.
        -   Passes `changedApiKey` to `SaveSettingsModal.vue`.
        -   Handles save logic: if API key change is confirmed in modal, `stagedApiKey.value` is used for `configToSave.config.settings.api_key`; otherwise, `originalApiKey.value` is used.
    -   **`SaveSettingsModal.vue`:**
        -   Receives `changedApiKey` prop as before.
        -   Displays checkbox and summary for API key change.
        -   Emits user's selection.

-   **Data Flow for API Key Change & Save (Revised):**
    1.  Config loaded: `deviceStore.deviceConfig` is populated. `DashboardView.vue` initializes `originalApiKey.value` and `stagedApiKey.value` from `deviceStore.deviceConfig.config.settings.api_key`.
    2.  `DashboardView.vue` passes `stagedApiKey` to `ApiKeyManager.vue` via `v-model`.
    3.  `ApiKeyManager.vue` displays the key based on its `modelValue` prop.
    4.  User action (Generate/Clear) in `ApiKeyManager.vue`:
        a.  `ApiKeyManager.vue` updates its internal representation of the key.
        b.  `ApiKeyManager.vue` emits `@update:modelValue` with the new key.
    5.  `DashboardView.vue`'s `v-model` binding updates `stagedApiKey.value`.
    6.  `DashboardView.vue`'s `changedApiKey` computed property detects the difference between `stagedApiKey.value` and `originalApiKey.value`.
    7.  User clicks "Save Settings"; `SaveSettingsModal.vue` is shown, displaying the API key change and checkbox via the `changedApiKey` prop.
    8.  User confirms selection in modal.
    9.  `DashboardView.vue`'s `handleSaveModalConfirm` prepares `configToSave`.
        -   If API key change is included: `configToSave.config.settings.api_key = stagedApiKey.value`.
        -   Else: `configToSave.config.settings.api_key = originalApiKey.value`.
    10. `deviceStore.saveConfig(configToSave)` sends the payload.
    11. On success, `deviceStore.fetchConfig()` is called. `DashboardView.vue` re-initializes `originalApiKey.value` and `stagedApiKey.value` from the new `deviceStore.deviceConfig`.

## 3. Low-Level Design and Implementation (Revised)

-   **`deviceStore.ts`:**
    -   Verification: Confirm the structure of `DeviceConfig` and the path to `settings` (e.g., `DeviceConfig.config.settings`). Ensure `settings` can accommodate an `api_key: string | null` field.
    -   **No new API-key-specific actions or computed properties.**

-   **`ApiKeyManager.vue` (New Component):**
    -   Located at `webapp/src/components/ApiKeyManager.vue`.
    -   Props: `modelValue: string | null`.
    -   Emits: `@update:modelValue (newValue: string | null)`.
    -   Internal logic:
        -   Uses `computed` for display: "API Key: Set" (if `props.modelValue`), "API Key: Not Set" (if `!props.modelValue`).
        -   Button "Generate New API Key": `const newKey = crypto.randomUUID(); emit('update:modelValue', newKey);`. Toast.
        -   Button "Clear API Key": `emit('update:modelValue', null);`. Toast.
        -   Button "Copy API Key": `navigator.clipboard.writeText(props.modelValue)`. Disabled if `!props.modelValue`. Toast.
    -   Toasts via shadcn-vue's sonner component.

-   **`DashboardView.vue`:**
    -   Refs:
        -   `originalApiKey = ref<string | null>(null)`
        -   `stagedApiKey = ref<string | null>(null)`
    -   Watch `deviceStore.deviceConfig`:
        ```typescript
        watch(() => deviceStore.deviceConfig, (config) => {
          const keyFromStore = config?.config?.settings?.api_key || null; // Adjust path as necessary
          originalApiKey.value = keyFromStore;
          stagedApiKey.value = keyFromStore;
          // ... existing wifi/tz logic ...
        }, { immediate: true, deep: true });
        ```
    -   Template: Add `<ApiKeyManager v-model="stagedApiKey" />` in the appropriate layout section (similar to Wi-Fi/Timezone inputs).
    -   Computed `changedApiKey`:
        ```typescript
        const changedApiKey = computed(() => {
          if (stagedApiKey.value !== originalApiKey.value) {
            return { oldValue: originalApiKey.value, newValue: stagedApiKey.value };
          }
          return null;
        });
        ```
    -   Update `hasSettingsChanged` computed to include `!!changedApiKey.value`.
    -   Pass `:changed-api-key="changedApiKey"` to `SaveSettingsModal`.
    -   `handleSaveModalConfirm(selection)`:
        -   In the part where `configToSave.config.settings` is prepared:
            ```typescript
            if (selection.apiKey && changedApiKey.value) {
              configToSave.config.settings.api_key = changedApiKey.value.newValue;
            } else {
              configToSave.config.settings.api_key = originalApiKey.value;
            }
            ```
            Ensure `configToSave.config.settings` object is created if it doesn't exist.

-   **`SaveSettingsModal.vue`:**
    -   Props: `changedApiKey: { oldValue: string | null, newValue: string | null } | null`. (Existing prop, ensure it's wired up).
    -   Template: Add checkbox for "API Key" `v-if="changedApiKey"` and textual summary.
    -   Emit `apiKey: boolean` (or similar key in the selection object) in confirm event.

## 4. Phase-wise Plan (Revised)

-   **Phase 1: Define API Key's Location in Configuration Data**
    -   Determine the precise path within the existing device configuration data structure where `DashboardView.vue` will place the `api_key: string | null` field (e.g., `config.settings.api_key`). This is the structure `deviceStore.saveConfig()` will receive.
    -   Review the TypeScript type definitions used by `deviceStore.ts` for the device configuration. If the type for the `settings` object is strictly defined and does not currently list `api_key` as a possible field, a minor, type-only annotation (`api_key?: string | null;`) might be added to that type definition for TypeScript correctness. This does not constitute a change to the store's runtime logic or JavaScript code.
    -   *Goal: Clearly define where the API key data will reside within the configuration object handled by `deviceStore.ts`, ensuring `DashboardView.vue` can correctly place it for saving, with absolutely no changes to `deviceStore.ts` runtime logic.*

-   **Phase 2: `ApiKeyManager.vue` Implementation**
    -   Create `webapp/src/components/ApiKeyManager.vue`.
    -   Implement props (`modelValue`), emits (`@update:modelValue`).
    -   Implement UI (display, generate/clear/copy buttons).
    -   Implement internal logic for button actions and emitting updates.
    -   Add toast notifications.
    -   *Goal: A functional `ApiKeyManager.vue` component that manages API key UI and emits changes.*

-   **Phase 3: `DashboardView.vue` - Integration of `ApiKeyManager.vue`**
    -   Add `originalApiKey` and `stagedApiKey` refs.
    -   Update watcher for `deviceStore.deviceConfig` to populate/reset these refs.
    -   Add `ApiKeyManager.vue` to the template using `v-model="stagedApiKey"`.
    -   Implement `changedApiKey` computed property.
    -   Update `hasSettingsChanged` computed.
    -   Pass `changedApiKey` to `SaveSettingsModal.vue`.
    -   *Goal: `DashboardView.vue` correctly uses `ApiKeyManager.vue`, tracks staged API key changes, and informs `SaveSettingsModal.vue`.*

-   **Phase 4: `SaveSettingsModal.vue` - Display & Selection for API Key**
    -   Ensure prop `changedApiKey` is correctly received.
    -   Add UI elements (checkbox, textual summary) for API key changes, conditioned on `changedApiKey`.
    -   Ensure the modal emits the user's selection for the API key (e.g., as part of the `selection` object).
    -   *Goal: Modal allows user to selectively include/exclude API key changes in the save operation.*

-   **Phase 5: `DashboardView.vue` - Save Logic for API Key**
    -   In `handleSaveModalConfirm`, update the `configToSave.config.settings` object to include `api_key: stagedApiKey.value` or `api_key: originalApiKey.value` based on modal selection and `changedApiKey`.
    -   Ensure `deviceStore.saveConfig(configToSave)` is called.
    -   Ensure `originalApiKey` and `stagedApiKey` are correctly reset after a successful save (typically by the `deviceStore.deviceConfig` watcher).
    -   *Goal: API key changes are correctly included/excluded in the save payload based on user choice.*

-   **Phase 6: End-to-End Testing & Refinement**
    -   Test all scenarios: generate, clear, copy, save selected, save unselected, no changes.
    -   Verify behavior with and without an existing API key.
    -   Check toast notifications and UI feedback.
    -   Address any linter errors or console warnings.
    -   *Goal: A robust and user-friendly API key management feature, consistent with existing settings management.* 