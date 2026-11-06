import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// La paleta oscura vive duplicada a propósito en CSS (variables) y en los
// tokens de Ant Design (theme.ts). Este test es la red de seguridad que evita
// derivas y tintes azulados. No fija hex concretos salvo la neutralidad.

// `baseUrl` evita que Vite reescriba `new URL(...)` como activo del bundle.
const baseUrl = import.meta.url;
const indexCss = readFileSync(new URL('../index.css', baseUrl), 'utf8');
const themeSource = readFileSync(new URL('../components/theme.ts', baseUrl), 'utf8');
const themeProviderSource = readFileSync(new URL('./ThemeProvider.tsx', baseUrl), 'utf8');
const indexHtml = readFileSync(new URL('../../index.html', baseUrl), 'utf8');
const manifest = JSON.parse(
  readFileSync(new URL('../../public/manifest.webmanifest', baseUrl), 'utf8'),
) as { theme_color: string };

const darkCssBlock = (() => {
  const match = indexCss.match(/:root\[data-theme='dark'\]\s*\{([\s\S]*?)\}/);
  if (!match) {
    throw new Error("No se encontró el bloque :root[data-theme='dark'] en index.css");
  }
  return match[1];
})();

const darkThemeBlock = (() => {
  const match = themeSource.match(/export const darkTheme[\s\S]*?=\s*\{([\s\S]*?)\n\};/);
  if (!match) {
    throw new Error('No se encontró darkTheme en components/theme.ts');
  }
  return match[1];
})();

function cssVar(block: string, name: string): string {
  const match = block.match(new RegExp(`--${name}:\\s*(#[0-9a-fA-F]{3,8})`));
  if (!match) {
    throw new Error(`Falta la variable --${name} en el bloque oscuro`);
  }
  return match[1];
}

function tokenHex(block: string, prop: string): string {
  const match = block.match(new RegExp(`${prop}:\\s*'(#[0-9a-fA-F]{3,8})'`));
  if (!match) {
    throw new Error(`Falta el token ${prop} en darkTheme`);
  }
  return match[1];
}

/** Extrae `THEME_COLORS.dark` de ThemeProvider sin importar el módulo (evita React/JSX). */
const themeColorsDark = (() => {
  const match = themeProviderSource.match(/THEME_COLORS[\s\S]*?dark:\s*'(#[0-9a-fA-F]{3,8})'/);
  if (!match) {
    throw new Error('No se encontró THEME_COLORS.dark en ThemeProvider.tsx');
  }
  return match[1];
})();

function toRgb(hex: string): [number, number, number] {
  const clean = hex.replace('#', '');
  const full =
    clean.length === 3
      ? clean
          .split('')
          .map((c) => c + c)
          .join('')
      : clean;
  return [
    parseInt(full.slice(0, 2), 16),
    parseInt(full.slice(2, 4), 16),
    parseInt(full.slice(4, 6), 16),
  ];
}

function relativeLuminance(hex: string): number {
  const [r, g, b] = toRgb(hex).map((channel) => {
    const c = channel / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrastRatio(foreground: string, background: string): number {
  const a = relativeLuminance(foreground);
  const b = relativeLuminance(background);
  const lighter = Math.max(a, b);
  const darker = Math.min(a, b);
  return (lighter + 0.05) / (darker + 0.05);
}

/** Un fondo es neutro si ningún canal azul excede de forma perceptible a R y G. */
function expectNeutralBackground(hex: string, label: string): void {
  const [r, g, b] = toRgb(hex);
  const dominant = Math.max(r, g);
  expect(
    b - dominant,
    `${label} (${hex}) tiene dominante azul: B=${b}, max(R,G)=${dominant}`,
  ).toBeLessThanOrEqual(4);
}

const darkPalette = {
  bg: cssVar(darkCssBlock, 'bg'),
  surface: cssVar(darkCssBlock, 'surface'),
  surfaceMuted: cssVar(darkCssBlock, 'surface-muted'),
  text: cssVar(darkCssBlock, 'text'),
  textMuted: cssVar(darkCssBlock, 'text-muted'),
  accent: cssVar(darkCssBlock, 'accent'),
  accentSoft: cssVar(darkCssBlock, 'accent-soft'),
};

describe('paleta oscura neutra (index.css)', () => {
  it('los fondos no tienen dominante azulada', () => {
    expectNeutralBackground(darkPalette.bg, '--bg');
    expectNeutralBackground(darkPalette.surface, '--surface');
    expectNeutralBackground(darkPalette.surfaceMuted, '--surface-muted');
    expectNeutralBackground(darkPalette.accentSoft, '--accent-soft');
  });

  it('el texto y su contraste alcanzan AA (>= 4.5:1)', () => {
    const combos: Array<[string, string, string]> = [
      ['--text', darkPalette.text, darkPalette.bg],
      ['--text', darkPalette.text, darkPalette.surface],
      ['--text', darkPalette.text, darkPalette.surfaceMuted],
      ['--text-muted', darkPalette.textMuted, darkPalette.bg],
      ['--text-muted', darkPalette.textMuted, darkPalette.surface],
      ['--text-muted', darkPalette.textMuted, darkPalette.surfaceMuted],
      ['--text sobre --accent-soft', darkPalette.text, darkPalette.accentSoft],
      ['--accent sobre --bg', darkPalette.accent, darkPalette.bg],
    ];
    for (const [label, fg, bg] of combos) {
      expect(contrastRatio(fg, bg), `${label}: ${fg} sobre ${bg}`).toBeGreaterThanOrEqual(4.5);
    }
  });
});

describe('darkTheme de Ant Design (components/theme.ts)', () => {
  it('los tokens de fondo no tienen dominante azulada', () => {
    expectNeutralBackground(tokenHex(darkThemeBlock, 'colorBgBase'), 'colorBgBase');
    expectNeutralBackground(tokenHex(darkThemeBlock, 'bodyBg'), 'Layout.bodyBg');
    expectNeutralBackground(tokenHex(darkThemeBlock, 'headerBg'), 'Layout.headerBg');
    expectNeutralBackground(tokenHex(darkThemeBlock, 'siderBg'), 'Layout.siderBg');
  });

  it('los tokens de fondo coinciden con las variables CSS neutras', () => {
    expect(tokenHex(darkThemeBlock, 'colorBgBase')).toBe(darkPalette.bg);
    expect(tokenHex(darkThemeBlock, 'bodyBg')).toBe(darkPalette.bg);
    expect(tokenHex(darkThemeBlock, 'headerBg')).toBe(darkPalette.surface);
    expect(tokenHex(darkThemeBlock, 'siderBg')).toBe(darkPalette.surface);
    expect(tokenHex(darkThemeBlock, 'colorTextBase')).toBe(darkPalette.text);
  });
});

describe('coherencia de theme-color (ThemeProvider / index.html / manifest)', () => {
  it('THEME_COLORS.dark coincide con el meta theme-color de index.html', () => {
    const match = indexHtml.match(/<meta name="theme-color" content="([^"]+)"/);
    expect(match?.[1]).toBe(themeColorsDark);
  });

  it('THEME_COLORS.dark coincide con theme_color del manifest', () => {
    expect(manifest.theme_color).toBe(themeColorsDark);
  });

  it('el meta theme-color y theme_color del manifest coinciden entre sí', () => {
    const match = indexHtml.match(/<meta name="theme-color" content="([^"]+)"/);
    expect(match?.[1]).toBe(manifest.theme_color);
  });
});
