/**
 * Renders a data structure as a key-value table.
 * Demonstrates compound key-value entries of various types.
 * 
 * @module
 */

import {
    DEFAULT_MODELS,
    FallbackViewModel,
    TAG_CELL,
    TAG_CHECKBOX,
    TAG_KVT,
    TAG_ROW,
    TAG_TEXT,
    Viewport,
    VisElement
} from '../src';

const ALPHABET = 'abcdefghijklmnopqrstuvwxyz';
const HEX_DIGITS = '0123456789abcdef';

function randomWord(characters, length) {
    return Array.from({ length }, () => characters[Math.floor(Math.random() * characters.length)]).join('');
}

addEventListener('load', () => {
    // The root table
    const root = new VisElement(TAG_KVT);
    root.attributes.title.value = 'Entity Data';

    // Position entry, rendered as a second, smaller table
    const position = new VisElement(TAG_KVT);
    const posNodes = Array.from({ length: 3 }, () => new VisElement(TAG_TEXT));
    position.parent = root;
    posNodes.forEach(n => n.parent = position);
    position.attributes.key.value = 'position';
    posNodes[0].attributes.key.value = 'x';
    posNodes[1].attributes.key.value = 'y';
    posNodes[2].attributes.key.value = 'z';
    posNodes.forEach(n => n.attributes.value.value = String(Math.floor(Math.random() * 100)));

    // Visibility entry, rendered as a checkbox
    const isvisible = new VisElement(TAG_CHECKBOX);
    isvisible.parent = root;
    isvisible.attributes.key.value = 'visible';
    isvisible.attributes.checked.value = 'true';

    // Tags entry, rendered as a row of cell elements
    const tags = new VisElement(TAG_ROW);
    const tagNodes = Array.from({ length: 3 }, () => new VisElement(TAG_CELL));
    tags.parent = root;
    tagNodes.forEach(n => n.parent = tags);
    tags.attributes.key.value = 'markers';
    tags.attributes.gap.value = '0.5';
    tagNodes.forEach(n => {
        n.attributes.value.value = String(Math.floor(Math.random() * 100));
        n.attributes.fill.value = '#' + randomWord(HEX_DIGITS, 6);
    });

    // Attributes entry, rendered as another sub-table
    const attributes = new VisElement(TAG_KVT);
    const attrNodes = Array.from({ length: 5 }, () => new VisElement(TAG_TEXT));
    attributes.parent = root;
    attrNodes.forEach(n => n.parent = attributes);
    attributes.attributes.key.value = 'attributes';
    attributes.attributes.title.value = 'Map';
    attrNodes.forEach(n => {
        n.attributes.key.value = randomWord(ALPHABET, 5);
        n.attributes.value.value = randomWord(ALPHABET, 5);
    });

    // Attach to the DOM
    const container = document.getElementById('app');
    new Viewport(container, DEFAULT_MODELS, FallbackViewModel).root = root;
});
