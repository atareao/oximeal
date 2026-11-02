/**
 * Utilidades de fecha en hora local.
 *
 * Todas las fechas "de calendario" se representan como `YYYY-MM-DD` y se
 * construyen/parsean manualmente para evitar el desfase de zona que provoca
 * `Date.prototype.toISOString()` sobre fechas locales.
 */

const MONTHS_SHORT = [
  'ene',
  'feb',
  'mar',
  'abr',
  'may',
  'jun',
  'jul',
  'ago',
  'sep',
  'oct',
  'nov',
  'dic',
] as const;

const DAYS_SHORT = ['Dom', 'Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb'] as const;

/** Convierte un `Date` local a `YYYY-MM-DD` sin pasar por UTC. */
export function toDateString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/** Parsea `YYYY-MM-DD` como medianoche local. */
export function parseDateString(dateStr: string): Date {
  const [year, month, day] = dateStr.split('-').map(Number);
  return new Date(year, month - 1, day);
}

const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

/**
 * ¿Es `value` una fecha `YYYY-MM-DD` real?
 *
 * Hace un round-trip con `Date` para descartar fechas imposibles como
 * `2026-13-45` o `2026-02-29` (año no bisiesto), que `Date` normalizaría
 * silenciosamente a otro día.
 */
export function isValidDateString(value: string): boolean {
  if (!DATE_PATTERN.test(value)) {
    return false;
  }
  const [year, month, day] = value.split('-').map(Number);
  const date = new Date(year, month - 1, day);
  return date.getFullYear() === year && date.getMonth() === month - 1 && date.getDate() === day;
}

/** Lunes de la semana ISO (lunes–domingo) que contiene `date`, como `YYYY-MM-DD`. */
export function startOfWeek(date: Date | string): string {
  const base = typeof date === 'string' ? parseDateString(date) : new Date(date);
  base.setHours(0, 0, 0, 0);
  // getDay(): 0 = domingo … 6 = sábado. Desplazamos para que lunes = 0.
  const offset = (base.getDay() + 6) % 7;
  base.setDate(base.getDate() - offset);
  return toDateString(base);
}

/** Suma (o resta, con `n` negativo) días a una fecha `YYYY-MM-DD`. */
export function addDays(dateStr: string, n: number): string {
  const date = parseDateString(dateStr);
  date.setDate(date.getDate() + n);
  return toDateString(date);
}

/** Los siete días (lunes→domingo) de la semana que empieza en `startDate`. */
export function getWeekDays(startDate: string): string[] {
  return Array.from({ length: 7 }, (_, index) => addDays(startDate, index));
}

/** Etiqueta corta de día, p. ej. `Lun 3`. */
export function formatDayLabel(dateStr: string): string {
  const date = parseDateString(dateStr);
  return `${DAYS_SHORT[date.getDay()]} ${date.getDate()}`;
}

/** Rango legible de semana, p. ej. `3 – 9 nov 2026`. */
export function formatWeekRange(start: string, end: string): string {
  const startDate = parseDateString(start);
  const endDate = parseDateString(end);

  const startDay = startDate.getDate();
  const endDay = endDate.getDate();
  const startMonth = MONTHS_SHORT[startDate.getMonth()];
  const endMonth = MONTHS_SHORT[endDate.getMonth()];
  const startYear = startDate.getFullYear();
  const endYear = endDate.getFullYear();

  if (startYear !== endYear) {
    return `${startDay} ${startMonth} ${startYear} – ${endDay} ${endMonth} ${endYear}`;
  }
  if (startDate.getMonth() !== endDate.getMonth()) {
    return `${startDay} ${startMonth} – ${endDay} ${endMonth} ${endYear}`;
  }
  return `${startDay} – ${endDay} ${endMonth} ${endYear}`;
}

/** ¿La fecha `YYYY-MM-DD` es el día de hoy en hora local? */
export function isToday(dateStr: string): boolean {
  return dateStr === toDateString(new Date());
}
