import { beforeEach, describe, expect, it } from '@jest/globals';
import { BLACK, ColorChannels, EM_TOLERANCE, parsePixels, parseResolvedColor, Testbed } from './utils';
import * as vis from '../../src';

const CELL_VALUE = '42';
const COLOR_VALUE = '#a5f78b';
const STROKE_WIDTH = 6;

const EXPECTED_COLOR: ColorChannels = {
    r: 165,
    g: 247,
    b: 139,
    a: 255,
};

describe(vis.CellViewModel, () => {
    const t = new Testbed();
    t.rootElementTagName = vis.TAG_CELL;
    t.theElementSelector = `.${vis.CLASS_CELL}`;

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
        await t.rootElement((root, value) => root.attributes.value.value = value, CELL_VALUE);
        await t.setupViewport();
        expect(await t.textContent()).toBe(CELL_VALUE);
    });

    it('contains text that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement((root, value) => root.attributes.value.value = value, CELL_VALUE);
        expect(await t.textContent()).toBe(CELL_VALUE);
    });

    it('is transparent by default', async () => {
        await t.setupViewport();
        const color = await t.getComputedStyle('background-color');
        // Alpha must be zero, we do not care about the rest
        expect(parseResolvedColor(color).a).toBe(0);
    });

    it('has fill color that was assigned beforehand', async () => {
        await t.rootElement((root, value) => root.attributes.fill.value = value, COLOR_VALUE);
        await t.setupViewport();
        const color = await t.getComputedStyle('background-color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR);
    });

    it('has fill color that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement((root, value) => root.attributes.fill.value = value, COLOR_VALUE);
        const color = await t.getComputedStyle('background-color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR);
    });

    it('has black text by default', async () => {
        await t.setupViewport();
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(BLACK);
    });

    it('has text color that was assigned beforehand', async () => {
        await t.rootElement((root, value) => root.attributes.color.value = value, COLOR_VALUE);
        await t.setupViewport();
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR);
    });

    it('has text color that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement((root, value) => root.attributes.color.value = value, COLOR_VALUE);
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR); 
    });

    it('has black outline by default', async () => {
        await t.setupViewport();
        const color = await t.getComputedStyle('border-color');
        expect(parseResolvedColor(color)).toStrictEqual(BLACK);
    });

    it('has stroke color that was assigned beforehand', async () => {
        await t.rootElement((root, value) => root.attributes.stroke.value = value, COLOR_VALUE);
        await t.setupViewport();
        const color = await t.getComputedStyle('border-color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR); 
    });

    it('has stroke color that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement((root, value) => root.attributes.stroke.value = value, COLOR_VALUE);
        const color = await t.getComputedStyle('border-color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR); 
    });

    it('has 1 pixel thick outline by default', async () => {
        await t.setupViewport();
        const borderWidth = await t.getComputedStyle('border-width');
        // Not exact, we must account for screen resolution
        expect(parsePixels(borderWidth)).toBeCloseTo(1, 0);
    });

    it('has stroke width that was assigned beforehand', async () => {
        await t.rootElement((root, value) => root.attributes['stroke-width'].value = value, String(STROKE_WIDTH));
        await t.setupViewport();
        const borderWidth = await t.getComputedStyle('border-width');
        expect(parsePixels(borderWidth)).toBeCloseTo(STROKE_WIDTH, 0);
    });

    it('has stroke width that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement((root, value) => root.attributes['stroke-width'].value = value, String(STROKE_WIDTH));
        const borderWidth = await t.getComputedStyle('border-width');
        expect(parsePixels(borderWidth)).toBeCloseTo(STROKE_WIDTH, 0);
    });

    it('has stroke style that was assigned beforehand', async () => {
        await t.rootElement(root => root.attributes['stroke-style'].value = 'dashed');
        await t.setupViewport();
        const borderWidth = await t.getComputedStyle('border-style');
        expect(borderWidth).toBe('dashed');
    });

    it('has stroke style that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement(root => root.attributes['stroke-style'].value = 'dotted');
        const borderWidth = await t.getComputedStyle('border-style');
        expect(borderWidth).toBe('dotted');
    });

    it('has size that was assigned beforehand', async () => {
        await t.rootElement(root => root.attributes.size.value = '4');
        await t.setupViewport();
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const boundingBox = await t.boundingBox();
        // This is a very rough estimate, approximating em units with font size
        expect(boundingBox.width).toBeCloseTo(fontSize * 4, EM_TOLERANCE);
        expect(boundingBox.height).toBeCloseTo(fontSize * 4, EM_TOLERANCE);
    });

    it('has size that was assigned later', async () => {
        await t.setupViewport();
        await t.rootElement(root => root.attributes.size.value = '1');
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const boundingBox = await t.boundingBox();
        // This is a very rough estimate, approximating em units with font size
        expect(boundingBox.width).toBeCloseTo(fontSize, EM_TOLERANCE);
        expect(boundingBox.height).toBeCloseTo(fontSize, EM_TOLERANCE);
    });
});
