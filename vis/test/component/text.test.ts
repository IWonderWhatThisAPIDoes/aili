import { beforeEach, describe, expect, it } from '@jest/globals';
import * as vis from '../../src';
import { BLACK, ColorChannels, parseResolvedColor, Testbed } from './utils';

const TEXT_VALUE = 'hello world';
const COLOR_VALUE = '#a5f78b';

const EXPECTED_COLOR: ColorChannels = {
    r: 165,
    g: 247,
    b: 139,
    a: 255,
};

describe(vis.TextViewModel, () => {
    const t = new Testbed();
    t.rootElementTagName = vis.TAG_TEXT;
    t.theElementSelector = `.${vis.CLASS_TEXT}`;

    beforeEach(() => t.beforeEach());

    it('renders as an element with the correct class', async () => {
        await t.setupViewport();
        expect(await t.appContainer.$$(t.theElementSelector)).toHaveLength(1);
    });

    it('contains no text by default', async () => {
        await t.setupViewport();
        expect(await t.textContent()).toBe('');
    });

    it('contains text that was assigned beforehand', async () => {
        await t.rootElement((root, value) => (root.attributes.value.value = value), TEXT_VALUE);
        await t.setupViewport();
        expect(await t.textContent()).toBe(TEXT_VALUE);
    });

    it('contains text that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement((root, value) => (root.attributes.value.value = value), TEXT_VALUE);
        expect(await t.textContent()).toBe(TEXT_VALUE);
    });

    it('has black text by default', async () => {
        await t.setupViewport();
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(BLACK);
    });

    it('has text color that was assigned beforehand', async () => {
        await t.rootElement((root, value) => (root.attributes.color.value = value), COLOR_VALUE);
        await t.setupViewport();
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR);
    });

    it('has text color that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement((root, value) => (root.attributes.color.value = value), COLOR_VALUE);
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR);
    });
});
