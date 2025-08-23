/**
 * Settings that affect graph layouts.
 *
 * @module
 */

/**
 * Default distance between graph nodes, in pixels.
 */
export const GRAPH_DEFAULT_GAP: number = 50;

/**
 * Specifies the direction along which edges are oriented
 * in {@link GraphLayoutModel.LAYERED} layout.
 */
export enum GraphLayoutDirection {
    /**
     * Edges are oriented primarily upwards.
     */
    NORTH = 'north',
    /**
     * Edges are oriented primarily downwards.
     */
    SOUTH = 'south',
    /**
     * Edges are oriented primarily to the right.
     */
    EAST = 'east',
    /**
     * Edges are oriented primarily to the left.
     */
    WEST = 'west',
    /**
     * Default setting.
     */
    DEFAULT = SOUTH,
}

/**
 * Specifies the layout model type.
 */
export enum GraphLayoutModel {
    /**
     * Layered or hierarchical model. Best for directed acyclical graphs.
     */
    LAYERED = 'layered',
    /**
     * Model for general graphs. Force-directed or a similar model.
     */
    UNORIENTED = 'unoriented',
    /**
     * Default setting.
     */
    DEFAULT = UNORIENTED,
    /**
     * Forces the [Dot](https://graphviz.org/docs/layouts/dot)
     * layout model if the backend is Graphviz.
     *
     * Dot is a layered or hierarchical model best suited for
     * directed acyclical graphs.
     *
     * Falls back to {@link LAYERED}.
     */
    GRAPHVIZ_DOT = 'gv-dot',
    /**
     * Forces the [Neato](https://graphviz.org/docs/layouts/neato)
     * layout model if the backend is Graphviz.
     *
     * Neato is a spring model intended for small graphs.
     *
     * Falls back to {@link UNORIENTED}.
     */
    GRAPHVIZ_NEATO = 'gv-neato',
    /**
     * Forces the [Circo](https://graphviz.org/docs/layouts/circo)
     * layout model if the backend is Graphviz.
     *
     * Circo lays out each individual
     * [block](https://en.wikipedia.org/wiki/Glossary_of_graph_theory#block)
     * of the graph on the perimeter of a circle.
     * Best suited for graphs with many distinct blocks.
     *
     * Falls back to {@link UNORIENTED}.
     */
    GRAPHVIZ_CIRCO = 'gv-circo',
    /**
     * Forces the [Twopi](https://graphviz.org/docs/layouts/twopi)
     * layout model if the backend is Graphviz.
     *
     * Twopi lays out nodes in concentric circles whose radii
     * roughly correspond to the distance to a root element.
     * Best suited for general trees.
     *
     * Falls back to {@link UNORIENTED}.
     */
    GRAPHVIZ_TWOPI = 'gv-twopi',
    /**
     * Forces the [FDP](https://graphviz.org/docs/layouts/fdp)
     * layout model if the backend is Graphviz.
     *
     * FDP stands for force-directed placement.
     *
     * Falls back to {@link UNORIENTED}.
     */
    GRAPHVIZ_FDP = 'gv-fdp',
}

/**
 * Groups all settings that affect the layout of a graph.
 */
export interface GraphLayoutSettings {
    /**
     * Layout model used to generate layout.
     */
    layoutModel: GraphLayoutModel;
    /**
     * Main direction of {@link GraphLayoutModel.LAYERED} layout
     * if it is in use.
     */
    layoutDirection: GraphLayoutDirection;
    /**
     * Preferred approximate distance between neighboring nodes, in pixels.
     */
    gap: number;
}
