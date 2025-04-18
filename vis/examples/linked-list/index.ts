/**
 * Renders a short linked list. Each node is represented
 * as a key-value table.
 * 
 * @module
 */

import {
    DEFAULT_MODEL_FACTORY,
    GraphLayoutDirection,
    GraphLayoutModel,
    TAG_CELL,
    TAG_GRAPH,
    TAG_KVT,
    TAG_TEXT,
    Viewport,
    VisConnector,
    VisElement
} from '../../src';

addEventListener('load', () => {
    // Constants
    const LENGTH = 4;
    const NODE_SPACING = 100; // pixels
    const POINTER_NODE_SIZE = 0.5; // em
    const POINTER_NODE_COLOR = '#857';

    // Create visualization elements
    const root = new VisElement(TAG_GRAPH);
    const nodes = Array.from({ length: LENGTH }, () => new VisElement(TAG_KVT));
    const valueNodes = Array.from({ length: LENGTH }, () => new VisElement(TAG_TEXT));
    const ptrNodes = Array.from({ length: LENGTH }, () => new VisElement(TAG_CELL));
    const head = new VisElement(TAG_KVT);
    const headPtr = new VisElement(TAG_CELL);
    const connectors = Array.from({ length: LENGTH }, () => new VisConnector());

    // Shorthand for manipulating all pointer nodes
    const allPtrNodes = [headPtr, ...ptrNodes];

    // Construct visualization tree structure
    nodes.forEach(n => n.parent = root);
    valueNodes.forEach((n, i) => n.parent = nodes[i]);
    ptrNodes.forEach((n, i) => n.parent = nodes[i]);
    head.parent = root;
    headPtr.parent = head;
    connectors.forEach((c, i) => {
        c.start.target = allPtrNodes[i];
        c.end.target = nodes[i];
    });

    // Make it look pretty
    root.attributes.layout.value = GraphLayoutModel.LAYERED;
    root.attributes.direction.value = GraphLayoutDirection.EAST;
    root.attributes.gap.value = String(NODE_SPACING);
    head.attributes.title.value = 'Head';
    nodes.forEach(n => n.attributes.title.value = 'Node');
    valueNodes.forEach(n => {
        n.attributes.key.value = 'value';
        n.attributes.value.value = String(Math.floor(Math.random() * 100));
    });
    allPtrNodes.forEach(n => {
        n.attributes.key.value = 'next';
        n.attributes.size.value = String(POINTER_NODE_SIZE);
        n.attributes.shape.value = 'circle';
        n.attributes.fill.value = POINTER_NODE_COLOR;
    });
    allPtrNodes.at(-1).attributes.shape.value = 'square';
    connectors.forEach(c => {
        c.end.attributes.decoration.value = 'arrow';
        c.end.attributes.anchor.value = 'northwest';
    });

    // Attach to the DOM
    const container = document.getElementById('app');
    new Viewport(container, DEFAULT_MODEL_FACTORY).root = root;
});
