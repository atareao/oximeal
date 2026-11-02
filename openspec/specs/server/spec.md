# server Specification

## Purpose
Esta capacidad define el servidor HTTP construido con Axum que expone la API de OxiMeal. Proporciona un endpoint de health check para comprobar que el servicio está vivo y publica la documentación OpenAPI de todos los endpoints disponibles.

## Requirements

### Requirement: Health check
El sistema SHALL exponer `GET /api/health` que verifica la conexión a la base de datos.

#### Scenario: Servicio sano
- GIVEN el servidor arrancado con migraciones aplicadas
- WHEN se consulta `GET /api/health`
- THEN responde 200 con estado ok

### Requirement: Documentación OpenAPI
El sistema SHALL exponer el documento OpenAPI y Swagger UI.

#### Scenario: Acceso a Swagger
- GIVEN el servidor arrancado
- WHEN se solicita `/api/docs`
- THEN se sirve la interfaz Swagger UI
