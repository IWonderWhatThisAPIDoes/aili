/**
 * @jest-environment jsdom
 */

import { afterEach, beforeEach, describe, expect, it } from '@jest/globals';
import { ViewportDOMRoot } from '../../src/viewport-dom';

describe(ViewportDOMRoot, () => {
    let container: HTMLElement;
    let vpdom: ViewportDOMRoot;

    beforeEach(() => {
        container = document.createElement('div');
        document.body.append(container);
        vpdom = new ViewportDOMRoot(container);
    });

    afterEach(() => {
        container.remove();
    });

    it('exposes the owner document of its container', () => {
        expect(vpdom.context.ownerDocument).toBe(container.ownerDocument);
    });

    it('renders its slot into the container', () => {
        const element = vpdom.context.ownerDocument.createElement('div');
        vpdom.slot.populator.insertFlowHtml(element);
        expect(container.contains(element)).toBe(true);
    });
});
