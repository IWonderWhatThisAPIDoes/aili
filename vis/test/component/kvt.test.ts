import { beforeEach, describe, expect, it } from '@jest/globals';
import { Testbed } from './utils';
import * as vis from '../../src';

const TITLE = 'Table Title';
/**
 * Keys and contents of children against which the table should be tested
 */
const CHILD_CONTENT = {
    key1: 'hello',
    '123': 'world',
    './*+': 'aaaaa',
    '': 'bbbbb',
};

describe(vis.KeyValueTableViewModel, () => {
    const t = new Testbed();
    t.rootElementTagName = vis.TAG_KVT;
    t.theElementSelector = `.${vis.CLASS_KVT}`;
    const theadSelector = `${t.theElementSelector} thead`;
    const trSelector = `${t.theElementSelector} tbody > tr`;
    const childSelector = `${t.theElementSelector} .${vis.CLASS_CELL}`;

    function insertChildren(): Promise<void> {
        return t.rootElement((root, children) => {
            for (const key in children) {
                const child = new vis.VisElement(vis.TAG_CELL);
                child.attributes.key.value = key;
                child.attributes.value.value = children[key];
                child.parent = root;
            }
        }, CHILD_CONTENT);
    }

    async function verifyChildren(): Promise<void> {
        // Extract the text content of all rows
        const rows = await t.appContainer.$$(trSelector);
        const rowTexts = await Promise.all(rows.map(r => r.getProperty('textContent').then(t => t.jsonValue())));
        // There should be one row for each child
        expect(rows).toHaveLength(Object.keys(CHILD_CONTENT).length);
        // Each row should contain its child's key and value
        for (const key in CHILD_CONTENT) {
            expect(rowTexts).toContain(`${key}${CHILD_CONTENT[key]}`);
        }
    }

    beforeEach(() => t.beforeEach());

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

    it('does not contain a header by default', async () => {
        await t.setupViewport();
        expect(await t.appContainer.$$(theadSelector)).toHaveLength(0);
    });

    it('contains a header if set beforehand', async () => {
        await t.rootElement((root, title) => root.attributes.title.value = title, TITLE);
        await t.setupViewport();
        expect(await t.textContent(theadSelector)).toBe(TITLE);
    });

    it('contains a header if set afterwards', async () => {
        await t.setupViewport();
        await t.rootElement((root, title) => root.attributes.title.value = title, TITLE);
        expect(await t.textContent(theadSelector)).toBe(TITLE);
    });

    it('does not contain a header after it is removed', async () => {
        await t.rootElement((root, title) => root.attributes.title.value = title, TITLE);
        await t.setupViewport();
        await t.rootElement(root => root.attributes.title.value = undefined);
        expect(await t.appContainer.$$(theadSelector)).toHaveLength(0);
    });

    it('contains pre-existing children as table rows labeled with their keys', async () => {
        await insertChildren();
        await t.setupViewport();
        await verifyChildren();
    });

    it('contains children inserted afterwardds as table rows labeled with their keys', async () => {
        await t.setupViewport();
        await insertChildren();
        await verifyChildren();
    });

    it('has all values aligned to the right', async () => {
        await t.setupViewport();
        await t.rootElement(root => {
            const child1 = new vis.VisElement(vis.TAG_CELL);
            const child2 = new vis.VisElement(vis.TAG_CELL);
            child1.attributes.size.value = '2';
            child2.attributes.size.value = '4';
            child1.parent = root;
            child2.parent = root;
        });
        const children = await t.appContainer.$$(childSelector);
        const childBounds = await Promise.all(children.map(Testbed.boundingBoxOf));
        expect(children).toHaveLength(2);
        // Right edges should align
        expect(childBounds[0].x + childBounds[0].width).toBeCloseTo(childBounds[1].x + childBounds[1].width);
    });
});
