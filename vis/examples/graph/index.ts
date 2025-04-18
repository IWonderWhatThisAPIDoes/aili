/**
 * Rendes a (deterministic) graph structure with an unoriented layout.
 * 
 * @module
 */

import {
    DEFAULT_MODEL_FACTORY,
    TAG_CELL,
    TAG_GRAPH,
    Viewport,
    VisConnector,
    VisElement
} from '../../src';

addEventListener('load', () => {
    // Create visualization elements
    const root = new VisElement(TAG_GRAPH);
    const nodes = Array.from({ length: 30 }, () => new VisElement(TAG_CELL));
    const connectors = Array.from({ length: 50 }, () => new VisConnector());

    // Construct visualization tree structure
    nodes.forEach(n => n.parent = root);
    connectors.forEach((c, i) => {
        c.start.target = nodes[i % 30];
        c.end.target = nodes[Math.abs(i % 41 - 7)];
    });

    // Make it look pretty
    nodes.forEach(n => {
        n.attributes.value.value = String(Math.floor(Math.random() * 100));
        n.attributes.fill.value = '#d0cdcd';
    });
    connectors.forEach(c => {
        c.attributes.shape.value = 'quadratic';
        c.end.attributes.decoration.value = 'arrow';
    });

    // Attach to the DOM
    const container = document.getElementById('app');
    new Viewport(container, DEFAULT_MODEL_FACTORY).root = root;
});
