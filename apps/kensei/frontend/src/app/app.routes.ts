import { Routes } from '@angular/router';

export const routes: Routes = [
    {
        path: 'profiler',
        loadComponent: () => import('./features/profiler/profiler.component').then(m => m.ProfilerComponent)
    },
    {
        path: 'js-analysis',
        loadComponent: () => import('./features/js-analysis/js-analysis.component').then(m => m.JsAnalysisComponent)
    },
    {
        path: 'spa-routes',
        loadComponent: () => import('./features/spa-routes/spa-routes.component').then(m => m.SpaRoutesComponent)
    },
    {
        path: 'history',
        loadComponent: () => import('./features/history/history.component').then(m => m.HistoryComponent)
    },
    {
        path: 'inventory',
        loadComponent: () => import('./features/inventory/inventory.component').then(m => m.InventoryComponent)
    },
    {
        path: '',
        redirectTo: '/profiler',
        pathMatch: 'full'
    }
];
