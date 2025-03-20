/**
 * The graph view model.
 * 
 * @module
 */

import { setAttributeBindings } from '../../attributes';
import { ElementViewSlot } from '../../slots';
import { ReadonlyVisConnector, ReadonlyVisElement } from '../../tree';
import { ViewportContext } from '../../viewport-dom';
import { FlowViewModel } from '../flow-base';
import { ViewModel, ViewLayoutMode } from '../../model';
import { GraphSlotManager } from './slot-manager';
import { GRAPH_DEFAULT_GAP, GraphLayoutDirection, GraphLayoutModel } from './layout-settings';
import { GraphvizLayout } from './graphviz-layout';
import * as bind from '../../attribute-binds';
import './graph.css';

export { GRAPH_DEFAULT_GAP, GraphLayoutDirection, GraphLayoutModel };

/**
 * CSS class for a graph element.
 */
export const CLASS_GRAPH: string = 'aili-graph';
/**
 * CSS class for the inner wrapper of a graph element.
 */
export const CLASS_GRAPH_INNER: string = 'aili-graph-inner';
/**
 * CSS class for a wrapper of a graph element's child.
 */
export const CLASS_GRAPH_SLOT: string = 'aili-graph-slot';

/**
 * {@link ViewModel} that represents an element as a graph (as in graph theory, not plot),
 * constructing its layout based on projected connectors.
 * 
 * ```text
 *   o     o
 *  / \    |
 * o   o   o
 *    / \ /
 *   o   o
 *   |
 *   o
 * ```
 * 
 * ## Permitted Parents
 * Any {@link ViewModel} that permits a {@link ViewLayoutMode.INLINE} child.
 * 
 * ## Permitted Children
 * Any {@link ViewModel}.
 * 
 * If a child has {@link ViewLayoutMode.INLINE},
 * the following attributes of {@link ReadonlyVisElement.attributes}
 * of the child affect the layout of this container.
 * 
 * ### order-children
 * ```text
 * order-children: false
 * ```
 * Used with {@link GraphLayoutModel.LAYERED}. Indicates that outgoing connectors
 * of this child are ordered, and should be laid out in order of increasing `order`
 * (see below).
 * 
 * ## Connectors
 * This view model builds its layout based on projected connectors
 * that connects its {@link ViewLayoutMode.INLINE} children.
 * The following attributes of {@link ReadonlyVisConnector.attributes}
 * of the projected connectors affect the layout of this container.
 * 
 * ### order
 * ```
 * order: none
 * ```
 * Used with {@link GraphLayoutModel.LAYERED}.
 * Determines the relative order of the connector within the other
 * outgoing connectors of its {@link ReadonlyVisConnector.start}.
 * Only applicable if {@link ReadonlyVisConnector.start} has `order-children`
 * set. A value of `none` means the order is unspecified.
 * 
 * ## Model Attributes
 * The following attributes of {@link ReadonlyVisElement.attributes}
 * affect the appearence of the visual.
 * 
 * ### layout
 * ```text
 * layout: unoriented
 * ```
 * Layout model that should be used to layout the graph.
 * 
 * For permitted values, see {@link GraphLayoutModel}.
 * 
 * ### stroke-style
 * ```text
 * stroke-style: solid
 * ```
 * Style of the outline. Permitted values are:
 * - `solid`
 * - `dashed`
 * - `dotted`
 * 
 * ### stroke-width
 * ```text
 * stroke-width: 0
 * ```
 * Width of the outline in pixels.
 * 
 * ### stroke
 * ```text
 * stroke: black
 * ```
 * Color of the outline.
 * 
 * ### fill
 * ```text
 * fill: transparent
 * ```
 * Color of the cell backdrop.
 * 
 * ### padding
 * ```text
 * padding: 1
 * ```
 * Padding between the outer edge of the container and its children, in em units.
 * 
 * ### gap
 * ```text
 * gap: 50
 * ```
 * Approximate preferred spacing between individual children, in pixels.
 * 
 * ### direction
 * ```text
 * direction: south
 * ```
 * Used with {@link GraphLayoutModel.LAYERED}. Determines the direction
 * along which the graph is laid out. Corresponds to the `rankdir`
 * attribute in Graphviz.
 * 
 * For permitted values, see {@link GraphLayoutDirection}.
 */
export class GraphViewModel extends FlowViewModel {
    constructor(element: ReadonlyVisElement, context: ViewportContext) {
        const html = context.ownerDocument.createElement('div');
        const htmlInner = context.ownerDocument.createElement('div');
        html.className = CLASS_GRAPH;
        htmlInner.className = CLASS_GRAPH_INNER;
        html.append(htmlInner);
        super(html);

        const layout = new GraphvizLayout();
        this.slotManager = new GraphSlotManager(htmlInner, layout, context);
        this.context = context;

        this.unhookOnDestroy(setAttributeBindings(element.attributes, {
            fill: bind.css(html, 'background-color', bind.color),
            stroke: bind.css(html, 'border-color', bind.color),
            'stroke-width': bind.css(html, 'border-width', bind.numeric(bind.positiveOrZero, 'px')),
            'stroke-style': bind.css(html, 'border-style', bind.whitelist(['solid', 'dashed', 'dotted'])),
            padding: bind.css(html, 'padding', bind.numeric(bind.positive, 'em')),
            layout: value => {
                value = bind.whitelist(Object.values(GraphLayoutModel))(value) ?? GraphLayoutModel.DEFAULT;
                layout.layoutModel = value as GraphLayoutModel;
                this.slotManager.updateLayout();
            },
            direction: value => {
                value = bind.whitelist(Object.values(GraphLayoutDirection))(value) ?? GraphLayoutDirection.DEFAULT;
                layout.layoutDirection = value as GraphLayoutDirection;
                this.slotManager.updateLayout();
            },
            gap: value => {
                let numValue = bind.getNumeric(bind.positiveOrZero, value) ?? GRAPH_DEFAULT_GAP;
                layout.gap = numValue;
                this.slotManager.updateLayout();
            },
        }));
    }
    createInlineSlot(child: ReadonlyVisElement): ElementViewSlot {
        const slotHtml = this.context.ownerDocument.createElement('div');
        slotHtml.className = CLASS_GRAPH_SLOT;
        const id = this.slotManager.addSlot(slotHtml, child);
        return {
            destroy: () => {
                this.slotManager.removeSlot(id);
            },
            populator: {
                insertFlowHtml(inner) {
                    slotHtml.append(inner);
                }
            }
        };
    }
    destroy(): void {
        super.destroy();
        this.slotManager.destroy();
    }
    private readonly slotManager: GraphSlotManager;
    private readonly context: ViewportContext;
}
