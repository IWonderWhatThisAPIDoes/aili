import { beforeEach, describe, expect, it } from '@jest/globals';
import * as vis from '../../src';
import { parsePixels, Testbed } from './utils';
import { BoundingBox, JSHandle } from 'puppeteer';

const CHILD_COUNT = 10;
const PADDING = 2;

describe(vis.GraphViewModel, () => {
    const t = new Testbed();
    t.rootElementTagName = vis.TAG_GRAPH;
    t.theElementSelector = `.${vis.CLASS_GRAPH}`;
    const childSelector = `${t.theElementSelector} .${vis.CLASS_CELL}`;

    beforeEach(() => t.beforeEach());

    function insertChildren(count: number): Promise<JSHandle<vis.VisElement>[]> {
        return Promise.all(Array.from({ length: count }, (_, i) => {
            return page.evaluateHandle((root, value) => {
                const child = new vis.VisElement(vis.TAG_CELL);
                child.attributes.value.value = value;
                child.parent = root;
                return child;
            }, t.rootElementHandle, String(i));
        }));
    }

    function connect(start: JSHandle<vis.VisElement>, end: JSHandle<vis.VisElement>): Promise<JSHandle<vis.VisConnector>> {
        return page.evaluateHandle((start, end) => {
            const connector = new vis.VisConnector();
            connector.start.target = start;
            connector.end.target = end;
            return connector;
        }, start, end);
    }

    async function boundingBoxesByTextContents(): Promise<Record<string, BoundingBox>> {
        const children = await t.appContainer.$$(childSelector);
        const childBounds = await Promise.all(children.map(c => c.boundingBox()));
        const childTexts = await Promise.all(children.map(c => c.getProperty('textContent').then(t => t.jsonValue())));
        return Object.fromEntries(childTexts.map((k, i) => [k, childBounds[i]]));
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
        await insertChildren(CHILD_COUNT);
        await t.setupViewport();
        const children = await t.appContainer.$$(childSelector);
        expect(children).toHaveLength(CHILD_COUNT);
    });

    it('contains inserted children as elements', async () => {
        await t.setupViewport();
        await insertChildren(CHILD_COUNT);
        const children = await t.appContainer.$$(childSelector);
        expect(children).toHaveLength(CHILD_COUNT);
    });

    it('adds padding as requested', async () => {
        await t.rootElement((root, padding) => root.attributes.padding.value = padding, String(PADDING));
        await insertChildren(CHILD_COUNT);
        await t.setupViewport();
        const fontSize = parsePixels(await t.getComputedStyle('font-size'));
        const graphBounds = await t.boundingBox();
        const children = await t.appContainer.$$(childSelector);
        const childBounds = await Promise.all(children.map(c => c.boundingBox()));
        const minLeft = Math.min(...childBounds.map(c => c.x));
        const minTop = Math.min(...childBounds.map(c => c.y))
        const maxRight = Math.max(...childBounds.map(c => c.x + c.width));
        const maxBottom = Math.max(...childBounds.map(c => c.y + c.height));
        expect(minLeft).toBeCloseTo(graphBounds.x + PADDING * fontSize, -1);
        expect(minTop).toBeCloseTo(graphBounds.y + PADDING * fontSize, -1);
        expect(maxRight).toBeCloseTo(graphBounds.x + graphBounds.width - PADDING * fontSize, -1);
        expect(maxBottom).toBeCloseTo(graphBounds.y + graphBounds.height - PADDING * fontSize, -1);
    });

    it('orders elements based on connectors in hierarchical layout', async () => {
        /*   0   3
         *  / \ /
         * 1   2
         *  \  |
         *   4 | 6
         *    \|/ \
         *     5   7
         *     |
         *     8
         */
        await t.rootElement(root => root.attributes.layout.value = vis.GraphLayoutModel.LAYERED);
        const elements = await insertChildren(9);
        await connect(elements[0], elements[1]);
        await connect(elements[0], elements[2]);
        await connect(elements[3], elements[2]);
        await connect(elements[1], elements[4]);
        await connect(elements[4], elements[5]);
        await connect(elements[2], elements[5]);
        await connect(elements[6], elements[5]);
        await connect(elements[6], elements[7]);
        await connect(elements[5], elements[8]);
        await t.setupViewport();
        const boundingBoxes = await boundingBoxesByTextContents();
        expect(boundingBoxes['0'].y).toBeLessThan(boundingBoxes['1'].y);
        expect(boundingBoxes['0'].y).toBeLessThan(boundingBoxes['2'].y);
        expect(boundingBoxes['3'].y).toBeLessThan(boundingBoxes['2'].y);
        expect(boundingBoxes['1'].y).toBeLessThan(boundingBoxes['4'].y);
        expect(boundingBoxes['4'].y).toBeLessThan(boundingBoxes['5'].y);
        expect(boundingBoxes['2'].y).toBeLessThan(boundingBoxes['5'].y);
        expect(boundingBoxes['6'].y).toBeLessThan(boundingBoxes['5'].y);
        expect(boundingBoxes['6'].y).toBeLessThan(boundingBoxes['7'].y);
        expect(boundingBoxes['5'].y).toBeLessThan(boundingBoxes['8'].y);
    });

    it('orders elements sideways when direction is specified', async () => {
        /*
         * 3 <-- 2 <-- 1 <-- 0
         */
        await t.rootElement(root => {
            root.attributes.layout.value = vis.GraphLayoutModel.LAYERED
            root.attributes.direction.value = vis.GraphLayoutDirection.WEST;
        });
        const elements = await insertChildren(4);
        await connect(elements[0], elements[1]);
        await connect(elements[1], elements[2]);
        await connect(elements[2], elements[3]);
        await t.setupViewport();
        const boundingBoxes = await boundingBoxesByTextContents();
        expect(boundingBoxes['0'].x).toBeGreaterThan(boundingBoxes['1'].x);
        expect(boundingBoxes['1'].x).toBeGreaterThan(boundingBoxes['2'].x);
        expect(boundingBoxes['2'].x).toBeGreaterThan(boundingBoxes['3'].x);
    });

    it('orders elements along the cross axis if connectors are ordered', async () => {
        /*    +--4
         *    |
         * +--1--5
         * |  |
         * |  +--3
         * 0
         * |  +--7
         * |  |
         * +--2--6
         *    |
         *    +--8
         */
        await t.rootElement(root => {
            root.attributes.layout.value = vis.GraphLayoutModel.LAYERED;
            root.attributes.direction.value = vis.GraphLayoutDirection.EAST;
        });
        const elements = await insertChildren(9);
        const CONNECTIONS = [[0, 1], [0, 2], [1, 3], [1, 4], [1, 5], [2, 6], [2, 7], [2, 8]];
        const connectors = await Promise.all(CONNECTIONS.map(([from, to]) => connect(elements[from], elements[to])));
        // Set orders for all connectors
        const CONNECTOR_ORDERS = [0, 1, 1, -1, 0, 1, 0, 2];
        await Promise.all(connectors.map((c, i) => {
            page.evaluate((c, o) => c.attributes.order.value = o, c, String(CONNECTOR_ORDERS[i]))
        }));
        // Make all non-leaf elements ordered
        for (const i of [0, 1, 2]) {
            await page.evaluate(e => e.attributes['order-children'].value = 'true', elements[i]);
        }
        await t.setupViewport();
        const boundingBoxes = await boundingBoxesByTextContents();
        expect(boundingBoxes['4'].y).toBeLessThan(boundingBoxes['5'].y);
        expect(boundingBoxes['5'].y).toBeLessThan(boundingBoxes['3'].y);
        expect(boundingBoxes['3'].y).toBeLessThan(boundingBoxes['7'].y);
        expect(boundingBoxes['7'].y).toBeLessThan(boundingBoxes['6'].y);
        expect(boundingBoxes['6'].y).toBeLessThan(boundingBoxes['8'].y);
    });
});
