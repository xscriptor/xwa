# Librerías Python Utilizadas en el Backend de Samurai


Última actualización: 2026-04-15

## Alcance

Este documento lista las librerías Python utilizadas por el backend en `samurai/backend`, basado en:

- `samurai/backend/requirements.txt`
- importaciones directas en `samurai/backend/**/*.py`
- comando de inicio en tiempo de ejecución en `samurai/backend/Dockerfile`

## Librerías en Uso Directo

| Librería | Versión | Propósito Principal | Dónde se Usa |
|---|---:|---|---|
| fastapi | 0.105.0 | Framework de API, endpoints REST y WebSocket | `app/main.py`, `app/scanner.py`, `app/crawler.py`, `app/recon/logger.py`, `app/recon/orchestrator.py` |
| uvicorn[standard] | 0.24.0 | Servidor ASGI utilizado para ejecutar la API | `backend/Dockerfile` (`CMD ["uvicorn", ...]`) |
| SQLAlchemy | 2.0.23 | ORM, sesiones, modelos y motor | `app/database.py`, `app/models.py`, `app/main.py`, `app/scanner.py`, `app/crawler.py` |
| psycopg2-binary | 2.9.9 | Driver PostgreSQL utilizado por SQLAlchemy en tiempo de ejecución | `app/database.py` (URL PostgreSQL), dependencia de runtime de SQLAlchemy |
| requests | 2.31.0 | Peticiones HTTP síncronas para sondeos de escaneo/crawling | `app/scanner.py`, `app/crawler.py` |
| beautifulsoup4 | 4.12.2 | Análisis HTML y extracción de formularios/DOM | `app/scanner.py`, `app/crawler.py` |
| playwright | 1.52.0 | Análisis headless runtime para mapeo de superficie JS | `app/crawler.py` (`from playwright.async_api import async_playwright`) |
| dnspython | 2.5.0 | Resolución DNS para módulos de reconocimiento | `app/recon/modules/dns_enumerator.py`, `app/recon/modules/subdomain_enumerator.py` |
| httpx | 0.25.2 | Cliente HTTP asíncrono para módulos de reconocimiento | `app/recon/modules/api_discovery.py`, `app/recon/modules/security_headers.py`, `app/recon/modules/subdomain_enumerator.py`, `app/recon/modules/technology_stack.py` |
| tldextract | 5.1.1 | Extracción de dominio registrable para lógica de subdominios | `app/recon/modules/subdomain_enumerator.py` |

## Dependencia de Prueba/Utilidad (Sin Versión Fija Directa)

| Librería | Fuente | Propósito | Dónde se Usa |
|---|---|---|---|
| websockets | Generalmente proporcionado por `uvicorn[standard]` | Cliente WebSocket para pruebas locales del backend | `backend/test_ws.py`, `backend/test_ws2.py` |

## Librerías Declaradas en requirements Sin Importaciones Directas Actuales

Las siguientes librerías están presentes en `requirements.txt` pero no aparecen actualmente como importaciones directas en el código del backend:

- celery
- redis
- aiohttp
- cryptography
- python-whois
- certifi

Razones:

- Evaluación de implementaciones futuras.
