import { defineStore } from 'pinia';
import { defineAsyncComponent, shallowRef, type Component } from 'vue';

// Define Tab IDs - these must match keys in tabViewComponentsMap
const TAB_IDS = {
    MACROPAD: 'macropad',
    DASHBOARD: 'dashboard',
    NOW: 'now',
    STATUS: 'status',
    DEVICE_SETTINGS: 'device-settings',
} as const; // Use "as const" for stricter type checking and to use values in types

// Type for the keys of TAB_IDS, effectively our TabName
export type TabId = typeof TAB_IDS[keyof typeof TAB_IDS];

export type TabDefinition = {
    id: TabId;
    label: string;
    component: Component; // Using shallowRef for components as they are non-reactive objects
};

// Map of tab IDs to their lazily loaded components
// Using shallowRef for components because they are large, non-reactive objects
// and this can optimize reactivity performance.
const tabViewComponentsMap: Record<TabId, Component> = {
    [TAB_IDS.MACROPAD]: shallowRef(defineAsyncComponent(() => import('@/components/macropad/MacroPadSettingsView.vue'))),
    [TAB_IDS.DASHBOARD]: shallowRef(defineAsyncComponent(() => import('@/components/dashboard/DashboardSettingsView.vue'))),
    [TAB_IDS.NOW]: shallowRef(defineAsyncComponent(() => import('@/views/tab-views/NowView.vue'))),
    [TAB_IDS.STATUS]: shallowRef(defineAsyncComponent(() => import('@/views/tab-views/StatusView.vue'))),
    [TAB_IDS.DEVICE_SETTINGS]: shallowRef(defineAsyncComponent(() => import('@/views/tab-views/DeviceSettingsView.vue'))),
};

const defaultActiveTab: TabId = TAB_IDS.MACROPAD;

// Define the type for the store's state
interface UiStoreState {
    activeTabId: TabId;
    isDebugLogVisible: boolean;
    availableTabs: TabDefinition[];
}

export const useUiStore = defineStore('ui', {
    state: (): UiStoreState => ({
        activeTabId: defaultActiveTab,
        isDebugLogVisible: false,
        availableTabs: [
            { id: TAB_IDS.MACROPAD, label: 'MacroPad', component: tabViewComponentsMap[TAB_IDS.MACROPAD] },
            { id: TAB_IDS.DASHBOARD, label: 'Dashboard', component: tabViewComponentsMap[TAB_IDS.DASHBOARD] },
            { id: TAB_IDS.NOW, label: 'Now', component: tabViewComponentsMap[TAB_IDS.NOW] },
            { id: TAB_IDS.DEVICE_SETTINGS, label: 'Device Settings', component: tabViewComponentsMap[TAB_IDS.DEVICE_SETTINGS] },
            { id: TAB_IDS.STATUS, label: 'Status', component: tabViewComponentsMap[TAB_IDS.STATUS] },
        ] as TabDefinition[], // Keep this assertion as availableTabs structure is fixed here
    }),
    getters: {
        activeTabDefinition(state): TabDefinition | undefined {
            return state.availableTabs.find(tab => tab.id === state.activeTabId);
        },
        activeViewComponent(state): Component | null {
            const activeDef = state.availableTabs.find(tab => tab.id === state.activeTabId);
            return activeDef ? activeDef.component : null;
        },
        isTabActive: (state) => (tabId: TabId): boolean => {
            return state.activeTabId === tabId;
        },
    },
    actions: {
        setActiveTab(tabId: TabId) {
            const isValidTab = this.availableTabs.some(tab => tab.id === tabId);
            if (isValidTab) {
                this.activeTabId = tabId; // No need for 'as TabId' anymore
                console.log('Active tab set to:', tabId);
            } else {
                console.warn(`Attempted to set invalid tab ID: ${tabId}. Resetting to default.`);
                this.activeTabId = defaultActiveTab;
            }
        },
        toggleDebugLogVisibility() {
            this.isDebugLogVisible = !this.isDebugLogVisible;
            console.log('Debug log visibility toggled to:', this.isDebugLogVisible);
        },
    },
});

// For convenience, if components using the store need the TabId type directly
// export type { TabId }; // Already exported 