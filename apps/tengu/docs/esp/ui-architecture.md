# Arquitectura de la UI de Tengu

## Framework

Angular 19 standalone (sin NgModules). La UI sigue el Nothing Design System definido en las referencias de Samurai.

## Arbol de Componentes

```
AppComponent (shell)
├── Sidebar
│   ├── Marca (TENGU / XWA - MODULE)
│   ├── Enlaces de navegacion (AUDIT, HISTORY)
│   └── Footer
│       ├── Alternador de tema
│       └── Enlaces sociales (GitHub, repo, dev site)
└── Router outlet
    ├── AuditComponent
    │   ├── Formulario de auditoria (URL, modo, pestañas de categoria)
    │   ├── Resumen de severidad (conteo de error, warning, info, pass)
    │   ├── Pestañas de resultado (filtro por categoria)
    │   ├── Lista de hallazgos en acordeon
    │   ├── Visor de codigo HTML con resaltado de lineas
    │   └── AuditExportActionsComponent
    └── HistoryComponent
        ├── Boton de actualizar
        ├── Tabla de auditorias (URL, estado, hallazgos, fecha, enlace de carga)
        └── Visualizacion de errores
```

## Tokens del Sistema de Diseno

Todos los tokens se definen como propiedades CSS personalizadas en `styles.scss`:

- **Fuentes**: Space Grotesk (cuerpo/titulos), Space Mono (datos/codigo), Doto (display)
- **Esquema de color**: Modo oscuro por defecto, modo claro via clase `.theme-light` en `<body>`
- **Escala de espaciado**: Base 8px -- 2xs (4px), xs (8px), sm (12px), md (16px), lg (24px), xl (32px), 2xl (48px), 3xl (64px), 4xl (96px)
- **Motivo de puntos**: Patron de fondo via superposiciones CSS gradient

## Patrones Clave

### Componentes Standalone
Cada componente es `standalone: true`. Sin NgModules. La funcionalidad compartida (ThemeService) se inyecta directamente.

### Streaming WebSocket
El componente de auditoria abre un WebSocket a `/api/audit/live` con parametros de consulta para URL, modo, subdominios y comprobaciones. Los mensajes siguen un protocolo de texto simple:

| Prefijo | Contenido |
|---|---|
| `[AUDIT]` | Mensaje de registro de estado |
| `[PAGE]` | URL de pagina descubierta |
| `[HTML]` | Codigo fuente HTML completo formateado |
| `[done]` | Auditoria completada exitosamente |
| `[!]` | Mensaje de error |
| Objeto JSON | Un hallazgo individual |

### Deteccion de Cambios
Los callbacks de WebSocket llaman a `ChangeDetectorRef.detectChanges()` manualmente para actualizar la vista fuera de la zona de Angular.

### Exportacion desde el Cliente
Las exportaciones generan contenido en memoria, crean un Blob y activan la descarga mediante un elemento anchor temporal. No hay generacion de archivos en el servidor.

### Resaltado de Lineas
Los hallazgos incluyen un fragmento del elemento HTML ofensivo. Despues del formateo, cada elemento ocupa su propia linea, y el frontend empareja fragmentos con lineas por nombre de etiqueta y valores de atributos clave.

## Rutas

| Ruta | Componente | Descripcion |
|---|---|---|
| `/audit` | AuditComponent | Ejecutar y ver auditorias |
| `/history` | HistoryComponent | Navegar por auditorias pasadas |
| `/` | (redireccion) | Redirige a `/audit` |

## Servicio de Tema

`ThemeService` usa Angular Signals para el estado oscuro/claro. El tema actual se persiste en `localStorage` bajo la clave `tengu-theme`. Al alternar se agrega/elimina la clase `.theme-light` en `<body>`.
