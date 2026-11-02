import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { ApiError, apiRequest } from './client';

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

    await expect(apiRequest('/shopping-list/completed', { method: 'DELETE' })).resolves.toBeUndefined();
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
});
