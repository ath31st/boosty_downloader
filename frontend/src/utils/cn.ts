/**
 * Объединяет классы CSS, отфильтровывая пустые значения
 */
export function cn(...classes: Array<string | undefined | null | false>): string {
  return classes.filter(Boolean).join(' ');
}
