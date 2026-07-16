# Manual de Tengu

## Despliegue

### Docker (recomendado)

```bash
docker compose up -d --build
```

Interfaz web en `http://localhost:8080`.

### Standalone

```bash
cargo run --release
```

Requiere Rust toolchain 1.81+. El frontend se compila automaticamente durante el build. Interfaz web en `http://localhost:8080`.

### Script de Inicio

```bash
./tengu.sh                       # Standalone efimero (sin estado en disco)
./tengu.sh --docker               # via docker-compose
./tengu.sh --docker -e            # Docker efimero (--rm, limpia automaticamente)
./tengu.sh --export archivo.json  # Guardar exportacion en ./exports/
./tengu.sh --import archivo.json  # Importar JSON de auditoria
./tengu.sh --build                # Solo compilar (sin ejecutar)
```

## Variables de Entorno

| Variable | Por Defecto | Descripcion |
|---|---|---|
| `PORT` | `8080` | Puerto de escucha HTTP |
| `RUST_LOG` | `tengu=info,tower_http=info` | Verbosidad de logging (formato env-filter) |

## Uso

### Auditoria de URL Unica

1. Ingresa una URL en el campo de texto
2. Selecciona las categorias de auditoria (Rendimiento, SEO, Accesibilidad, Buenas Practicas)
3. Haz clic en START AUDIT
4. Los resultados se transmiten en tiempo real via WebSocket

### Auditoria de Sitio Completo

1. Activa el modo FULL SITE
2. Opcionalmente activa INCLUDE SUBDOMAINS
3. Ingresa la URL inicial
4. Tengu rastrea las paginas descubiertas (hasta 50) y audita cada una

### Interpretacion de Resultados

Cada hallazgo muestra:
- **Severidad**: Error, Warning, Info o Pass
- **Check**: Nombre de la comprobacion especifica
- **Titulo**: Resumen del problema
- **Descripcion**: Explicacion detallada con recomendaciones
- **Fragmento**: El elemento HTML relevante (si aplica)
- **Linea**: Numero de linea en el codigo HTML formateado

### Visor de Codigo Fuente

Despues de una auditoria, haz clic en VIEW HTML para ver el codigo fuente formateado. Las lineas con hallazgos se resaltan con un borde rojo a la izquierda.

### Exportacion

Los resultados se pueden exportar en cinco formatos:

| Formato | Extension | Contenido |
|---|---|---|
| CSV | `.csv` | Datos tabulares con todos los campos |
| JSON | `.json` | Payload completo con metadatos |
| PDF | `.pdf` | Informe apaisado con tabla de hallazgos |
| HTML | `.html` | Informe auto-contenido con estilos |
| Markdown | `.md` | Informe de texto ligero |

## Endpoints de API

| Metodo | Ruta | Descripcion |
|---|---|---|
| `GET` | `/api/health` | Health check |
| `GET` | `/api/audit/live` | WebSocket para auditoria en tiempo real |
| `GET` | `/api/audits` | Listar auditorias pasadas |
| `GET` | `/api/audits/:id` | Obtener detalle de auditoria |
| `DELETE` | `/api/audits/:id` | Eliminar auditoria |
| `GET` | `/api/audits/export` | Exportar todas las auditorias como JSON |
| `POST` | `/api/audits/import` | Importar auditorias desde JSON |

## Solucion de Problemas

### La conexion WebSocket falla
Asegurate de que el puerto sea accesible y que ningun firewall bloquee las actualizaciones WebSocket. Usa `RUST_LOG=debug` para ver los detalles de las peticiones.

### La auditoria no devuelve hallazgos
La auditoria analiza el HTML despues de formatearlo. Si la pagina esta vacia, detras de un login o bloquea bots, los resultados pueden estar vacios. Prueba con una pagina accesible publicamente.

### Error de compilacion
- Rust: asegura tener 1.81+ con `rustup update`
- Frontend: `cd frontend && npm ci` para instalar dependencias
- Compilacion limpia: `./clean.sh` y reintenta
