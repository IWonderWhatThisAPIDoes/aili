/**
 * The checkbox view model.
 *
 * @module
 */

import { setAttributeBindings } from '../attributes';
import { ReadonlyVisElement } from '../tree';
import { ViewportContext } from '../viewport-dom';
import { FlowViewModel } from './flow-base';
import { ViewModel, ViewLayoutMode } from '../model';

/**
 * CSS class for a checkbox element.
 */
export const CLASS_CHECKBOX: string = 'aili-checkbox';

/**
 * {@link ViewModel} that represents an element as a checkbox
 * that can be checked or unchecked.
 * ```text
 * [x]
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
 * ### checked
 * ```text
 * checked: false
 * ```
 * Whether the checkbox is checked.
 */
export class CheckboxViewModel extends FlowViewModel {
    constructor(element: ReadonlyVisElement, context: ViewportContext) {
        const html = context.ownerDocument.createElement('input');
        html.type = 'checkbox';
        html.disabled = true;
        html.className = CLASS_CHECKBOX;
        super(html);

        this.unhookOnDestroy(
            setAttributeBindings(element.attributes, {
                checked(value) {
                    html.checked = value === 'true';
                },
            }),
        );
    }
}
