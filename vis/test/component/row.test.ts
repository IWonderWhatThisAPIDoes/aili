import { beforeEach, describe, expect, it } from '@jest/globals';
import { parsePixels, Testbed } from './utils';
import { BoundingBox, JSHandle } from 'puppeteer';
import * as vis from '../../src';

const UNIFORM_CHILDREN: [number, string][] = [
    [2, 'abc'],
    [2, 'def'],
    [2, 'ghi'],
];

const NONUNIFORM_CHILDREN: [number, string][] = [
    [2, 'abc'],
    [4, 'def'],
    [3, 'ghi'],
];

const PADDING = 1.5;

describe(vis.RowViewModel, () => {
    const t = new Testbed();
    t.rootElementTagName = vis.TAG_ROW;
    t.theElementSelector = `.${vis.CLASS_ROW}`;
    const childSelector = `${t.theElementSelector} .${vis.CLASS_CELL}`;

    beforeEach(() => t.beforeEach());

    function insertChildren(children: [number, string][]): Promise<JSHandle<vis.VisElement>[]> {
        return Promise.all(children.map(([size, value]) => {
            return page.evaluateHandle((root, size, value) => {
                const child = new vis.VisElement(vis.TAG_CELL);
                child.attributes.size.value = String(size);
                child.attributes.value.value = value;
                child.parent = root;
                return child;
            }, t.rootElementHandle, size, value);
        }));
    }

    async function boundingBoxes(): Promise<{ rowBounds: BoundingBox, childBounds: BoundingBox[] }> {
        const rowBounds = await t.boundingBox();
        const children = await t.appContainer.$$(childSelector);
        const childBounds = await Promise.all(children.map(c => c.boundingBox()));
        return { rowBounds, childBounds };
    }

    async function childTextsInPositionOrder(): Promise<string[]> {
        const children = await t.appContainer.$$(childSelector);
        const childBounds = await Promise.all(children.map(c => c.boundingBox()));
        const childTexts = await Promise.all(children.map(c => c.getProperty('textContent').then(t => t.jsonValue())));
        const childBoundsAndTexts: [BoundingBox, string][] = childBounds.map((b, i) => [b, childTexts[i]]);
        return childBoundsAndTexts.sort((a, b) => a[0].x - b[0].x).map(([_, text]) => text);
    }

    it('renders as an element with the correct class', async () => {
        await t.setupViewport();
        expect(await t.appContainer.$$(t.theElementSelector)).toHaveLength(1);
    });

    it('has nonzero size even when empty', async () => {
        await t.setupViewport();
        const bounds = await t.boundingBox();
        expect(bounds.width).toBeGreaterThan(0);
        expect(bounds.height).toBeGreaterThan(0);
    });

    it('contains pre-existing children as elements', async () => {
        await insertChildren(UNIFORM_CHILDREN);
        await t.setupViewport();
        const children = await t.appContainer.$$(childSelector);
        expect(children).toHaveLength(NONUNIFORM_CHILDREN.length);
    });

    it('contains inserted children as elements', async () => {
        await t.setupViewport();
        await insertChildren(UNIFORM_CHILDREN);
        const children = await t.appContainer.$$(childSelector);
        expect(children).toHaveLength(NONUNIFORM_CHILDREN.length);
    });

    it('lays out its children in a row by default', async () => {
        await t.setupViewport();
        await insertChildren(UNIFORM_CHILDREN);
        const { rowBounds, childBounds } = await boundingBoxes();
        // Get bounding boxes in horizontal position order
        childBounds.sort((a, b) => a.x - b.x);
        // Each child should end before the next starts
        expect(childBounds[0].x).toBeCloseTo(rowBounds.x);
        expect(childBounds[0].x + childBounds[0].width).toBeCloseTo(childBounds[1].x);
        expect(childBounds[1].x + childBounds[1].width).toBeCloseTo(childBounds[2].x);
        expect(childBounds[2].x + childBounds[2].width).toBeCloseTo(rowBounds.x + rowBounds.width);
        // All children should have the same position along secondary axis
        expect(childBounds[0].y).toBeCloseTo(rowBounds.y);
        expect(childBounds[1].y).toBeCloseTo(rowBounds.y);
        expect(childBounds[2].y).toBeCloseTo(rowBounds.y);
    });

    it('lays out its children in a column if set', async () => {
        await t.setupViewport();
        await insertChildren(UNIFORM_CHILDREN);
        await t.rootElement(root => root.attributes.direction.value = 'column');
        const { rowBounds, childBounds } = await boundingBoxes();
        // Get bounding boxes in vertical position order
        childBounds.sort((a, b) => a.y - b.y);
        // Each child should end before the next starts
        expect(childBounds[0].y).toBeCloseTo(rowBounds.y);
        expect(childBounds[0].y + childBounds[0].height).toBeCloseTo(childBounds[1].y);
        expect(childBounds[1].y + childBounds[1].height).toBeCloseTo(childBounds[2].y);
        expect(childBounds[2].y + childBounds[2].height).toBeCloseTo(rowBounds.y + rowBounds.height);
        // All children should have the same position along secondary axis
        expect(childBounds[0].x).toBeCloseTo(rowBounds.x);
        expect(childBounds[1].x).toBeCloseTo(rowBounds.x);
        expect(childBounds[2].x).toBeCloseTo(rowBounds.x);
    });

    it('lays out its children in order if specified', async () => {
        const childVisElements = await insertChildren(UNIFORM_CHILDREN);
        // Set order for children
        await page.evaluate(child => child.attributes.order.value = '1', childVisElements[0]);
        await page.evaluate(child => child.attributes.order.value = '2', childVisElements[1]);
        await t.setupViewport();
        const EXPECTED_ORDER = [
            UNIFORM_CHILDREN[2][1],
            UNIFORM_CHILDREN[0][1],
            UNIFORM_CHILDREN[1][1],
        ];
        expect(await childTextsInPositionOrder()).toStrictEqual(EXPECTED_ORDER);
    });

    it('supports negative values for order indices', async () => {
        await t.setupViewport();
        const childVisElements = await insertChildren(UNIFORM_CHILDREN);
        // Set order for children
        await page.evaluate(child => child.attributes.order.value = '-1', childVisElements[1]);
        await page.evaluate(child => child.attributes.order.value = '1', childVisElements[2]);
        const EXPECTED_ORDER = [
            UNIFORM_CHILDREN[1][1],
            UNIFORM_CHILDREN[0][1],
            UNIFORM_CHILDREN[2][1],
        ];
        expect(await childTextsInPositionOrder()).toStrictEqual(EXPECTED_ORDER);
    });

    it('centers children on the main axis by default', async () => {
        await t.setupViewport();
        await insertChildren(NONUNIFORM_CHILDREN);
        const { rowBounds, childBounds } = await boundingBoxes();
        for (const bounds of childBounds) {
            expect(bounds.y + bounds.height / 2).toBeCloseTo(rowBounds.y + rowBounds.height / 2);
        }
    });

    it('aligns children to the start if requested', async () => {
        await t.setupViewport();
        await insertChildren(NONUNIFORM_CHILDREN);
        await t.rootElement(root => root.attributes['align-items'].value = 'start');
        const { rowBounds, childBounds } = await boundingBoxes();
        for (const bounds of childBounds) {
            expect(bounds.y).toBeCloseTo(rowBounds.y);
        }
    });

    it('aligns children to the end if requested', async () => {
        await t.rootElement(root => {
            root.attributes['align-items'].value = 'end';
            root.attributes.direction.value = 'column';
        });
        await t.setupViewport();
        await insertChildren(NONUNIFORM_CHILDREN);
        const { rowBounds, childBounds } = await boundingBoxes();
        for (const bounds of childBounds) {
            expect(bounds.x + bounds.width).toBeCloseTo(rowBounds.x + rowBounds.width);
        }
    });

    it('adds spacing between children if requested', async () => {
        await t.setupViewport();
        await insertChildren(NONUNIFORM_CHILDREN);
        await t.rootElement((root, padding) => root.attributes.gap.value = padding, String(PADDING));
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const { rowBounds, childBounds } = await boundingBoxes();
        childBounds.sort((a, b) => a.x - b.x);
        // Children should be spaced accordingly
        expect(childBounds[1].x).toBeCloseTo(childBounds[0].x + childBounds[0].width + PADDING * fontSize);
        expect(childBounds[2].x).toBeCloseTo(childBounds[1].x + childBounds[1].width + PADDING * fontSize);
        // There should be no padding around the row as a whole
        expect(childBounds[0].x).toBeCloseTo(rowBounds.x);
        expect(childBounds[2].x + childBounds[2].width).toBeCloseTo(rowBounds.x + rowBounds.width);
        expect(Math.min(...childBounds.map(b => b.y))).toBeCloseTo(rowBounds.y);
        expect(Math.max(...childBounds.map(b => b.y + b.height))).toBeCloseTo(rowBounds.y + rowBounds.height);
    });

    it('adds padding around the whole row if requested', async () => {
        await t.setupViewport();
        await insertChildren(NONUNIFORM_CHILDREN);
        await t.rootElement((root, padding) => root.attributes.padding.value = padding, String(PADDING));
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const { rowBounds, childBounds } = await boundingBoxes();
        childBounds.sort((a, b) => a.x - b.x);
        // Children should be compact
        expect(childBounds[1].x).toBeCloseTo(childBounds[0].x + childBounds[0].width);
        expect(childBounds[2].x).toBeCloseTo(childBounds[1].x + childBounds[1].width);
        // Whole row should be padded
        expect(childBounds[0].x).toBeCloseTo(rowBounds.x + PADDING * fontSize);
        expect(childBounds[2].x + childBounds[2].width).toBeCloseTo(rowBounds.x + rowBounds.width - PADDING * fontSize);
        expect(Math.min(...childBounds.map(b => b.y))).toBeCloseTo(rowBounds.y + PADDING * fontSize);
        expect(Math.max(...childBounds.map(b => b.y + b.height))).toBeCloseTo(rowBounds.y + rowBounds.height - PADDING * fontSize);
    });

    it('adds padding both between and around children if requested', async () => {
        await insertChildren(NONUNIFORM_CHILDREN);
        await t.rootElement((root, padding) => {
            root.attributes.padding.value = padding;
            root.attributes.gap.value = padding;
            root.attributes.direction.value = 'column';
            root.attributes['align-items'].value = 'start';
        }, String(PADDING));
        await t.setupViewport();
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const { rowBounds, childBounds } = await boundingBoxes();
        childBounds.sort((a, b) => a.y - b.y);
        // Children should be spaced accordingly
        expect(childBounds[1].y).toBeCloseTo(childBounds[0].y + childBounds[0].height + PADDING * fontSize);
        expect(childBounds[2].y).toBeCloseTo(childBounds[1].y + childBounds[1].height + PADDING * fontSize);
        // Whole row should be padded
        expect(childBounds[0].y).toBeCloseTo(rowBounds.y + PADDING * fontSize);
        expect(childBounds[2].y + childBounds[2].width).toBeCloseTo(rowBounds.y + rowBounds.height - PADDING * fontSize);
        expect(Math.min(...childBounds.map(b => b.x))).toBeCloseTo(rowBounds.x + PADDING * fontSize);
        expect(Math.max(...childBounds.map(b => b.x + b.width))).toBeCloseTo(rowBounds.x + rowBounds.width - PADDING * fontSize);
    });
});
