# Guia de Uso del Auditor

## Ejecutar una Auditoria

### Desde la Interfaz Web

1. Navega a `http://localhost:8080`
2. Ingresa la URL objetivo
3. Selecciona las categorias de auditoria usando las pestañas (ALL ejecuta las cuatro categorias)
4. Elige el modo de auditoria:
   - **SINGLE URL**: audita una pagina
   - **FULL SITE**: rastrea desde la URL dada y audita todas las paginas descubiertas (max 50)
5. Para modo FULL SITE, activa opcionalmente INCLUDE SUBDOMAINS
6. Haz clic en START AUDIT
7. Observa los resultados en tiempo real

### Desde la API

Abre un WebSocket a:

```
ws://localhost:8080/api/audit/live?url=<URL>&mode=single&subdomains=false&checks=performance,seo,accessibility,best_practices
```

## Categorias de Auditoria

### Rendimiento

Evalua la eficiencia de carga de la pagina:

- Peso total de pagina (bytes totales, numero de peticiones)
- Cascada de carga de recursos (bloqueante vs diferido)
- Optimizacion de imagenes (dimensiones faltantes, formato incorrecto)
- Estrategia de carga de fuentes (swap vs block)
- Politica de cache (Cache-Control, ETag, Last-Modified)
- Negociacion de compresion (Brotli, gzip)
- Recursos que bloquean el renderizado
- Candidatos a Core Web Vitals (elemento LCP, deteccion de CLS)

### SEO

Evalua la visibilidad en buscadores:

- Etiqueta title: presencia, longitud, unicidad
- Meta description: presencia y calidad
- Jerarquia de encabezados: validacion de orden h1-h6, niveles faltantes
- Deteccion de URL canonica
- Etiquetas Open Graph (og:title, og:type, og:image, og:url, og:description)
- Etiquetas Twitter Card
- Extraccion de datos estructurados JSON-LD
- Directivas meta robots
- Presencia de hreflang
- Atributo lang en HTML

### Accesibilidad

Evalua el cumplimiento de WCAG 2.2:

- Presencia de texto alternativo en imagenes
- Estructura de encabezados y esquema del documento
- Uso de atributos ARIA (roles, etiquetas, descripciones)
- Deteccion de elementos landmark
- Asociaciones de etiquetas de formulario (label-for, aria-label)
- Navegacion por teclado (valores de tabindex)
- Calidad del texto de enlaces (descriptivo vs generico)
- Estructura de tablas (caption, scope, headers)
- Atributos title en iframes
- Configuracion de viewport y zoom
- Atributo de idioma
- Contraste de color (nota informativa)

### Buenas Practicas

Evalua seguridad y cumplimiento de estandares:

- Obligacion de HTTPS
- Headers de seguridad (HSTS, CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)
- Atributos de cookies (Secure, HttpOnly, SameSite)
- Declaracion de doctype
- Elementos HTML obsoletos
- Deteccion de contenido mixto
- Subresource Integrity (SRI)
- Deteccion de errores de consola (nota informativa)

## Interpretacion de Hallazgos

Cada hallazgo tiene:

- **Severidad**: Error (debe corregirse), Warning (deberia corregirse), Info (considerar), Pass (sin problemas)
- **Check**: Identificador corto de la comprobacion
- **Titulo**: Resumen de una linea
- **Descripcion**: Explicacion detallada, impacto y recomendacion actionable
- **Fragmento**: El elemento HTML relevante (si aplica)
- **Linea**: Numero de linea en el codigo HTML formateado

## Exportacion de Resultados

Despues de una auditoria, aparecen botones de exportacion debajo de la lista de hallazgos. Hay cinco formatos disponibles:

| Formato | Caso de Uso |
|---|---|
| CSV | Analisis en hojas de calculo, filtrado, informes |
| JSON | Procesamiento programatico, pipelines CI |
| PDF | Entregables para clientes, informes impresos |
| HTML | Informe auto-contenido con estilos |
| Markdown | Documentacion, sistemas de seguimiento de incidencias |
