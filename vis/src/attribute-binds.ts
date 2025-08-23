/**
 * Shorthands for observers that can be used with attribute hooks.
 *
 * @module
 */

/**
 * Constructs an observer that binds an attribute's
 * value to the text content of an HTML element.
 *
 * @param html The element whose text content should be bound.
 * @returns Callable that can be used as an observer.
 *
 * @example
 * ```
 * let html = document.createElement('span');
 * // Keep the element's text synchronized with the attribute
 * attributes.text.onChange(textContent(html));
 * ```
 */
export function textContent(html: HTMLElement): (value: string | undefined) => void {
    return value => (html.textContent = value ?? '');
}
/**
 * Constructs an observer that binds an attribute's
 * value to one or more CSS properties of an HTML element.
 *
 * @param html The element whose style should be bound.
 * @param propertyName One or more names of style properties that should be bound, in kebab case.
 * @param filter Filter for values that are sent to the binding.
 *               It can modify the value of the argument being read or completely reject it.
 * @returns Callable that can be used as an observer.
 *
 * @example
 * ```
 * let html = document.createElement('span');
 * // Keep the element's text color synchronized with the attribute
 * attributes.color.onChange.hook(css(html, 'color', x => x));
 * ```
 */
export function css(
    html: HTMLElement,
    propertyName: string | readonly string[],
    filter: (attr: string) => string | undefined,
): (value: string | undefined) => void {
    const propertyNameList = typeof propertyName === 'string' ? [propertyName] : propertyName;
    return value => {
        const filteredValue = value && filter(value);
        if (filteredValue !== undefined) {
            propertyNameList.forEach(propertyName => {
                html.style.setProperty(propertyName, filteredValue);
            });
        } else {
            propertyNameList.forEach(propertyName => {
                html.style.removeProperty(propertyName);
            });
        }
    };
}
/**
 * Attribute value filter for attributes that represent colors.
 *
 * @param value Value of the attribute to filter.
 * @returns Filtered value, or `undefined` if the value is rejected.
 *
 * @example
 * ```
 * let html = document.createElement('span');
 * // Keep the element's text color synchronized with the attribute
 * attributes.color.onChange.hook(css(html, 'color', color));
 * ```
 */
export function color(value: string): string | undefined {
    return CSS.supports('color', value) ? value : undefined;
}
/**
 * Shorthand for parsing a numeric attribute value
 * with a constraint on values of the attribute.
 *
 * @param numFilter Filter for numeric values. It can modify
 *                  the value of the argument or completely reject it.
 * @param value Value of the attribute to filter.
 * @returns Filtered numeric value, or `undefined` if the value is rejected.
 */
export function getNumeric(
    numFilter: (v: number) => number | undefined,
    value: string,
): number | undefined {
    let numericValue: number;
    let filteredValue: number | undefined;
    if (
        Number.isFinite((numericValue = Number(value))) &&
        (filteredValue = numFilter(numericValue)) !== undefined
    ) {
        return filteredValue;
    }
    return undefined;
}
/**
 * Constructs an attribute value filter for numeric attributes,
 * optionally with a constraint on the values of the attribute
 * or with units.
 *
 * @param numFilter Filter for numeric values. It can modify
 *                  the value of the argument or completely reject it.
 * @param unit Optionally, units of the attribute, as a string which will
 *             be appended to the value.
 * @returns Attribute value filter.
 *
 * @example
 * ```
 * let filter = numeric(x => x ? x - 1 : undefined, 'em');
 *
 * // Value is decremented and unit is appended
 * filter('42') === '41em';
 * // Falsy value (zero) is rejected
 * filter('0') === undefined;
 * // Non-numeric value is rejected
 * filter('abc') === undefined;
 *
 * // Use the filter with a binding
 * let html = document.createElement('div');
 * attributes.width.onChange.hook(css(html, 'width', filter));
 * ```
 */
export function numeric(
    numFilter: (v: number) => number | undefined,
    unit: string = '',
): (attr: string) => string | undefined {
    return value => {
        const numericValue = getNumeric(numFilter, value);
        if (numericValue !== undefined) {
            return numericValue + unit;
        } else {
            return undefined;
        }
    };
}
/**
 * Numeric value filter that rejects all non-integers.
 *
 * @param value The value to check.
 * @returns The value, unchanged, if it is integer, otherwise `undefined`.
 *
 * @example
 * ```
 * let html = document.createElement('div');
 * // Synchronize an integer-valued CSS property
 * attributes.order.onChange.hook(css(html, 'order', numeric(integer)));
 * ```
 */
export function integer(value: number): number | undefined {
    return Number.isInteger(value) ? value : undefined;
}
/**
 * Numeric value filter that rejects all numbers except positive values and zero.
 *
 * @param value The value to check.
 * @returns The value, unchanged, if it is non-negative, otherwise `undefined`.
 *
 * @example
 * ```
 * let html = document.createElement('div');
 * // Synchronize a non-negative-valued CSS property
 * attributes.padding.onChange.hook(css(html, 'padding', numeric(positiveOrZero, 'px')));
 * ```
 */
export function positiveOrZero(value: number): number | undefined {
    return value >= 0 ? value : undefined;
}
/**
 * Numeric value filter that rejects all numbers except positive values.
 *
 * @param value The value to check.
 * @returns The value, unchanged, if it is positive, otherwise `undefined`.
 *
 * @example
 * ```
 * let html = document.createElement('div');
 * // Synchronize a non-negative CSS property
 * attributes.width.onChange.hook(css(html, 'width', numeric(positive, 'px')));
 * ```
 */
export function positive(value: number): number | undefined {
    return value > 0 ? value : undefined;
}
/**
 * Constructs an attribute value filter that only accepts
 * a set of values.
 *
 * @param permittedValues The values to permit.
 * @returns Attribute value filter.
 *
 * @example
 * ```
 * let filter = whitelist(['start', 'center', 'end']);
 *
 * // Whitelisted values are accepted
 * filter('start') === 'start';
 * // Everything else is rejected
 * filter('abc') === undefined;
 *
 * // Use the filter with a binding
 * let html = document.createElement('div');
 * attributes.width.onChange.hook(css(html, 'justify-content', filter));
 * ```
 */
export function whitelist(permittedValues: readonly string[]): (s: string) => string | undefined {
    return value => (permittedValues.includes(value) ? value : undefined);
}
