import { defineStore } from 'pinia';

export type TabName = 'macropad' | 'dashboard' | 'now' | 'device-settings';

export const useUiStore = defineStore('ui', {
    state: () => ({
        activeTab: 'macropad' as TabName,
        isDebugLogVisible: false,
        // other global UI states can go here
    }),
    getters: {
        isActiveTab: (state) => (tabName: TabName): boolean => {
            return state.activeTab === tabName;
        },
    },
    actions: {
        setActiveTab(tabName: TabName) {
            this.activeTab = tabName;
            console.log('Active tab set to:', tabName);
        },
        toggleDebugLogVisibility() {
            this.isDebugLogVisible = !this.isDebugLogVisible;
            console.log('Debug log visibility toggled to:', this.isDebugLogVisible);
        },
    },
}); 