import { beforeEach, describe, expect, it } from '@jest/globals';
import * as vis from '../../src';
import { Testbed } from './utils';

describe(vis.CheckboxViewModel, () => {
    const t = new Testbed();
    t.rootElementTagName = vis.TAG_CHECKBOX;
    t.theElementSelector = `.${vis.CLASS_CHECKBOX}`;

    beforeEach(() => t.beforeEach());

    it('renders as an element with the correct class', async () => {
        await t.setupViewport();
        expect(await t.appContainer.$$(t.theElementSelector)).toHaveLength(1);
    });

    it('is unchecked by default', async () => {
        await t.setupViewport();
        expect(await t.appContainer.$$(`${t.theElementSelector}:not(:checked)`)).toHaveLength(1);
    });

    it('is checked if set beforehand', async () => {
        await t.rootElement(root => (root.attributes.checked.value = 'true'));
        await t.setupViewport();
        expect(await t.appContainer.$$(`${t.theElementSelector}:checked`)).toHaveLength(1);
    });

    it('is checked if set afterwards', async () => {
        await t.setupViewport();
        await t.rootElement(root => (root.attributes.checked.value = 'true'));
        expect(await t.appContainer.$$(`${t.theElementSelector}:checked`)).toHaveLength(1);
    });

    it('does not react to input', async () => {
        await t.setupViewport();
        await (await t.theElement()).click();
        expect(await t.appContainer.$$(`${t.theElementSelector}:not(:checked)`)).toHaveLength(1);
    });
});
