import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { message } from 'antd';
import {
  ApiError,
  apiRequest,
  clearCompletedShoppingItems,
  createRecipe,
  createShoppingItem,
  deleteRecipe,
  fetchAuthMe,
  importRecipe,
  logoutAuth,
  patchShoppingItem,
  setCsrfToken,
  updateRecipe,
  setForbiddenHandler,
  setUnauthorizedHandler,
} from './client';

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

describe('apiRequest', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('devuelve el JSON en una respuesta correcta', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ items: [], total: 0 }));

    const result = await apiRequest<{ items: unknown[]; total: number }>('/recipes');

    expect(result).toEqual({ items: [], total: 0 });
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/recipes',
      expect.objectContaining({ method: 'GET' }),
    );
  });

  it('parsea el sobre {error,message} y lanza ApiError con el mensaje', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(
      jsonResponse({ error: 'not_found', message: 'recurso no encontrado: receta' }, 404),
    );

    await expect(apiRequest('/recipes/desconocida')).rejects.toMatchObject({
      name: 'ApiError',
      status: 404,
      code: 'not_found',
      message: 'recurso no encontrado: receta',
    });
  });

  it('lanza ApiError con mensaje genérico si el cuerpo de error no es JSON', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(new Response('boom', { status: 500 }));

    const error = await apiRequest('/recipes').catch((caught: unknown) => caught);

    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(500);
    expect((error as ApiError).message).toContain('500');
  });

  it('devuelve undefined en respuestas 204', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }));

    await expect(
      apiRequest('/shopping-list/completed', { method: 'DELETE' }),
    ).resolves.toBeUndefined();
  });

  it('resuelve undefined con la sobrecarga apiRequest<void> en un DELETE 204', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }));

    await expect(
      apiRequest<void>('/shopping-list/completed', { method: 'DELETE' }),
    ).resolves.toBeUndefined();

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/shopping-list/completed',
      expect.objectContaining({ method: 'DELETE' }),
    );
  });

  it('normaliza los fallos de red a ApiError(0, network_error)', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'));

    const error = await apiRequest('/recipes').catch((caught: unknown) => caught);

    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(0);
    expect((error as ApiError).code).toBe('network_error');
    expect((error as ApiError).message).toBe('No se pudo contactar con el servidor');
  });

  it('propaga AbortError sin envolverlo', async () => {
    const fetchMock = vi.mocked(fetch);
    const abortError = new DOMException('abortado', 'AbortError');
    fetchMock.mockRejectedValueOnce(abortError);

    const error = await apiRequest('/recipes').catch((caught: unknown) => caught);

    expect(error).toBe(abortError);
    expect(error).not.toBeInstanceOf(ApiError);
  });

  it('serializa el cuerpo y añade Content-Type en peticiones con body', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ id: '1' }, 201));

    await apiRequest('/shopping-list', { method: 'POST', body: { name: 'Pan' } });

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/shopping-list',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ name: 'Pan' }),
        headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
      }),
    );
  });

  it('envía las credenciales same-origin en cada petición', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ items: [], total: 0 }));

    await apiRequest('/recipes');

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/recipes',
      expect.objectContaining({ credentials: 'same-origin' }),
    );
  });
});

