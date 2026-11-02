import { existsSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

interface ManifestIcon {
  src: string;
  sizes: string;
  type: string;
  purpose?: string;
}

interface WebManifest {
  name: string;
  short_name: string;
  start_url: string;
  scope: string;
  id: string;
  display: string;
  orientation: string;
  theme_color: string;
  background_color: string;
  lang: string;
  icons: ManifestIcon[];
}

// La base se guarda en una variable para que Vite no reescriba `new URL(...)`
// como un activo del bundle: aquí necesitamos la ruta real en disco.
const baseUrl = import.meta.url;
const manifest = JSON.parse(
  readFileSync(new URL('../../public/manifest.webmanifest', baseUrl), 'utf8'),
) as WebManifest;

const indexHtml = readFileSync(new URL('../../index.html', baseUrl), 'utf8');

/** Lee ancho y alto de un PNG desde la cabecera IHDR (big-endian). */
function pngSize(url: URL): { width: number; height: number } {
  const buffer = readFileSync(url);
  return { width: buffer.readUInt32BE(16), height: buffer.readUInt32BE(20) };
}

describe('manifest.webmanifest', () => {
  it('es JSON válido con los campos obligatorios', () => {
    expect(manifest.name).toBeTruthy();
    expect(manifest.short_name).toBeTruthy();
    expect(manifest.start_url).toBe('/');
    expect(manifest.scope).toBe('/');
    expect(manifest.id).toBe('/');
    expect(manifest.display).toBe('standalone');
    expect(manifest.orientation).toBe('any');
    expect(manifest.theme_color).toBeTruthy();
    expect(manifest.background_color).toBeTruthy();
    expect(manifest.lang).toBe('es');
  });

  it('declara los iconos 192, 512 y una variante maskable', () => {
    const sizes = manifest.icons.map((icon) => icon.sizes);
    expect(sizes).toContain('192x192');
    expect(sizes).toContain('512x512');
    expect(manifest.icons.some((icon) => icon.purpose === 'maskable')).toBe(true);
  });

  it('cada icono existe en disco con las dimensiones declaradas', () => {
    for (const icon of manifest.icons) {
      const url = new URL(`../../public${icon.src}`, baseUrl);
      expect(existsSync(url), `falta ${icon.src}`).toBe(true);

      const [width, height] = icon.sizes.split('x').map(Number);
      expect(pngSize(url)).toEqual({ width, height });
    }
  });

  it('el theme_color coincide con el meta theme-color de index.html', () => {
    const match = indexHtml.match(/<meta name="theme-color" content="([^"]+)"/);
    expect(match?.[1]).toBe(manifest.theme_color);
  });
});
