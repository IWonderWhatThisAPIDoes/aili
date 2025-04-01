import { beforeEach, describe, expect, it } from '@jest/globals';
import * as vis from '../../src';
import { BLACK, ColorChannels, EM_TOLERANCE, parsePixels, parseResolvedColor, Testbed } from './utils';
import { JSHandle } from 'puppeteer';
import 'jest-puppeteer';

const LABEL_VALUE = 'hello world';
const PARENT_SIZE = 12;
const COLOR_VALUE = '#a5f78b';
const PADDING = 2;

const EXPECTED_COLOR: ColorChannels = {
    r: 165,
    g: 247,
    b: 139,
    a: 255,
};

describe(vis.LabelViewModel, () => {
    const t = new Testbed();
    let label: JSHandle<vis.VisElement>;
    t.rootElementTagName = vis.TAG_CELL; // Use a cell as the carrier element
    t.theElementSelector = `.${vis.CLASS_LABEL}`;
    const parentSelector = `.${vis.CLASS_CELL}`;

    function insertLabel(): Promise<void> {
        return t.rootElement((root, label) => label.parent = root, label);
    }

    async function setLabelText(): Promise<void> {
        await page.evaluate((label, value) => label.attributes.value.value = value, label, LABEL_VALUE);
    }

    beforeEach(async () => {
        await t.beforeEach();
        await t.rootElement((root, size) => root.attributes.size.value = size, String(PARENT_SIZE));
        label = await page.evaluateHandle(tagName => new vis.VisElement(tagName), vis.TAG_LABEL);
    });

    it('renders renders when inserted beforehand', async () => {
        await insertLabel();
        await t.setupViewport();
        expect(await t.appContainer.$$(t.theElementSelector)).toHaveLength(1);
    });

    it('renders when inserted afterwards', async () => {
        await t.setupViewport();
        await insertLabel();
        expect(await t.appContainer.$$(t.theElementSelector)).toHaveLength(1);
    });

    it('contains no text by default', async () => {
        await t.setupViewport();
        await insertLabel();
        expect(await t.textContent()).toBe('');
    });

    it('contains text that was assigned beforehand', async () => {
        await setLabelText();
        await t.setupViewport();
        await insertLabel();
        expect(await t.textContent()).toBe(LABEL_VALUE);
    });

    it('contains text that was assigned afterwards', async () => {
        await t.setupViewport();
        await insertLabel();
        await setLabelText();
        expect(await t.textContent()).toBe(LABEL_VALUE);
    });

    it('has black text by default', async () => {
        await t.setupViewport();
        await insertLabel();
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(BLACK);
    });

    it('has text color that was assigned beforehand', async () => {
        await page.evaluate((label, value) => label.attributes.color.value = value, label, COLOR_VALUE);
        await t.setupViewport();
        await insertLabel();
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR);
    });

    it('has text color that was assigned later', async () => {
        await t.setupViewport();
        await insertLabel();
        await page.evaluate((label, value) => label.attributes.color.value = value, label, COLOR_VALUE);
        const color = await t.getComputedStyle('color');
        expect(parseResolvedColor(color)).toStrictEqual(EXPECTED_COLOR); 
    });

    it('is centered on its parent by default', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Center on center
        expect(labelBox.x + labelBox.width / 2).toBeCloseTo(parentBox.x + parentBox.width / 2, EM_TOLERANCE);
        expect(labelBox.y + labelBox.height / 2).toBeCloseTo(parentBox.y + parentBox.height / 2, EM_TOLERANCE);
    });

    it('pushes self to the bottom of parent if set beforehand', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await page.evaluate(label => label.attributes['vertical-justify'].value = 'end', label);
        await t.setupViewport();
        await insertLabel();
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Center on center
        expect(labelBox.x + labelBox.width / 2).toBeCloseTo(parentBox.x + parentBox.width / 2, EM_TOLERANCE);
        // Bottom on bottom
        expect(labelBox.y + labelBox.height).toBeCloseTo(parentBox.y + parentBox.height, EM_TOLERANCE);
    });

    it('pushes self to the top of parent if set afterwards', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate(label => label.attributes['vertical-justify'].value = 'start', label);
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Center on center
        expect(labelBox.x + labelBox.width / 2).toBeCloseTo(parentBox.x + parentBox.width / 2, EM_TOLERANCE);
        // Top on top
        expect(labelBox.y).toBeCloseTo(parentBox.y, EM_TOLERANCE);
    });

    it('pushes self to the left of parent if set beforehand', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await page.evaluate(label => label.attributes['horizontal-justify'].value = 'start', label);
        await t.setupViewport();
        await insertLabel();
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Left on left
        expect(labelBox.x).toBeCloseTo(parentBox.x, EM_TOLERANCE);
        // Center on center
        expect(labelBox.y + labelBox.height / 2).toBeCloseTo(parentBox.y + parentBox.height / 2, EM_TOLERANCE);
    });

    it('pushes self to the right of parent if set afterwards', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate(label => label.attributes['horizontal-justify'].value = 'end', label);
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Right on right
        expect(labelBox.x + labelBox.width).toBeCloseTo(parentBox.x + parentBox.width, EM_TOLERANCE);
        // Center on center
        expect(labelBox.y + labelBox.height / 2).toBeCloseTo(parentBox.y + parentBox.height / 2, EM_TOLERANCE);
    });

    it('pushes self to the top right corner', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate(label => {
            label.attributes['horizontal-justify'].value = 'end';
            label.attributes['vertical-justify'].value = 'start';
        }, label);
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Right on right
        expect(labelBox.x + labelBox.width).toBeCloseTo(parentBox.x + parentBox.width, EM_TOLERANCE);
        // Top on top
        expect(labelBox.y).toBeCloseTo(parentBox.y, EM_TOLERANCE);
    });

    it('pushes self outside of parent vertically if set beforehand', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await page.evaluate(label => {
            label.attributes['vertical-justify'].value = 'start';
            label.attributes['vertical-align'].value = 'outside';
        }, label);
        await t.setupViewport();
        await insertLabel();
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Center on center
        expect(labelBox.x + labelBox.width / 2).toBeCloseTo(parentBox.x + parentBox.width / 2, EM_TOLERANCE);
        // Bottom on top
        expect(labelBox.y + labelBox.height).toBeCloseTo(parentBox.y, EM_TOLERANCE);
    });

    it('pushes self outside of parent vertically if set afterwards', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate(label => {
            label.attributes['vertical-justify'].value = 'end';
            label.attributes['horizontal-justify'].value = 'start';
            label.attributes['vertical-align'].value = 'outside';
        }, label);
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Left on left
        expect(labelBox.x).toBeCloseTo(parentBox.x, EM_TOLERANCE);
        // Top on bottom
        expect(labelBox.y).toBeCloseTo(parentBox.y + parentBox.height, EM_TOLERANCE);
    });

    it('pushes self outside of parent horizontally if set beforehand', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await page.evaluate(label => {
            label.attributes['horizontal-justify'].value = 'start';
            label.attributes['horizontal-align'].value = 'outside';
        }, label);
        await t.setupViewport();
        await insertLabel();
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Right on left
        expect(labelBox.x + labelBox.width).toBeCloseTo(parentBox.x, EM_TOLERANCE);
        // Center on center
        expect(labelBox.y + labelBox.height / 2).toBeCloseTo(parentBox.y + parentBox.height / 2, EM_TOLERANCE);
    });

    it('pushes self outside of parent horizontally if set afterwards', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate(label => {
            label.attributes['horizontal-justify'].value = 'end';
            label.attributes['vertical-justify'].value = 'start';
            label.attributes['horizontal-align'].value = 'outside';
        }, label);
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Left on right
        expect(labelBox.x).toBeCloseTo(parentBox.x + parentBox.width, EM_TOLERANCE);
        // Top on top
        expect(labelBox.y).toBeCloseTo(parentBox.y, EM_TOLERANCE);
    });

    it('pushes self outside the bottom left corner', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate(label => {
            label.attributes['horizontal-justify'].value = 'start';
            label.attributes['vertical-justify'].value = 'end';
            label.attributes['horizontal-align'].value = 'outside';
            label.attributes['vertical-align'].value = 'outside';
        }, label);
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Right on left
        expect(labelBox.x + labelBox.width).toBeCloseTo(parentBox.x, EM_TOLERANCE);
        // Top on bottom
        expect(labelBox.y).toBeCloseTo(parentBox.y + parentBox.height, EM_TOLERANCE);
    });

    it('pushes itself away from parent\'s edge if set beforehand', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await page.evaluate((label, padding) => {
            label.attributes['horizontal-justify'].value = 'end';
            label.attributes['vertical-justify'].value = 'start';
            label.attributes['vertical-align'].value = 'outside';
            label.attributes.padding.value = padding;
        }, label, String(PADDING));
        await t.setupViewport();
        await insertLabel();
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Right on right, with padding
        expect(labelBox.x + labelBox.width).toBeCloseTo(parentBox.x + parentBox.width - PADDING * fontSize, EM_TOLERANCE);
        // Bottom on top, with padding
        expect(labelBox.y + labelBox.height).toBeCloseTo(parentBox.y - PADDING * fontSize, EM_TOLERANCE);
    });

    it('pushes itself away from parent\'s edge if set afterwards', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate((label, padding) => {
            label.attributes['horizontal-justify'].value = 'start';
            label.attributes['vertical-justify'].value = 'end';
            label.attributes['horizontal-align'].value = 'outside';
            label.attributes.padding.value = padding;
        }, label, String(PADDING));
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Right on left, with padding
        expect(labelBox.x + labelBox.width).toBeCloseTo(parentBox.x - PADDING * fontSize, EM_TOLERANCE);
        // Bottom on bottom, with padding
        expect(labelBox.y + labelBox.height).toBeCloseTo(parentBox.y + parentBox.height - PADDING * fontSize, EM_TOLERANCE);
    });

    it('pushes itself into parent\'s top edge if set beforehand', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await insertLabel();
        await page.evaluate(label => {
            label.attributes['vertical-justify'].value = 'start';
            label.attributes['vertical-align'].value = 'middle';
        }, label);
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Center on center
        expect(labelBox.x + labelBox.width / 2).toBeCloseTo(parentBox.x + parentBox.width / 2, EM_TOLERANCE);
        // Center on top
        expect(labelBox.y + labelBox.height / 2).toBeCloseTo(parentBox.y, EM_TOLERANCE);
    });

    it('pushes itself into parent\'s left edge if set afterwards', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await page.evaluate(label => {
            label.attributes['horizontal-justify'].value = 'start';
            label.attributes['horizontal-align'].value = 'middle';
        }, label);
        await insertLabel();
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Center on left
        expect(labelBox.x + labelBox.width / 2).toBeCloseTo(parentBox.x, EM_TOLERANCE);
        // Center on center
        expect(labelBox.y + labelBox.height / 2).toBeCloseTo(parentBox.y + parentBox.height / 2, EM_TOLERANCE);
    });

    it('pushes itself into parent\'s bottom right corner', async () => {
        await setLabelText(); // Set text to ensure nonzero size
        await t.setupViewport();
        await page.evaluate(label => {
            label.attributes['vertical-justify'].value = 'end';
            label.attributes['vertical-align'].value = 'middle';
            label.attributes['horizontal-justify'].value = 'end';
            label.attributes['horizontal-align'].value = 'middle';
        }, label);
        await insertLabel();
        const labelBox = await t.boundingBox();
        const parentBox = await t.boundingBox(parentSelector);
        // Center on right
        expect(labelBox.x + labelBox.width / 2).toBeCloseTo(parentBox.x + parentBox.width, EM_TOLERANCE);
        // Center on bottom
        expect(labelBox.y + labelBox.height / 2).toBeCloseTo(parentBox.y + parentBox.height, EM_TOLERANCE);
    });
});