describe('CSRF y manejo de sesión en el cliente', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
    setCsrfToken(null);
    setUnauthorizedHandler(null);
    setForbiddenHandler(null);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    setCsrfToken(null);
    setUnauthorizedHandler(null);
    setForbiddenHandler(null);
  });

  const csrfCases: ReadonlyArray<{
    label: string;
    method: string;
    url: string;
    response: () => Response;
    call: () => Promise<unknown>;
  }> = [
    {
      label: 'POST',
      method: 'POST',
      url: '/api/shopping-list',
      response: () => jsonResponse({ id: '1' }, 201),
      call: () => createShoppingItem({ name: 'Pan', amount: 1, unit: 'ud' }),
    },
    {
      label: 'PATCH',
      method: 'PATCH',
      url: '/api/shopping-list/item-1',
      response: () => jsonResponse({ id: 'item-1', checked: true }, 200),
      call: () => patchShoppingItem('item-1', { checked: true }),
    },
    {
      label: 'DELETE',
      method: 'DELETE',
      url: '/api/shopping-list/completed',
      response: () => new Response(null, { status: 204 }),
      call: () => clearCompletedShoppingItems(),
    },
  ];

  it.each(csrfCases)(
    'añade X-CSRF-Token a las mutaciones $label cuando hay token',
    async ({ method, url, response, call }) => {
      const fetchMock = vi.mocked(fetch);
      fetchMock.mockResolvedValueOnce(response());

      setCsrfToken('token-123');
      await call();

      expect(fetchMock).toHaveBeenCalledWith(
        url,
        expect.objectContaining({
          method,
          headers: expect.objectContaining({ 'X-CSRF-Token': 'token-123' }),
        }),
      );
    },
  );

  it('no añade X-CSRF-Token a las peticiones GET', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ items: [], total: 0 }));

    setCsrfToken('token-123');
    await apiRequest('/recipes');

    const [, init] = fetchMock.mock.calls[0];
    expect(init?.headers).not.toHaveProperty('X-CSRF-Token');
  });

  it('notifica al handler global cuando una petición devuelve 401', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(
      jsonResponse({ error: 'unauthorized', message: 'sesión caducada' }, 401),
    );
    const handler = vi.fn();
    setUnauthorizedHandler(handler);

    await expect(apiRequest('/recipes')).rejects.toBeInstanceOf(ApiError);

    expect(handler).toHaveBeenCalledTimes(1);
  });

  it('en 403 usa el aviso de antd cargado dinámicamente cuando no hay handler inyectado', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(
      jsonResponse({ error: 'forbidden', message: 'permiso denegado' }, 403),
    );
    const warning = vi.spyOn(message, 'warning').mockImplementation((() => {}) as never);
    const handler = vi.fn();
    setUnauthorizedHandler(handler);

    await expect(apiRequest('/recipes')).rejects.toMatchObject({ status: 403 });

    await vi.waitFor(() => expect(warning).toHaveBeenCalledTimes(1));
    expect(handler).not.toHaveBeenCalled();
  });

  it('en 403 delega en el handler inyectado y no usa el aviso estático', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(
      jsonResponse({ error: 'forbidden', message: 'permiso denegado' }, 403),
    );
    const warning = vi.spyOn(message, 'warning').mockImplementation((() => {}) as never);
    const forbidden = vi.fn();
    const unauthorized = vi.fn();
    setForbiddenHandler(forbidden);
    setUnauthorizedHandler(unauthorized);

    await expect(apiRequest('/recipes')).rejects.toMatchObject({ status: 403 });

    expect(forbidden).toHaveBeenCalledTimes(1);
    expect(warning).not.toHaveBeenCalled();
    expect(unauthorized).not.toHaveBeenCalled();
  });

  it('obtiene /auth/me desde la raíz, no bajo /api', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ authenticated: true, name: 'Ana' }));

    const me = await fetchAuthMe();

    expect(me).toEqual({ authenticated: true, name: 'Ana' });
    expect(fetchMock).toHaveBeenCalledWith('/auth/me', expect.objectContaining({ method: 'GET' }));
  });

  it('invoca POST /auth/logout y devuelve la URL de fin de sesión', async () => {
    const fetchMock = vi.mocked(fetch);
    const logoutUrl = 'https://idp.example/logout?id_token_hint=abc&client_id=oximeal';
    fetchMock.mockResolvedValueOnce(jsonResponse({ logout_url: logoutUrl }));
    setCsrfToken('token-123');

    await expect(logoutAuth()).resolves.toEqual({ logout_url: logoutUrl });

    expect(fetchMock).toHaveBeenCalledWith(
      '/auth/logout',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({ 'X-CSRF-Token': 'token-123' }),
      }),
    );
  });

  it('devuelve logout_url null cuando el backend no puede construir la URL', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ logout_url: null }));

    await expect(logoutAuth()).resolves.toEqual({ logout_url: null });
  });

  it('normaliza a logout_url null una respuesta sin el campo esperado', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({}));

    await expect(logoutAuth()).resolves.toEqual({ logout_url: null });
  });
});

describe('mutaciones de recetas', () => {
  const input = {
    name: 'Tortilla',
    category: 'main',
    ingredients: [{ name: 'Patata', amount: 500, unit: 'g' }],
    tags: [],
  };

  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
    setCsrfToken(null);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    setCsrfToken(null);
  });

  it('createRecipe hace POST /api/recipes', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ id: 'r1', ...input }, 201));

    await createRecipe(input);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/recipes',
      expect.objectContaining({ method: 'POST', body: JSON.stringify(input) }),
    );
  });

  it('updateRecipe hace PUT /api/recipes/{id} con CSRF', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ id: 'r1', ...input }));
    setCsrfToken('token-123');

    await updateRecipe('r1', input);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/recipes/r1',
      expect.objectContaining({
        method: 'PUT',
        body: JSON.stringify(input),
        headers: expect.objectContaining({ 'X-CSRF-Token': 'token-123' }),
      }),
    );
  });

  it('importRecipe hace POST /api/recipes/import con el cuerpo y CSRF', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(
      jsonResponse({ id: 'r1', name: 'Tortilla', source_url: 'https://example.com/r' }, 201),
    );
    setCsrfToken('token-123');

    const result = await importRecipe({ url: 'https://example.com/r' });

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/recipes/import',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ url: 'https://example.com/r' }),
        headers: expect.objectContaining({ 'X-CSRF-Token': 'token-123' }),
      }),
    );
    expect(result.recipe).toMatchObject({ id: 'r1', source_url: 'https://example.com/r' });
    expect(result.created).toBe(true);
  });

  it('importRecipe marca created=false cuando el origen responde 200 (idempotente)', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(jsonResponse({ id: 'r1', name: 'Tortilla' }, 200));

    const result = await importRecipe({ url: 'https://example.com/r' });

    expect(result.created).toBe(false);
    expect(result.recipe.id).toBe('r1');
  });

  it('importRecipe propaga el ApiError del backend (422 sin receta)', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(
      jsonResponse(
        { error: 'unprocessable_entity', message: 'la página no contiene una receta' },
        422,
      ),
    );

    await expect(importRecipe({ url: 'https://example.com/r' })).rejects.toMatchObject({
      name: 'ApiError',
      status: 422,
      message: 'la página no contiene una receta',
    });
  });

  it('deleteRecipe hace DELETE /api/recipes/{id} y resuelve undefined en 204', async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }));
    setCsrfToken('token-123');

    await expect(deleteRecipe('r1')).resolves.toBeUndefined();

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/recipes/r1',
      expect.objectContaining({
        method: 'DELETE',
        headers: expect.objectContaining({ 'X-CSRF-Token': 'token-123' }),
      }),
    );
  });
});
