/**
 * @jest-environment jsdom
 */

import { beforeEach, describe, expect, it, jest } from '@jest/globals';
import * as bind from '../../src/attribute-binds';

describe(bind.textContent, () => {
    const TEXT_CONTENT = 'hello world';
    let html: HTMLElement;
    let binding: (value: string | undefined) => void;

    beforeEach(() => {
        html = document.createElement('div');
        binding = bind.textContent(html);
    });

    it('updates HTML text content when invoked', () => {
        binding(TEXT_CONTENT);
        expect(html.textContent).toBe(TEXT_CONTENT);
    });

    it('clears HTML text content when reset', () => {
        html.textContent = TEXT_CONTENT; // Set a non-empty content
        binding(undefined);
        expect(html.textContent).toBe('');
    });
});

describe(bind.css, () => {
    const PROPERTY_NAME = 'color';
    const OTHER_PROPERTY_NAME = 'border-color';
    const PROPERTY_VALUE = 'red';
    const ATTRIBUTE_VALUE = 'not red';
    const mockFilter = jest
        .fn<(_: string) => string | undefined>(_ => PROPERTY_VALUE)
        .mockName('filter');
    let html: HTMLElement;

    beforeEach(() => {
        mockFilter.mockClear();
        html = document.createElement('div');
    });

    it('updates CSS property with filtered value', () => {
        bind.css(html, PROPERTY_NAME, mockFilter)(ATTRIBUTE_VALUE);
        expect(mockFilter).toBeCalledWith(ATTRIBUTE_VALUE);
        expect(html.style.getPropertyValue(PROPERTY_NAME)).toBe(PROPERTY_VALUE);
    });

    it('removes property if filter fails', () => {
        html.style.setProperty(PROPERTY_NAME, PROPERTY_VALUE);
        mockFilter.mockImplementationOnce(_ => undefined);
        bind.css(html, PROPERTY_NAME, mockFilter)(ATTRIBUTE_VALUE);
        expect(mockFilter).toBeCalledWith(ATTRIBUTE_VALUE);
        expect(html.style.getPropertyValue(PROPERTY_NAME)).toBe('');
    });

    it('removes property when reset', () => {
        html.style.setProperty(PROPERTY_NAME, PROPERTY_VALUE);
        bind.css(html, PROPERTY_NAME, mockFilter)(undefined);
        expect(mockFilter).not.toBeCalled();
        expect(html.style.getPropertyValue(PROPERTY_NAME)).toBe('');
    });

    it('updates multiple properties at once', () => {
        bind.css(html, [PROPERTY_NAME, OTHER_PROPERTY_NAME], mockFilter)(ATTRIBUTE_VALUE);
        expect(html.style.getPropertyValue(PROPERTY_NAME)).toBe(PROPERTY_VALUE);
        expect(html.style.getPropertyValue(OTHER_PROPERTY_NAME)).toBe(PROPERTY_VALUE);
    });
});

describe(bind.getNumeric, () => {
    const ATTRIBUTE_VALUE = '4.2';
    const NUMERIC_VALUE = 4.2;
    const PROPERTY_VALUE = 42;
    const mockFilter = jest
        .fn<(_: number) => number | undefined>(_ => PROPERTY_VALUE)
        .mockName('filter');

    beforeEach(() => {
        mockFilter.mockClear();
    });

    it('returns the attribute value as a number, passed through the filter', () => {
        const value = bind.getNumeric(mockFilter, ATTRIBUTE_VALUE);
        expect(mockFilter).toBeCalledWith(NUMERIC_VALUE);
        expect(value).toBe(PROPERTY_VALUE);
    });

    it('rejects input if the filter rejects it', () => {
        mockFilter.mockImplementationOnce(() => undefined);
        const value = bind.getNumeric(mockFilter, ATTRIBUTE_VALUE);
        expect(mockFilter).toBeCalledWith(NUMERIC_VALUE);
        expect(value).toBeUndefined();
    });

    it('rejects non-numeric arguments', () => {
        const value = bind.getNumeric(mockFilter, 'abc');
        expect(value).toBeUndefined();
        expect(mockFilter).not.toBeCalled();
    });

    it('rejects trailing non-numeric characters', () => {
        const value = bind.getNumeric(mockFilter, '42 abc');
        expect(value).toBeUndefined();
        expect(mockFilter).not.toBeCalled();
    });
});

