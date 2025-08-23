/**
 * @jest-environment jsdom
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { Viewport } from '../../src/viewport';
import { VisConnector, VisElement } from '../../src/tree';
import { CLASS_ELEMENT, TestViewModel } from './test-model';
import { ViewModelFactory } from '../../src';
import * as jsplumb from '@jsplumb/browser-ui';

const ELEMENT_TAG_NAME = 'test';
const CONTAINER_ID = 'app';
const SELECTOR_ELEMENT = `#${CONTAINER_ID} .${CLASS_ELEMENT}`;
const SELECTOR_CONNECTOR = `#${CONTAINER_ID} .${jsplumb.CLASS_CONNECTOR}`;
const SELECTOR_CONNECTED = `#${CONTAINER_ID} .${jsplumb.CLASS_CONNECTED}`;

const viewModelFactory = new ViewModelFactory(new Map(), TestViewModel);

describe(Viewport, () => {
    let container: HTMLDivElement;
    let root: VisElement;
    let viewport: Viewport;

    beforeEach(() => {
        container = document.createElement('div');
        container.id = CONTAINER_ID;
        document.body.append(container);
        root = new VisElement(ELEMENT_TAG_NAME);
        viewport = new Viewport(container, viewModelFactory);
        viewport.root = root;
    });

    afterEach(() => {
        container.remove();
    });

    it('renders the root element', () => {
        /* Expected:
         * +------+
         * | root |
         * +------+
         */
        expect(document.querySelectorAll(SELECTOR_ELEMENT)).toHaveLength(1);
    });

    it("ignores the root element's parent", () => {
        /* Expected:
         * +------+
         * | root |
         * +------+
         */
        const parent = new VisElement(ELEMENT_TAG_NAME);
        root.parent = parent;
        expect(document.querySelectorAll(SELECTOR_ELEMENT)).toHaveLength(1);
    });

    it("ignores the root element's parent even if it was rendered before", () => {
        /* Expected:
         * +------+
         * | root |
         * +------+
         */
        const parent = new VisElement(ELEMENT_TAG_NAME);
        parent.parent = root;
        parent.parent = undefined;
        root.parent = parent;
        expect(document.querySelectorAll(SELECTOR_ELEMENT)).toHaveLength(1);
    });

    it('renders a child of the root element', () => {
        /* Expected:
         * +-----------+
         * | root      |
         * | +-------+ |
         * | | child | |
         * | +-------+ |
         * +-----------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        child.parent = root;
        expect(document.querySelectorAll(SELECTOR_ELEMENT)).toHaveLength(2);
    });

    it('unrenders a child that has been removed', () => {
        /* Expected:
         * +------+
         * | root |
         * +------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        child.parent = root;
        child.parent = undefined;
        expect(document.querySelectorAll(SELECTOR_ELEMENT)).toHaveLength(1);
    });

    it('renders a child recursively', () => {
        /* Expected:
         * +--------------------+
         * | root               |
         * | +----------------+ |
         * | | child          | |
         * | | +------------+ | |
         * | | | grandchild | | |
         * | | +------------+ | |
         * | +----------------+ |
         * +--------------------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        grandchild.parent = child;
        child.parent = root;
        const matchedDOMElements = document.querySelectorAll(SELECTOR_ELEMENT);
        expect(matchedDOMElements).toHaveLength(3);
        expect(matchedDOMElements[0].contains(matchedDOMElements[1])).toBe(true);
        expect(matchedDOMElements[1].contains(matchedDOMElements[2])).toBe(true);
    });

    it('renders a grandchild when it is added', () => {
        /* Expected:
         * +--------------------+
         * | root               |
         * | +----------------+ |
         * | | child          | |
         * | | +------------+ | |
         * | | | grandchild | | |
         * | | +------------+ | |
         * | +----------------+ |
         * +--------------------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        // Note that the assignments are in the opposite order
        // than in the previous test case
        child.parent = root;
        grandchild.parent = child;
        const matchedDOMElements = document.querySelectorAll(SELECTOR_ELEMENT);
        expect(matchedDOMElements).toHaveLength(3);
        expect(matchedDOMElements[0].contains(matchedDOMElements[1])).toBe(true);
        expect(matchedDOMElements[1].contains(matchedDOMElements[2])).toBe(true);
    });

    it('moves elements between parents', () => {
        /* Expected:
         * +---------------------------+
         * | root                      |
         * | +-------+  +------------+ |
         * | | child |  | grandchild | |
         * | +-------+  +------------+ |
         * +---------------------------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        grandchild.parent = child;
        child.parent = root;
        grandchild.parent = root;
        const matchedDOMElements = document.querySelectorAll(SELECTOR_ELEMENT);
        expect(matchedDOMElements).toHaveLength(3);
        expect(matchedDOMElements[0].contains(matchedDOMElements[1])).toBe(true);
        expect(matchedDOMElements[0].contains(matchedDOMElements[2])).toBe(true);
        expect(matchedDOMElements[1].contains(matchedDOMElements[2])).toBe(false);
    });

    it('re-renders full removed subtree after it is re-inserted', () => {
        /* Expected:
         * +--------------------+
         * | root               |
         * | +----------------+ |
         * | | child          | |
         * | | +------------+ | |
         * | | | grandchild | | |
         * | | +------------+ | |
         * | +----------------+ |
         * +--------------------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        grandchild.parent = child;
        child.parent = root;
        child.parent = undefined;
        child.parent = root;
        const matchedDOMElements = document.querySelectorAll(SELECTOR_ELEMENT);
        expect(matchedDOMElements).toHaveLength(3);
        expect(matchedDOMElements[0].contains(matchedDOMElements[1])).toBe(true);
        expect(matchedDOMElements[1].contains(matchedDOMElements[2])).toBe(true);
    });

    it('does not re-render child elements that were removed while their parent was detached', () => {
        /* Expected:
         * +-----------+
         * | root      |
         * | +-------+ |
         * | | child | |
         * | +-------+ |
         * +-----------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        grandchild.parent = child;
        child.parent = root;
        child.parent = undefined;
        grandchild.parent = undefined;
        child.parent = root;
        const matchedDOMElements = document.querySelectorAll(SELECTOR_ELEMENT);
        expect(matchedDOMElements).toHaveLength(2);
        expect(matchedDOMElements[0].contains(matchedDOMElements[1])).toBe(true);
    });

    it('renders a connector from an element to itself', () => {
        /* Expected:
         * +------+
         * | root <--+
         * +---,--+  |
         *     +-----+
         */
        const conn = new VisConnector();
        conn.start.target = root;
        conn.end.target = root;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).toHaveLength(1);
        expect(document.querySelectorAll(SELECTOR_CONNECTED)).toHaveLength(1);
    });

    it('renders a connector from an element to its child', () => {
        /* Expected:
         * +-----------+
         * | root      |
         * | +-------+ |
         * | | child <----+
         * | +-------+ |  |
         * +--------,--+  |
         *          +-----+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        child.parent = root;
        conn.start.target = root;
        conn.end.target = child;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).toHaveLength(1);
        const connected = document.querySelectorAll(SELECTOR_CONNECTED);
        expect(connected).toHaveLength(2);
        expect(connected[0].contains(connected[1])).toBe(true);
    });

    it('renders a connector between sibling elements', () => {
        /* Expected:
         * +----------------------+
         * | root                 |
         * | +------+   +-------+ |
         * | | left |---> right | |
         * | +------+   +-------+ |
         * +----------------------+
         */
        const left = new VisElement(ELEMENT_TAG_NAME);
        const right = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        left.parent = root;
        right.parent = root;
        conn.start.target = left;
        conn.end.target = right;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).toHaveLength(1);
        const connected = document.querySelectorAll(SELECTOR_CONNECTED);
        expect(connected).toHaveLength(2);
        expect(connected[0].contains(connected[1])).toBe(false);
        expect(connected[1].contains(connected[0])).toBe(false);
    });

    it('does not render a connector with detached element', () => {
        /* Expected:
         * +------+
         * | root |
         * +------+
         */
        const other = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        conn.start.target = root;
        conn.end.target = other;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).not.toContainEqual(expect.anything());
    });

    it('renders a connector with newly inserted element', () => {
        /* Expected:
         * +-----------+
         * | root      |
         * | +-------+ |
         * | | child <----+
         * | +-------+ |  |
         * +--------,--+  |
         *          +-----+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        conn.start.target = root;
        conn.end.target = child;
        // Child is inserted after connector is attached
        child.parent = root;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).toHaveLength(1);
        const connected = document.querySelectorAll(SELECTOR_CONNECTED);
        expect(connected).toHaveLength(2);
        expect(connected[0].contains(connected[1])).toBe(true);
    });

    it('unrenders a connector when its end is detached', () => {
        /* Expected:
         * +-----------+
         * | root      |
         * | +-------+ |
         * | | child | |- - -> * (not rendered)
         * | +-------+ |
         * +-----------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        child.parent = root;
        conn.start.target = root;
        conn.end.target = child;
        // Detach end pin
        conn.end.target = undefined;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).not.toContainEqual(expect.anything());
    });

    it('unrenders a connector when its start is detached', () => {
        /* Expected:
         * +-----------+
         * | root      |
         * | +-------+ |
         * | | child <- - - - * (not rendered)
         * | +-------+ |
         * +-----------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        child.parent = root;
        conn.start.target = root;
        conn.end.target = child;
        // Detach start pin
        conn.start.target = undefined;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).not.toContainEqual(expect.anything());
    });

    it('unrenders a connector when end target element is detached', () => {
        /* Expected:
         * +------+ (not rendered) + - - - +
         * | root | - - - - - - - -> child -
         * +------+                + - - - +
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        child.parent = root;
        conn.start.target = root;
        conn.end.target = child;
        // Detach connection target
        child.parent = undefined;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).not.toContainEqual(expect.anything());
    });

    it('unrenders a connector when start target element is detached', () => {
        /* Expected:
         * +------+ (not rendered) + - - - +
         * | root <- - - - - - - - - child -
         * +------+                + - - - +
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        child.parent = root;
        conn.end.target = root;
        conn.start.target = child;
        // Detach connection target
        child.parent = undefined;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).not.toContainEqual(expect.anything());
    });

    it('renders a connector when its target element is moved', () => {
        /* Expected:
         * +----------------------------+
         * | root                       |
         * | +-------+   +------------+ |
         * | | child |---> grandchild | |
         * | +-------+   +------------+ |
         * +----------------------------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        grandchild.parent = child;
        child.parent = root;
        conn.start.target = child;
        conn.end.target = grandchild;
        grandchild.parent = root;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).toHaveLength(1);
        const connected = document.querySelectorAll(SELECTOR_CONNECTED);
        expect(connected).toHaveLength(2);
        expect(connected[0].contains(connected[1])).toBe(false);
        expect(connected[1].contains(connected[0])).toBe(false);
    });

    it('renders a connector when its endpoint is moved', () => {
        /* Expected:
         * +-----------+
         * | root      |
         * | +-------+ |
         * | | child <----+
         * | +----,--+ |  |
         * +------|----+  |
         *        +-------+
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        child.parent = root;
        conn.start.target = root;
        conn.end.target = child;
        // Move connector target
        conn.start.target = child;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).toHaveLength(1);
        const connected = document.querySelectorAll(SELECTOR_CONNECTED);
        expect(connected).toHaveLength(1);
    });

    it('unrenders a connector when its endpoint is detached indirectly', () => {
        /* Expected:
         * +------+                + - - - - - - - - - +
         * | root |- - - +         -  child            -
         * +------+      -         -  + - - - - - - +  -
         *               + - - - - - -> grandchild  -  -
         *          (not rendered) -  + - - - - - - +  -
         *                         + - - - - - - - - - +
         *
         * The purpose of this test case is to test that when an element is
         * detached, all connectors attached to its whole subtree get erased,
         * not just those connected directly to the element
         */
        const child = new VisElement(ELEMENT_TAG_NAME);
        const grandchild = new VisElement(ELEMENT_TAG_NAME);
        const conn = new VisConnector();
        grandchild.parent = child;
        child.parent = root;
        conn.start.target = root;
        conn.end.target = grandchild;
        // Detach the child that contains the connector somewhere in its subtree
        child.parent = undefined;
        expect(document.querySelectorAll(SELECTOR_CONNECTOR)).not.toContainEqual(expect.anything());
    });

    it('unrenders everything when root element is detached', () => {
        viewport.root = undefined;
        expect(document.querySelectorAll(SELECTOR_ELEMENT)).not.toContainEqual(expect.anything());
    });

    it('renders the new tree when root is swapped', () => {
        const newRoot = new VisElement(ELEMENT_TAG_NAME);
        const child = new VisElement(ELEMENT_TAG_NAME);
        child.parent = newRoot;
        viewport.root = newRoot;
        expect(document.querySelectorAll(SELECTOR_ELEMENT)).toHaveLength(2);
    });
});
