/**
 * {@link GraphLayout} implementation backed by [Graphviz](https://graphviz.org/).
 *
 * @module
 */

import { GraphLayout, LayoutEdge, LayoutNode } from './layout';
import {
    GRAPH_DEFAULT_GAP,
    GraphLayoutDirection,
    GraphLayoutModel,
    GraphLayoutSettings,
} from './layout-settings';
import * as graphviz from '@viz-js/viz';

/**
 * {@link GraphLayout} that constructs its layouts using [Graphviz](https://graphviz.org/).
 */
export class GraphvizLayout implements GraphLayout, GraphLayoutSettings {
    constructor() {
        this.nodes = {};
        this.edges = {};
        this.graphviz = graphviz.instance();
        this.graph = {
            nodes: [],
            edges: [],
            directed: true,
            graphAttributes: {
                // All models except DOT: Voronoi ovelap removal performs best
                overlap: 'voronoi',
            },
        };
        this.layoutModel = GraphLayoutModel.DEFAULT;
        this.layoutDirection = GraphLayoutDirection.DEFAULT;
        this.gap = GRAPH_DEFAULT_GAP;
    }
    set layoutModel(layout: GraphLayoutModel) {
        this.graph.graphAttributes ??= {};
        this.graph.graphAttributes.layout = LAYOUT_MODEL_TO_GRAPHVIZ[layout];
    }
    set layoutDirection(direction: GraphLayoutDirection) {
        this.graph.graphAttributes ??= {};
        this.graph.graphAttributes.rankdir = LAYOUT_DIRECTION_TO_GRAPHVIZ[direction];
    }
    set gap(gap: number) {
        this.graph.graphAttributes ??= {};
        // For DOT and TWOPI layout
        this.graph.graphAttributes.nodesep = gap / POINTS_PER_INCH;
        this.graph.graphAttributes.ranksep = gap / POINTS_PER_INCH;
        // For NEATO and FDP layout
        this.graph.graphAttributes.sep = gap / POINTS_PER_INCH;
        this.graph.graphAttributes.defaultdist = gap / POINTS_PER_INCH;
        // For CIRCO layout
        this.graph.graphAttributes.minDist = gap / POINTS_PER_INCH;
    }
    addNode(): LayoutNode {
        const nodeId = String(this.nextNodeId++);
        const node = new GraphvizLayoutNode(nodeId);
        this.nodes[nodeId] = node;
        this.graph.nodes ??= [];
        node.index = this.graph.nodes.length;
        this.graph.nodes.push(node.node);
        return node;
    }
    getNodeById(id: string): LayoutNode | undefined {
        return this.nodes[id];
    }
    removeNode(nodeId: string) {
        const node = this.nodes[nodeId];
        if (!node) {
            return;
        }
        // Remove node from layout
        if (this.graph.nodes) {
            delete this.graph.nodes[node.index];
        }
        // Forget the slot data to make it eligible for GC
        delete this.nodes[nodeId];
    }
    addEdge(startId: string, endId: string): LayoutEdge {
        const layoutEdge = { tail: startId, head: endId, attributes: {} };
        this.graph.edges ??= [];
        const index = this.graph.edges.push(layoutEdge) - 1;
        const edgeId = String(this.nextEdgeId++);
        const edge = new GraphvizLayoutEdge(edgeId);
        edge.index = index;
        edge.edgeOrderChanged = () => this.rebuildEdgeOrder();
        this.edges[edgeId] = edge;
        return edge;
    }
    getEdgeById(id: string): LayoutEdge | undefined {
        return this.edges[id];
    }
    removeEdge(id: string): void {
        const edge = this.edges[id];
        if (!edge) {
            return;
        }
        if (this.graph.edges) {
            delete this.graph.edges[edge.index];
        }
        delete this.edges[id];
    }
    async recalculateLayout(): Promise<void> {
        const graphviz = await this.graphviz;
        const layout = graphviz.renderJSON(this.graph);
        const bb = (layout as { bb: string }).bb.split(',').map(Number.parseFloat);
        this.width = bb[2];
        this.height = bb[3];
        for (const object of (
            layout as { objects: { name: string; pos: string; width: string; height: string }[] }
        ).objects ?? []) {
            const slot = this.nodes[object.name];
            const pos = object.pos.split(',').map(Number.parseFloat);
            slot.left = pos[0] - (Number.parseFloat(object.width) * POINTS_PER_INCH) / 2;
            slot.top =
                this.height - (Number.parseFloat(object.height) * POINTS_PER_INCH) / 2 - pos[1];
            slot.node.attributes.pos = `${(pos[0] * POINTS_PER_INCH) / 2},${(pos[1] * POINTS_PER_INCH) / 2}`;
        }
    }
    /**
     * Reorders edges of the graph to reflect an update
     * to {@link LayoutEdge.order}.
     *
     * Internally, Graphviz orders edges in declaration order
     * when [ordering](https://graphviz.org/docs/attrs/ordering/)
     * is set on their endpoint nodes. Here, declaration order
     * is the order of the edges in the Graphviz graph,
     * which we must sort by the declared order.
     */
    private rebuildEdgeOrder(): void {
        // Sort edges by their declared order
        // Edges with undefined order do not matter, but we need the comparison
        // function to be consistent, so they are assigned an order of zero.
        const sortedEdges = Object.values(this.edges).sort(
            (a, b) => (a.order ?? 0) - (b.order ?? 0),
        );
        // Get the edges as they are currently in the graph
        const oldEdges = (this.graph.edges ??= []);
        // Place edges into the graph in the new order
        // and reassign indices to edge handles
        this.graph.edges = sortedEdges.map((edge, i) => {
            const structureEdge = oldEdges[edge.index];
            edge.index = i;
            return structureEdge;
        });
    }
    width: number = 0;
    height: number = 0;
    private nextNodeId: number = 1;
    private nextEdgeId: number = 1;
    private readonly graph: graphviz.Graph;
    private readonly graphviz: Promise<graphviz.Viz>;
    private readonly nodes: Record<string, GraphvizLayoutNode>;
    private readonly edges: Record<string, GraphvizLayoutEdge>;
}

