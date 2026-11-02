import type {
  ApiErrorBody,
  AuthMeResponse,
  MealPlanEntryView,
  Recipe,
  RecipeImportInput,
  RecipeImportResult,
  RecipeInput,
  RecipeListResponse,
  RecipeQueryParams,
  ShoppingItem,
  ShoppingItemInput,
  ShoppingItemPatch,
  ShoppingListGroup,
} from './types';

const API_BASE = '/api';
const REQUEST_TIMEOUT_MS = 10_000;
const CSRF_HEADER = 'X-CSRF-Token';
const MUTATION_METHODS: ReadonlySet<NonNullable<RequestOptions['method']>> = new Set([
  'POST',
  'PUT',
  'PATCH',
  'DELETE',
]);

/** Error tipado de la API, con el sobre `{error,message}` del backend. */
export class ApiError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
  }
}

export interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  body?: unknown;
  signal?: AbortSignal;
}

/**
 * Token anti-CSRF a nivel de módulo. `SessionProvider` lo rellena tras
 * `GET /auth/me`; el cliente lo adjunta a toda mutación autenticada por cookie.
 */
let csrfToken: string | null = null;

export function setCsrfToken(token: string | null): void {
  csrfToken = token;
}

type UnauthorizedHandler = () => void;

/**
 * Handler global invocado cuando cualquier petición responde 401.
 * Lo registra `SessionProvider` para marcar la sesión como anónima y volver
 * al login.
 */
let unauthorizedHandler: UnauthorizedHandler | null = null;

export function setUnauthorizedHandler(handler: UnauthorizedHandler | null): void {
  unauthorizedHandler = handler;
}

type ForbiddenHandler = () => void;

/**
 * Handler global invocado cuando cualquier petición responde 403.
 *
 * Lo registra un componente montado **dentro** de `<AntdApp>` (p. ej.
 * `ForbiddenNotifier`) que usa `App.useApp().message`, de forma que el aviso
 * comparta el contexto de tema/configuración de la aplicación. Si no hay
 * handler registrado, el fallback carga antd de forma **dinámica**
 * (`import('antd')`) para no arrastrarlo al grafo estático del arranque.
 */
let forbiddenHandler: ForbiddenHandler | null = null;

export function setForbiddenHandler(handler: ForbiddenHandler | null): void {
  forbiddenHandler = handler;
}

function isApiErrorBody(value: unknown): value is ApiErrorBody {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as Record<string, unknown>).error === 'string' &&
    typeof (value as Record<string, unknown>).message === 'string'
  );
}

function isAbortError(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    (error as { name?: unknown }).name === 'AbortError'
  );
}

/**
 * Realiza una petición HTTP y devuelve el JSON tipado junto con el código de
 * estado de la respuesta.
 *
 * - Adjunta `credentials: 'same-origin'` (cookies de sesión same-origin).
 * - Añade `X-CSRF-Token` en POST/PATCH/DELETE cuando hay token.
 * - En 401 invoca el handler global de sesión; en 403 muestra un aviso de
 *   permisos insuficientes sin destruir la sesión.
 * - Lanza {@link ApiError} con el mensaje del backend si la respuesta no es 2xx.
 * - Los fallos de red se normalizan a `ApiError(0, 'network_error', …)`; las
 *   cancelaciones (`AbortError`) se propagan sin envolver.
 */
async function performRequestWithStatus<T>(
  url: string,
  options: RequestOptions = {},
): Promise<{ data: T | undefined; status: number }> {
  const { method = 'GET', body, signal } = options;

  const headers: Record<string, string> = { Accept: 'application/json' };
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }
  if (MUTATION_METHODS.has(method) && csrfToken) {
    headers[CSRF_HEADER] = csrfToken;
  }

  const timeoutSignal = AbortSignal.timeout(REQUEST_TIMEOUT_MS);
  const combinedSignal = signal ? AbortSignal.any([signal, timeoutSignal]) : timeoutSignal;

  let response: Response;
  try {
    response = await fetch(url, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: combinedSignal,
      credentials: 'same-origin',
    });
  } catch (error) {
    if (isAbortError(error)) {
      throw error;
    }
    throw new ApiError(0, 'network_error', 'No se pudo contactar con el servidor');
  }

  if (!response.ok) {
    let code = 'http_error';
    let messageText = `Error ${response.status} al contactar con la API`;

    try {
      const payload: unknown = await response.json();
      if (isApiErrorBody(payload)) {
        code = payload.error;
        messageText = payload.message;
      }
    } catch {
      // Cuerpo no JSON: se conserva el mensaje genérico.
    }

    if (response.status === 401) {
      unauthorizedHandler?.();
    } else if (response.status === 403) {
      if (forbiddenHandler) {
        forbiddenHandler();
      } else {
        // Carga diferida: mantiene antd fuera del grafo estático del entry.
        void import('antd')
          .then(({ message }) => message.warning('Permisos insuficientes'))
          .catch(() => {
            // antd no disponible (p. ej. chunk obsoleto): el 403 ya se lanzó como ApiError.
          });
      }
    }

    throw new ApiError(response.status, code, messageText);
  }

  if (response.status === 204) {
    return { data: undefined, status: response.status };
  }

  return { data: (await response.json()) as T, status: response.status };
}

