# Capability: frontend-serving

## ADDED Requirements

### Requirement: Tipos MIME de la PWA
El backend SHALL servir el manifiesto web con `Content-Type: application/manifest+json` y el Service Worker con `Cache-Control: no-cache`, con independencia de lo que detecte por defecto la biblioteca de tipos MIME.

#### Scenario: MIME del manifiesto
- GIVEN un manifiesto `manifest.webmanifest` empaquetado
- WHEN se solicita al backend
- THEN la respuesta incluye `Content-Type: application/manifest+json`

#### Scenario: Cache-Control del Service Worker
- GIVEN el fichero `sw.js` empaquetado
- WHEN se solicita al backend
- THEN la respuesta incluye `Cache-Control: no-cache`
