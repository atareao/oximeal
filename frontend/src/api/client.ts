import type {
  ApiErrorBody,
  MealPlanEntryView,
  Recipe,
  RecipeListResponse,
  RecipeQueryParams,
  ShoppingItem,
  ShoppingItemInput,
  ShoppingItemPatch,
  ShoppingListGroup,
} from './types';

const API_BASE = '/api';
const REQUEST_TIMEOUT_MS = 10_000;

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
  method?: 'GET' | 'POST' | 'PATCH' | 'DELETE';
  body?: unknown;
  signal?: AbortSignal;
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
 * Realiza una petición a la API y devuelve el JSON tipado.
 * Lanza {@link ApiError} con el mensaje del backend si la respuesta no es 2xx.
 * Los fallos de red se normalizan a `ApiError(0, 'network_error', …)`; las
 * cancelaciones (`AbortError`) se propagan sin envolver.
 */
export function apiRequest(
  path: string,
  options: RequestOptions & { method: 'DELETE' },
): Promise<void>;
export function apiRequest<T>(path: string, options?: RequestOptions): Promise<T>;
export async function apiRequest<T>(
  path: string,
  options: RequestOptions = {},
): Promise<T | undefined> {
  const { method = 'GET', body, signal } = options;

  const headers: Record<string, string> = { Accept: 'application/json' };
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }

  const timeoutSignal = AbortSignal.timeout(REQUEST_TIMEOUT_MS);
  const combinedSignal = signal ? AbortSignal.any([signal, timeoutSignal]) : timeoutSignal;

  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: combinedSignal,
    });
  } catch (error) {
    if (isAbortError(error)) {
      throw error;
    }
    throw new ApiError(0, 'network_error', 'No se pudo contactar con el servidor');
  }

  if (!response.ok) {
    let code = 'http_error';
    let message = `Error ${response.status} al contactar con la API`;

    try {
      const payload: unknown = await response.json();
      if (isApiErrorBody(payload)) {
        code = payload.error;
        message = payload.message;
      }
    } catch {
      // Cuerpo no JSON: se conserva el mensaje genérico.
    }

    throw new ApiError(response.status, code, message);
  }

  if (response.status === 204) {
    return undefined;
  }

  return (await response.json()) as T;
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