class GraphvizLayoutNode implements LayoutNode {
    constructor(id: string) {
        this.node = { name: id, attributes: {} };
    }
    setSize(width: number, height: number): void {
        this.node.attributes.width = width / POINTS_PER_INCH;
        this.node.attributes.height = height / POINTS_PER_INCH;
    }
    left: number = 0;
    top: number = 0;
    get id(): string {
        return this.node.name;
    }
    readonly node: { name: string; attributes: Record<string, string | number> };
    index: number;
    set orderedOutEdges(value: boolean) {
        this.node.attributes.ordering = value ? 'out' : '';
    }
}

class GraphvizLayoutEdge implements LayoutEdge {
    constructor(id: string) {
        this.id = id;
    }
    readonly id: string;
    index: number;
    set order(value: number | undefined) {
        this._order = value;
        if (value !== undefined) {
            this.edgeOrderChanged();
        }
    }
    get order(): number | undefined {
        return this._order;
    }
    _order: number | undefined = undefined;
    edgeOrderChanged: () => void;
}

const LAYOUT_DIRECTION_TO_GRAPHVIZ = {
    [GraphLayoutDirection.NORTH]: 'BT',
    [GraphLayoutDirection.SOUTH]: 'TB',
    [GraphLayoutDirection.EAST]: 'LR',
    [GraphLayoutDirection.WEST]: 'RL',
};

const LAYOUT_MODEL_TO_GRAPHVIZ = {
    [GraphLayoutModel.LAYERED]: 'dot',
    [GraphLayoutModel.UNORIENTED]: 'neato',
    [GraphLayoutModel.GRAPHVIZ_DOT]: 'dot',
    [GraphLayoutModel.GRAPHVIZ_NEATO]: 'neato',
    [GraphLayoutModel.GRAPHVIZ_FDP]: 'fdp',
    [GraphLayoutModel.GRAPHVIZ_CIRCO]: 'circo',
    [GraphLayoutModel.GRAPHVIZ_TWOPI]: 'twopi',
};

const POINTS_PER_INCH: number = 72;
