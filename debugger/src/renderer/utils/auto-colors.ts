/**
 * Utilities for automatic color-coding of concepts
 *
 * @module
 */

/**
 * Constructs a deterministic color that can be used to identify a given string.
 *
 * @param s String that is to be color-coded.
 * @returns CSS color specification corresponding to `s`.
 */
export function colorFromString(s: string): string {
    // Hash the string to ensure enough variety in generated colors
    // This is going to crash on strings that are shorter than 4 characters,
    // let us assume we do not need that many of those
    const hash = Array.from(s).reduce((h: number, c: string) => {
        // Most characters we are likely to encounter here are at least 7 bits long
        // Some may be 6
        // Shorter characters are not a concern here
        const STRIDE = 6;
        return (h << STRIDE) ^ (h + (c.codePointAt(0) ?? 0));
    }, 0);
    return colorFromNumber(hash);
}

/**
 * Constructs a randomized color.
 *
 * @returns CSS color specification.
 */
export function generateRandomColor(): string {
    return colorFromNumber(Math.round(Math.random() * 0x10000001));
}

/**
 * Constructs a deterministic color from a number.
 *
 * @param n Number that is to be color-coded.
 * @returns CSS color specification corresponding to `n`.
 */
function colorFromNumber(n: number): string {
    const h = n % 360;
    const c = ((n / 7) % 100) + 100;
    return `lch(70 ${c} ${h}deg)`;
}
