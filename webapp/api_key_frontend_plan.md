# Frontend API Key Management Plan

This document outlines the plan for implementing API key management in the frontend application, focusing on minimal changes and adherence to the existing architecture.

## 1. Requirements

-   **View API Key:** Users should be able to see the status of the API key (e.g., "Set", "Not Set"). A "Copy API Key" button should be available if a key is set.
-   **Generate API Key:** Users should be able to generate a new API key (UUID).
-   **Clear API Key:** Users should be able to clear/remove an existing API key.
-   **Staged Changes:** All API key modifications (generation, clearing) must be staged locally and not immediately saved to the device.
-   **Save Workflow Integration:** Staged API key changes must be part of the existing "Save Settings" workflow in `DashboardView.vue`.
-   **Selective Save:** Users must be able to selectively include or exclude API key changes during the save operation via the `SaveSettingsModal.vue`.
-   **State Management:** The `deviceStore.ts` will be the source of truth for API key state and modifications.
-   **User Feedback:** Clear feedback should be provided for actions like key generation, clearing, and copying (e.g., using toasts), and to indicate that changes require saving.

## 2. Architecture

The implementation will leverage the existing Vue 3 (Composition API, TypeScript, Pinia) architecture.

-   **State Management (`deviceStore.ts`):**
    -   `deviceConfig.value.settings.api_key` will hold the current, potentially staged, API key.
    -   `lastFetchedConfig.value.settings.api_key` will represent the last known API key persisted on the device (used as a baseline for detecting changes).
    -   The existing `updateDeviceApiKey(newApiKey: string | null)` action will be used to stage changes to `deviceConfig.value.settings.api_key`.
    -   The existing `apiKey` computed property (`deviceStore.apiKey`) will provide reactive access to the staged API key.

-   **UI Components:**
    -   **`ApiKeyManager.vue` (To be created/re-created):**
        -   A dedicated component for displaying API key status, and providing "Generate", "Clear", and "Copy" actions.
        -   Interacts with `deviceStore` to read `apiKey` and call `updateDeviceApiKey`.
    -   **`DashboardView.vue`:**
        -   Hosts `ApiKeyManager.vue`.
        -   Tracks the initial API key state (from when the config was loaded or last saved) to detect local changes.
        -   A computed property `changedApiKey` will determine if the API key has been modified locally, providing `{ oldValue, newValue }` for the modal.
        -   Passes `changedApiKey` to `SaveSettingsModal.vue`.
        -   Handles the save logic based on user selection in the modal, preparing the correct `api_key` value for `deviceStore.saveConfig()`.
    -   **`SaveSettingsModal.vue`:**
        -   Receives `changedApiKey` prop.
        -   Displays a checkbox to include/exclude API key changes if `changedApiKey` indicates a modification.
        -   Shows a textual summary of the API key change (e.g., "API Key: Will be set", "API Key: Will be cleared").
        -   Emits the user's selection for the API key back to `DashboardView.vue`.

-   **Data Flow for API Key Change & Save:**
    1.  Config loaded: `deviceStore.deviceConfig` and `initialApiKey` (in `DashboardView.vue`) are populated.
    2.  `ApiKeyManager.vue` (via `deviceStore.apiKey`) displays current key status.
    3.  User action (Generate/Clear) in `ApiKeyManager.vue` calls `deviceStore.updateDeviceApiKey()`.
    4.  `deviceStore` updates `deviceConfig.value.settings.api_key` (staged change).
    5.  `DashboardView.vue`'s `changedApiKey` computed property detects the change.
    6.  User clicks "Save Settings"; `SaveSettingsModal.vue` is shown, displaying the API key change and checkbox.
    7.  User confirms selection in modal.
    8.  `DashboardView.vue`'s `handleSaveModalConfirm` adjusts `configToSave.settings.api_key` based on modal selection (apply staged change or revert to initial value).
    9.  `deviceStore.saveConfig()` sends the payload. On success, `lastFetchedConfig` (and `initialApiKey` in `DashboardView`) are updated.

## 3. Low-Level Design and Implementation

-   **`deviceStore.ts`:**
    -   Type `DeviceConfig.settings.api_key?: string | null` is appropriate.
    -   Action `updateDeviceApiKey(newApiKey: string | null)`: Ensure it robustly handles `deviceConfig` or `settings` being initially null (current implementation seems to cover this).
    -   Computed `apiKey`: `computed(() => deviceConfig.value?.settings?.api_key || null)`.

