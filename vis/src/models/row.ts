/**
 * The row/column view model.
 * 
 * @module
 */

import { setAttributeBindings } from '../attributes';
import { ViewLayoutMode, ViewModel } from '../model';
import { ElementViewSlot } from '../slots';
import { ReadonlyVisElement } from '../tree';
import { ViewportContext } from '../viewport-dom';
import { FlowViewModel } from './flow-base';
import * as bind from '../attribute-binds';
import './row.css';

/**
 * CSS class for a row element.
 */
export const CLASS_ROW: string = 'aili-row';
/**
 * CSS class for the wrapper of an inline slot in a row element.
 */
export const CLASS_ROW_SLOT: string = 'aili-row-slot';

/**
 * {@link ViewModel} that represents an element as a flex
 * container that lays out its child elements in a row or column.
 * 
 * ```text
 * +-----------------------------------------------------+
 * |  +-------+ +-------+ +-------+ +-------+ +-------+  |
 * |  |       | |       | |       | |       | |       |  |
 * |  |       | |       | |       | |       | |       |  |
 * |  +-------+ +-------+ +-------+ +-------+ +-------+  |
 * +-----------------------------------------------------+
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
 * ### order
 * ```text
 * order: 0
 * ```
 * Relative order of the child within the container.
 * Must be integer. Can be negative.
 * Children with smaller `order` are laid out at the start
 * of the container. Relative order of elements with the same
 * `order` is unspecified.
 * 
 * ## Model Attributes
 * The following attributes of {@link ReadonlyVisElement.attributes}
 * affect the appearence of the visual.
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
 * padding: 0
 * ```
 * Padding between the outer edge of the container and its children, in em units.
 * 
 * ### gap
 * ```text
 * gap: 0
 * ```
 * Padding between individual children, in em units.
 * 
 * ### direction
 * ```text
 * direction: row
 * ```
 * Orientation of the container's main axis. Permitted values are:
 * - `row`
 * - `column`
 * 
 * ### align-items
 * ```text
 * align-items: center
 * ```
 * Alignment of children in the direction orthogonal to the main axis.
 * Permitted values are:
 * - `start`
 * - `center`
 * - `end`
 */
export class RowViewModel extends FlowViewModel {
    constructor(element: ReadonlyVisElement, context: ViewportContext) {
        const html = context.ownerDocument.createElement('div');
        html.className = CLASS_ROW;
        super(html);

        this.ownerDocument = context.ownerDocument;
        this.unhookOnDestroy(setAttributeBindings(element.attributes, {
            'stroke-width': bind.css(html, 'border-width', bind.numeric(bind.positiveOrZero, 'px')),
            'stroke-style': bind.css(html, 'border-style', bind.whitelist(['solid', 'dashed', 'dotted'])),
            stroke: bind.css(html, 'border-color', bind.color),
            fill: bind.css(html, 'background-color', bind.color),
            padding: bind.css(html, 'padding', bind.numeric(bind.positiveOrZero, 'em')),
            gap: bind.css(html, 'gap', bind.numeric(bind.positiveOrZero, 'em')),
            direction: bind.css(html, 'flex-direction', bind.whitelist(['row', 'column'])),
            'align-items': bind.css(html, 'align-items', bind.whitelist(['start', 'center', 'end'])),
        }));
    }
    createInlineSlot(child: ReadonlyVisElement): ElementViewSlot {
        const slotHtml = this.ownerDocument.createElement('div');
        slotHtml.className = CLASS_ROW_SLOT;
        this.html.append(slotHtml);
        const observer = setAttributeBindings(child.attributes, {
            order: bind.css(slotHtml, 'order', bind.numeric(bind.integer)),
        });
        return {
            destroy() {
                slotHtml.remove();
                observer.unhook();
            },
            populator: {
                insertFlowHtml(inner) {
                    slotHtml.append(inner);
                }
            }
        }
    }
    private readonly ownerDocument: Document;
}
