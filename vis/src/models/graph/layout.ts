/**
 * Interface for layout engines that can be used with a graph view model.
 * 
 * @module
 */

/**
 * Node of a layout graph.
 */
export interface LayoutNode {
    /**
     * Unique identifier of the node.
     */
    readonly id: string;
    /**
     * Offset of the left side of the node's bounding box
     * from the left side of the graph.
     */
    readonly left: number;
    /**
     * Offset of the top side of the node's bounding box
     * from the top side of the graph.
     */
    readonly top: number;
    /**
     * Updates the size of a node's bounding box.
     * 
     * @param width New width of the node.
     * @param height New height of the node.
     */
    setSize(width: number, height: number): void;
    /**
     * Whether the outgoing edges of the node are ordered.
     * 
     * When outgoing edges are ordered, layouts may prefer
     * to lay out successor nodes in a way that preserves
     * their order given by {@link LayoutEdge.order}.
     */
    set orderedOutEdges(value: boolean);
}

/**
 * Edge of a layout graph.
 */
export interface LayoutEdge {
    /**
     * Unique identifier of the edge.
     */
    readonly id: string;
    /**
     * Relative order of the edge among other outgoing edges
     * of its starting node.
     * 
     * If {@link LayoutNode.orderedOutEdges} is set on the starting
     * node, layouts may prefer to lay out its successors in a way
     * that preserves edge order.
     */
    set order(value: number | undefined);
}

/**
 * Graph layout engine.
 */
export interface GraphLayout {
    /**
     * Creates a new node.
     */
    addNode(): LayoutNode;
    /**
     * Removes a node from the layout.
     * 
     * @param id Identifier of the node to remove.
     */
    removeNode(id: string): void;
    /**
     * Retrieves an existing node by its identifier.
     * 
     * @param id Identifier of the node to find.
     */
    getNodeById(id: string): LayoutNode | undefined;
    /**
     * Creates a new edge.
     * 
     * @param startId Identifier of the starting node.
     * @param endId Identifier of the ending node.
     */
    addEdge(startId: string, endId: string): LayoutEdge | undefined;
    /**
     * Removes an edge from the layout.
     * 
     * @param id Identifier of the edge to remove.
     */
    removeEdge(id: string): void;
    /**
     * Retrieves an existing edge by its identifier.
     * 
     * @param id Identifier of the edge to find.
     */
    getEdgeById(id: string): LayoutEdge | undefined;
    /**
     * Asynchronously recalculates the layout of the graph.
     * 
     * @returns Promise that resolves when the calculation is done.
     */
    recalculateLayout(): Promise<void>;
    /**
     * Width of the layout's bounding box.
     */
    readonly width: number;
    /**
     * Height of the layout's bounding box.
     */
    readonly height: number;
}