-   **`ApiKeyManager.vue` (New/Re-created Component):**
    -   Located at `webapp/src/components/ApiKeyManager.vue`.
    -   Uses `useDeviceStore()`.
    -   Displays:
        -   Text: "API Key: Set" or "API Key: Not Set" based on `deviceStore.apiKey`.
    -   Buttons (Shadcn-Vue `Button`):
        -   "Generate New API Key": Calls `crypto.randomUUID()`, then `deviceStore.updateDeviceApiKey(uuid)`. Toast: "New API key generated. Save settings to apply."
        -   "Clear API Key": Calls `deviceStore.updateDeviceApiKey(null)`. Toast: "API key cleared. Save settings to apply."
        -   "Copy API Key": Uses `navigator.clipboard.writeText(deviceStore.apiKey)`. Disabled if no key is set. Toast: "API Key copied" or "No API Key to copy."
    -   Toasts via `vue-sonner`.

-   **`DashboardView.vue`:**
    -   `initialApiKey = ref<string | null>(null)`: Updated in `watch` for `deviceStore.deviceConfig`.
    -   `changedApiKey = computed(...)`: Compares `deviceStore.deviceConfig?.settings?.api_key` with `initialApiKey.value`. Returns `{ oldValue: string | null, newValue: string | null } | null`.
    -   `hasSettingsChanged`: Include `!!changedApiKey.value`.
    -   Pass `:changed-api-key="changedApiKey"` to `SaveSettingsModal`.
    -   `handleSaveModalConfirm(selection)`:
        -   Deep copy `deviceStore.deviceConfig` to `configToSave`.
        -   If `selection.apiKey === true` AND `changedApiKey.value`:
            `configToSave.settings.api_key = changedApiKey.value.newValue;`
        -   Else if `selection.apiKey === false` AND `changedApiKey.value`:
            `configToSave.settings.api_key = changedApiKey.value.oldValue;`
        -   Else (no user change to API key, or not part of modal selection): `configToSave.settings.api_key` (from the deep copy) will naturally hold the correct current value (either unchanged original, or user-staged if `changedApiKey.value` was null because it was the first value set).

-   **`SaveSettingsModal.vue`:**
    -   Props: `changedApiKey: { oldValue: string | null, newValue: string | null } | null`.
    -   State: `selected.apiKey: boolean`, initialized to `!!props.changedApiKey`.
    -   Template:
        -   Checkbox "API Key" `v-if="changedApiKey"` bound to `selected.apiKey`.
        -   Textual summary `v-if="changedApiKey"`:
            -   If `!changedApiKey.oldValue && changedApiKey.newValue`: "API Key: Will be set."
            -   If `changedApiKey.oldValue && !changedApiKey.newValue`: "API Key: Will be cleared."
            -   If `changedApiKey.oldValue && changedApiKey.newValue`: "API Key: Will be changed."
    -   Emit `apiKey: selected.value.apiKey` in confirm event.

## 4. Phase-wise Plan

-   **Phase 1: `deviceStore.ts` Baseline Verification**
    -   Confirm `DeviceConfig` type and `api_key` field.
    -   Confirm `updateDeviceApiKey` action stages changes correctly.
    -   Confirm `apiKey` computed property.
    -   *Goal: Store is ready for API key logic.*

-   **Phase 2: `ApiKeyManager.vue` Implementation**
    -   Create `webapp/src/components/ApiKeyManager.vue`.
    -   Implement UI (display, generate/clear/copy buttons).
    -   Integrate with `deviceStore` actions and computed property.
    -   Add toast notifications.
    -   *Goal: User can view status and stage API key changes locally.*

-   **Phase 3: `DashboardView.vue` - Change Detection & Modal Input**
    -   Add `ApiKeyManager.vue` to template.
    -   Implement `initialApiKey` ref and update it via watcher on `deviceStore.deviceConfig`.
    -   Implement `changedApiKey` computed property.
    -   Update `hasSettingsChanged` computed.
    -   Pass `changedApiKey` prop to `SaveSettingsModal`.
    -   *Goal: Dashboard correctly detects staged API key changes and informs the save modal.*

-   **Phase 4: `SaveSettingsModal.vue` - Display & Selection**
    -   Verify prop `changedApiKey` handling.
    -   Conditionally render checkbox and change summary for API key.
    -   Ensure `selected.apiKey` is correctly managed and emitted.
    -   (Most of this might be in place from previous work, verify against `DashboardView` changes).
    -   *Goal: Modal allows user to confirm/reject saving API key changes.*

-   **Phase 5: `DashboardView.vue` - Save Logic Implementation**
    -   Refine `handleSaveModalConfirm` to accurately set `configToSave.settings.api_key` based on `selection.apiKey` and `changedApiKey.value` (applying new value, reverting to old value, or preserving current if no change/selection).
    -   Call `deviceStore.saveConfig(configToSave)`.
    -   *Goal: API key changes are correctly included/excluded in the save payload based on user choice.*

-   **Phase 6: End-to-End Testing & Refinement**
    -   Test all scenarios: generate, clear, save selected, save unselected, no changes.
    -   Verify behavior with and without an existing API key.
    -   Check toast notifications and UI feedback.
    -   Address any linter errors or console warnings.
    -   *Goal: A robust and user-friendly API key management feature.* 