describe(bind.numeric, () => {
    const ATTRIBUTE_VALUE = '4.2';
    const NUMERIC_VALUE = 4.2;
    const NUMERIC_PROPERTY_VALUE = 42;
    const PROPERTY_VALUE = '42';
    const UNIT = 'px';
    const PROPERTY_WITH_UNIT = '42px';
    const mockFilter = jest
        .fn<(_: number) => number | undefined>(_ => NUMERIC_PROPERTY_VALUE)
        .mockName('filter');

    beforeEach(() => {
        mockFilter.mockClear();
    });

    it('returns the attribute value as a number, passed through the filter', () => {
        const value = bind.numeric(mockFilter)(ATTRIBUTE_VALUE);
        expect(mockFilter).toBeCalledWith(NUMERIC_VALUE);
        expect(value).toBe(PROPERTY_VALUE);
    });

    it('includes a unit if provided', () => {
        const value = bind.numeric(mockFilter, UNIT)(ATTRIBUTE_VALUE);
        expect(mockFilter).toBeCalledWith(NUMERIC_VALUE);
        expect(value).toBe(PROPERTY_WITH_UNIT);
    });

    it('rejects input if the filter rejects it', () => {
        mockFilter.mockImplementationOnce(() => undefined);
        const value = bind.numeric(mockFilter, UNIT)(ATTRIBUTE_VALUE);
        expect(mockFilter).toBeCalledWith(NUMERIC_VALUE);
        expect(value).toBeUndefined();
    });

    it('rejects non-numeric arguments', () => {
        const value = bind.numeric(mockFilter, UNIT)('abc');
        expect(value).toBeUndefined();
        expect(mockFilter).not.toBeCalled();
    });

    it('rejects trailing non-numeric characters', () => {
        const value = bind.numeric(mockFilter, UNIT)('42 abc');
        expect(value).toBeUndefined();
        expect(mockFilter).not.toBeCalled();
    });
});

describe(bind.whitelist, () => {
    const ATTRIBUTE_VALUES = ['abc', 'def'];
    const INVALID_VALUE = 'xyz';
    const filter = bind.whitelist(ATTRIBUTE_VALUES);

    it('accepts a value that was provided on construction', () => {
        expect(filter(ATTRIBUTE_VALUES[1])).toBe(ATTRIBUTE_VALUES[1]);
    });

    it('rejects a value that was not provided', () => {
        expect(filter(INVALID_VALUE)).toBeUndefined();
    });
});

describe(bind.positive, () => {
    const VALUE = 42;

    it('accepts a positive number', () => {
        expect(bind.positive(VALUE)).toBe(VALUE);
    });

    it('rejects a negative number', () => {
        expect(bind.positive(-1)).toBeUndefined();
    });

    it('rejects a zero', () => {
        expect(bind.positive(0)).toBeUndefined();
    });

    it('rejects a non-number', () => {
        expect(bind.positive(Number.NaN)).toBeUndefined();
    });
});

describe(bind.positiveOrZero, () => {
    const VALUE = 42;

    it('accepts a positive number', () => {
        expect(bind.positiveOrZero(VALUE)).toBe(VALUE);
    });

    it('rejects a negative number', () => {
        expect(bind.positiveOrZero(-1)).toBeUndefined();
    });

    it('accepts a zero', () => {
        expect(bind.positiveOrZero(0)).toBe(0);
    });

    it('rejects a non-number', () => {
        expect(bind.positiveOrZero(Number.NaN)).toBeUndefined();
    });
});

describe(bind.integer, () => {
    const VALUE = 42;
    const NEGATIVE_VALUE = 42;
    const INVALID_VALUE = 4.2;

    it('accepts a positive integer value', () => {
        expect(bind.integer(VALUE)).toBe(VALUE);
    });

    it('accepts a negative integer value', () => {
        expect(bind.integer(NEGATIVE_VALUE)).toBe(NEGATIVE_VALUE);
    });

    it('rejects a non-integer value', () => {
        expect(bind.integer(INVALID_VALUE)).toBeUndefined();
    });

    it('rejects positive infinity', () => {
        expect(bind.integer(Number.POSITIVE_INFINITY)).toBeUndefined();
    });

    it('rejects negative infinity', () => {
        expect(bind.integer(Number.NEGATIVE_INFINITY)).toBeUndefined();
    });

    it('rejects a non-number', () => {
        expect(bind.integer(Number.NaN)).toBeUndefined();
    });
});