async function performRequest<T>(
  url: string,
  options: RequestOptions = {},
): Promise<T | undefined> {
  const { data } = await performRequestWithStatus<T>(url, options);
  return data;
}

export function apiRequest(
  path: string,
  options: RequestOptions & { method: 'DELETE' },
): Promise<void>;
export function apiRequest<T>(path: string, options?: RequestOptions): Promise<T>;
export async function apiRequest<T>(
  path: string,
  options: RequestOptions = {},
): Promise<T | undefined> {
  return performRequest<T>(`${API_BASE}${path}`, options);
}

/** `GET /auth/me`: estado de sesión del BFF (fuera de `/api`). */
export function fetchAuthMe(signal?: AbortSignal): Promise<AuthMeResponse> {
  return performRequest<AuthMeResponse>('/auth/me', { signal }) as Promise<AuthMeResponse>;
}

/** `POST /auth/logout`: destruye la sesión server-side y limpia la cookie. */
export function logoutAuth(): Promise<void> {
  return performRequest<void>('/auth/logout', { method: 'POST' }) as Promise<void>;
}

function buildQuery(params: Record<string, string | number | undefined>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== '') {
      search.set(key, String(value));
    }
  }
  const query = search.toString();
  return query ? `?${query}` : '';
}

export function fetchRecipes(
  params: RecipeQueryParams = {},
  signal?: AbortSignal,
): Promise<RecipeListResponse> {
  const query = buildQuery({
    q: params.q,
    category: params.category,
    page: params.page,
    page_size: params.page_size,
  });
  return apiRequest<RecipeListResponse>(`/recipes${query}`, { signal });
}

export function fetchRecipe(id: string, signal?: AbortSignal): Promise<Recipe> {
  return apiRequest<Recipe>(`/recipes/${encodeURIComponent(id)}`, { signal });
}

/** `POST /api/recipes` — alta de receta. */
export function createRecipe(input: RecipeInput): Promise<Recipe> {
  return apiRequest<Recipe>('/recipes', { method: 'POST', body: input });
}

/**
 * `POST /api/recipes/import` — importa una receta desde una URL (Schema.org).
 *
 * Devuelve la receta creada y si fue un alta (`201`) o una importación
 * idempotente de una `source_url` ya existente (`200`).
 */
export async function importRecipe(input: RecipeImportInput): Promise<RecipeImportResult> {
  const { data, status } = await performRequestWithStatus<Recipe>('/api/recipes/import', {
    method: 'POST',
    body: input,
  });
  if (!data) {
    throw new ApiError(status, 'invalid_response', 'Respuesta vacía al importar la receta');
  }
  return { recipe: data, created: status === 201 };
}

/** `PUT /api/recipes/{id}` — reemplazo completo de una receta. */
export function updateRecipe(id: string, input: RecipeInput): Promise<Recipe> {
  return apiRequest<Recipe>(`/recipes/${encodeURIComponent(id)}`, { method: 'PUT', body: input });
}

/** `DELETE /api/recipes/{id}` — borrado de una receta (204 sin cuerpo). */
export function deleteRecipe(id: string): Promise<void> {
  return apiRequest(`/recipes/${encodeURIComponent(id)}`, { method: 'DELETE' });
}

export function fetchMealPlan(
  startDate: string,
  endDate: string,
  signal?: AbortSignal,
): Promise<MealPlanEntryView[]> {
  const query = buildQuery({ start_date: startDate, end_date: endDate });
  return apiRequest<MealPlanEntryView[]>(`/meal-plan${query}`, { signal });
}

export function fetchShoppingList(signal?: AbortSignal): Promise<ShoppingListGroup[]> {
  return apiRequest<ShoppingListGroup[]>('/shopping-list', { signal });
}

export function patchShoppingItem(id: string, patch: ShoppingItemPatch): Promise<ShoppingItem> {
  return apiRequest<ShoppingItem>(`/shopping-list/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: patch,
  });
}

export function createShoppingItem(input: ShoppingItemInput): Promise<ShoppingItem> {
  return apiRequest<ShoppingItem>('/shopping-list', { method: 'POST', body: input });
}

export function consolidateShoppingList(): Promise<ShoppingListGroup[]> {
  return apiRequest<ShoppingListGroup[]>('/shopping-list/consolidate', { method: 'POST' });
}

export function clearCompletedShoppingItems(): Promise<void> {
  return apiRequest('/shopping-list/completed', { method: 'DELETE' });
}
