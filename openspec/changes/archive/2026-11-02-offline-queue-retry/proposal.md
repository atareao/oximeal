# Change: reintento del drenado de la cola de escrituras offline

## Why

La verificación final de la Fase 5 con un smoke E2E real (Chrome sobre el binario
con la UI embebida) destapó que **la cola no se drena sola al recuperar la
conexión**: tras reconectar se midieron 0 `POST /api/shopping-list` y el aviso de
cambios pendientes seguía visible 20 s después (también tras recargar con
conexión). Pulsando «Sincronizar ahora» el drenado funciona y no duplica ítems,
así que el drenado está bien: lo que falta es **el disparo**.

`MutationQueueProvider` solo intenta drenar al montar, cuando cambia el hook
`useOnline()` y cuando cambia la cola. Si un drenado termina con
`stoppedBy: 'network'` — red caída mientras `navigator.onLine` seguía diciendo
`true` (wifi cautivo, servidor caído, red que vuelve sin evento `online` fiable,
pestaña que despierta) — **nadie reintenta** y la cola queda atascada
indefinidamente con el aviso de pendientes delante del usuario.

## What Changes

- Nuevo módulo `frontend/src/offline/drainScheduler.ts`: política de reintento
  aislada del provider y testeable con temporizadores falsos.
  - Reintento con **backoff exponencial acotado** (2 s, 4 s, 8 s, 16 s, 30 s
    máximo) mientras el drenado se detenga por red.
  - Reprogramación **inmediata** si el drenado progresó y aún quedan pendientes.
  - Sin reintento cuando la cola queda vacía (y el backoff se reinicia).
  - Sin reintento en bucle ante un 401/403: se espera a que cambien las
    condiciones (evento `online`, `visibilitychange`, cambio de cola), que es
    cuando vuelve a intentarse.
  - `setTimeout` reprogramado, nunca `setInterval`; cancelación en `dispose()`.
- `MutationQueueProvider` usa el planificador y añade reintento al recuperar el
  primer plano (`visibilitychange`) y el foco de la ventana.
- La reentrancia actual se mantiene: un solo drenado en vuelo.

## Impact

- Capabilities: nuevo requisito en `offline-writes`.
- Frontend: `frontend/src/offline/drainScheduler.ts` (nuevo),
  `frontend/src/offline/MutationQueueProvider.tsx`, tests.
- Sin cambios de contrato con el backend y sin dependencias nuevas.
- Cierra el issue #35.
