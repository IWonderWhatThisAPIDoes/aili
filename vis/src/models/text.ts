/**
 * The inline label view model.
 *
 * @module
 */

import { setAttributeBindings } from '../attributes';
import { ViewLayoutMode, ViewModel } from '../model';
import { ReadonlyVisElement } from '../tree';
import { ViewportContext } from '../viewport-dom';
import { FlowViewModel } from './flow-base';
import * as bind from '../attribute-binds';
import './text.css';

/**
 * CSS class for an inline text element.
 */
export const CLASS_TEXT: string = 'aili-text';

/**
 * {@link ViewModel} that represents an element as a simple
 * text label that is inlined in its parent's layout.
 *
 * ```text
 * + - - +
 * -  42 -
 * + - - +
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
 * The text that will be displayed on the label.
 *
 * ### color
 * ```text
 * color: black
 * ```
 * Color of the label text.
 */
export class TextViewModel extends FlowViewModel {
    constructor(element: ReadonlyVisElement, context: ViewportContext) {
        const html = context.ownerDocument.createElement('div');
        html.className = CLASS_TEXT;
        super(html);

        this.unhookOnDestroy(
            setAttributeBindings(element.attributes, {
                value: bind.textContent(html),
                color: bind.css(html, 'color', bind.color),
            }),
        );
    }
}
