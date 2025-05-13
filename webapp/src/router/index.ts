import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
    {
        path: '/',
        name: 'Dashboard',
        component: () => import('@/views/DashboardView.vue'),
    },
    {
        path: '/edit-macro/:buttonIndex',
        name: 'edit-macro',
        component: () => import('@/views/EditMacroView.vue'),
        props: true,
    },
]

export const router = createRouter({
    history: createWebHistory(),
    routes,
}) 