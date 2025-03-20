/**
 * The cell view model.
 * 
 * @module
 */

import { setAttributeBindings } from '../attributes';
import { ViewLayoutMode, ViewModel } from '../model';
import { ReadonlyVisElement } from '../tree';
import { ViewportContext } from '../viewport-dom';
import { FlowViewModel } from './flow-base';
import * as bind from '../attribute-binds';
import './cell.css';

/**
 * CSS class for a cell element.
 */
export const CLASS_CELL: string = 'aili-cell';
/**
 * CSS class for the text box in a cell element.
 */
export const CLASS_CELL_INNER: string = 'aili-cell-inner';

/**
 * {@link ViewModel} that represents an element as an outlined cell
 * with a simple label in the center.
 * 
 * ```text
 * +----------+     ,----,      ,--------,
 * |          |   /`      `\   |          |
 * |    42    |  |    42    |  |    42    |
 * |          |   \,      ,/   |          |
 * +----------+     `----`      `--------`
 * ```
 * 
 * ## Permitted Parents
 * Any {@link ViewModel} that permits a {@link ViewLayoutMode.INLINE} child.
 * 
 * ## Permitted Children
 * Only {@link ViewLayoutMode.COMPANION} {@link ViewModel}s.
 * 
 * ## Model Attributes
 * The following attributes of {@link ReadonlyVisElement.attributes}
 * affect the appearence of the visual.
 * 
 * ### value
 * ```text
 * value: ''
 * ```
 * The text that will be displayed in the box.
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
 * stroke-width: 1
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
 * ### color
 * ```text
 * color: black
 * ```
 * Color of the label text.
 * 
 * ### size
 * ```text
 * size: 2
 * ```
 * Size of the cell in em units.
 * 
 * ### shape
 * ```text
 * shape: square
 * ```
 * Shape of the cell. Permitted values are:
 * - `square`
 * - `circle`
 * - `rounded`
 */
export class CellViewModel extends FlowViewModel {
    constructor(element: ReadonlyVisElement, context: ViewportContext) {
        const html = context.ownerDocument.createElement('div');
        const htmlInner = context.ownerDocument.createElement('span');
        html.className = CLASS_CELL;
        htmlInner.className = CLASS_CELL_INNER;
        html.append(htmlInner);
        super(html);

        this.unhookOnDestroy(setAttributeBindings(element.attributes, {
            value: bind.textContent(htmlInner),
            'stroke-width': bind.css(html, 'border-width', bind.numeric(bind.positiveOrZero, 'px')),
            'stroke-style': bind.css(html, 'border-style', bind.whitelist(['solid', 'dashed', 'dotted'])),
            stroke: bind.css(html, 'border-color', bind.color),
            fill: bind.css(html, 'background-color', bind.color),
            color: bind.css(html, 'color', bind.color),
            size: bind.css(html, ['min-width', 'min-height'], bind.numeric(bind.positive, 'em')),
            shape(value) {
                switch (value) {
                    default:
                        html.style.borderRadius = '0';
                        break;
                    case 'rounded':
                        html.style.borderRadius = '0.5em';
                        break;
                    case 'circle':
                        html.style.borderRadius = '50%';
                        break;
                }
            }
        }));
    }
}
