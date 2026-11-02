/**
 * Configuración de la aplicación derivada de variables de entorno de Vite.
 *
 * `VITE_VALET_URL` es opcional: si no está definida (o queda vacía tras
 * recortar espacios) `hasValet` es `false` y la UI oculta las acciones que
 * dependen del asistente.
 */
const rawValetUrl = import.meta.env.VITE_VALET_URL ?? '';

export const VALET_URL: string = rawValetUrl.trim();

export const hasValet: boolean = VALET_URL.length > 0;
