/**
 * Rendes a B-tree. Demonstrates structured graph nodes and ordered edges.
 *
 * @module
 */

import {
    DEFAULT_MODEL_FACTORY,
    GraphLayoutModel,
    TAG_CELL,
    TAG_GRAPH,
    TAG_ROW,
    TAG_TEXT,
    Viewport,
    VisConnector,
    VisElement,
} from '../../src';

/**
 * Converts node indices from zero-based breadth-first left-to-right order
 * to one-based depth-first in-order-subtree order.
 *
 * ```text
 *   0+1          2+4
 *  / | \  --->  / | \
 * 2  3  4      1  3  5
 * ```
 */
function inOrderIndex(index, nodeWidth, totalLayers) {
    const nodeIndex = Math.floor(index / nodeWidth);
    const layer =
        Math.ceil(Math.log2((nodeIndex + 1) * nodeWidth + 1) / Math.log2(nodeWidth + 1)) - 1;
    const indexWithinLayer = nodeIndex - (Math.pow(nodeWidth + 1, layer) - 1) / nodeWidth;
    const indexWithinNode = index % nodeWidth;
    const layerStride = Math.pow(nodeWidth + 1, totalLayers - layer - 1);
    return (indexWithinLayer * (nodeWidth + 1) + indexWithinNode + 1) * layerStride;
}

addEventListener('load', () => {
    // Structural constants
    const LAYERS = 2; // Number of layers
    const NODE_WIDTH = 3; // Number of items per node
    const LEAF_COUNT = Math.pow(NODE_WIDTH + 1, LAYERS);
    const NODE_COUNT = (LEAF_COUNT - 1) / NODE_WIDTH;

    // Visualization constants
    const NODE_SPACING = 30; // pixels
    const NODE_COLOR = '#d0cdcd';
    const NIL_NODE_COLOR = '#857';
    const NIL_NODE_SIZE = 0.5; // em

    // Create visualization elements
    const root = new VisElement(TAG_GRAPH);
    const nodes = Array.from({ length: NODE_COUNT }, () => new VisElement(TAG_ROW));
    const items = Array.from({ length: NODE_COUNT * NODE_WIDTH }, () => new VisElement(TAG_CELL));
    const pointers = Array.from(
        { length: NODE_COUNT * (NODE_WIDTH + 1) },
        () => new VisElement(TAG_TEXT),
    );
    const nilNodes = Array.from({ length: LEAF_COUNT }, () => new VisElement(TAG_CELL));
    const connectors = Array.from(
        { length: NODE_COUNT + LEAF_COUNT - 1 },
        () => new VisConnector(),
    );

    // Group all nodes together for ease of structure building
    const allNodes = [...nodes, ...nilNodes];

    // Construct visualization tree structure
    nodes.forEach(n => (n.parent = root));
    items.forEach((e, i) => (e.parent = nodes[Math.floor(i / NODE_WIDTH)]));
    pointers.forEach((e, i) => (e.parent = nodes[Math.floor(i / (NODE_WIDTH + 1))]));
    nilNodes.forEach(n => (n.parent = root));
    connectors.forEach((c, i) => {
        c.start.target = pointers[i];
        c.end.target = allNodes[i + 1];
    });

    // Ensure correct ordering of nodes and items within nodes
    items.forEach((e, i) => (e.attributes.order.value = String((i % NODE_WIDTH) * 2 + 1)));
    pointers.forEach((e, i) => (e.attributes.order.value = String((i % (NODE_WIDTH + 1)) * 2)));
    nodes.forEach(n => (n.attributes['order-children'].value = 'true'));
    connectors.forEach((c, i) => (c.attributes.order.value = String(i % (NODE_WIDTH + 1))));

    // Add labels to nodes to indicated intended order
    items.forEach(
        (e, i) => (e.attributes.value.value = String(inOrderIndex(i, NODE_WIDTH, LAYERS))),
    );

    // Make it look pretty
    root.attributes.layout.value = GraphLayoutModel.LAYERED;
    root.attributes.gap.value = String(NODE_SPACING);
    nodes.forEach(n => {
        n.attributes['align-items'].value = 'end';
        n.attributes.fill.value = NODE_COLOR;
    });
    nilNodes.forEach(n => {
        n.attributes.size.value = String(NIL_NODE_SIZE);
        n.attributes.fill.value = NIL_NODE_COLOR;
    });
    connectors.forEach(c => (c.end.attributes.decoration.value = 'arrow'));

    // Attach to the DOM
    const container = document.getElementById('app');
    new Viewport(container, DEFAULT_MODEL_FACTORY).root = root;
});
