/**
 * The key-value table view model.
 *
 * @module
 */

import { setAttributeBindings } from '../attributes';
import { ViewModel, ViewLayoutMode } from '../model';
import { ElementViewSlot } from '../slots';
import { ReadonlyVisElement } from '../tree';
import { ViewportContext } from '../viewport-dom';
import * as bind from '../attribute-binds';
import { FlowViewModel } from './flow-base';
import './kvt.css';

/**
 * CSS class for a key-value table element.
 */
export const CLASS_KVT: string = 'aili-kvt';
/**
 * CSS class for the table cell that contains the key in a KVT entry.
 */
export const CLASS_KVT_KEY: string = 'aili-kvt-key';
/**
 * CSS class for the table cell that contains the value in a KVT entry.
 */
export const CLASS_KVT_VALUE: string = 'aili-kvt-value';
/**
 * CSS class for the wrapper for the value in a KVT entry.
 */
export const CLASS_KVT_VALUE_INNER: string = 'aili-kvt-inner';
/**
 * CSS class for the table cell that contains the value in a KVT entry
 * if the value's render model is also a KVT.
 */
export const CLASS_KVT_NESTED: string = 'aili-kvt-nested';

/**
 * {@link ViewModel} that represents an element as a table
 * of key-value pairs.
 *
 * ```text
 * +-----------+
 * |   Point   |
 * +-----------+
 * | x       4 |
 * |- - - - - -|
 * | y       3 |
 * +-----------+
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
 * ### key
 * ```text
 * key: ''
 * ```
 * Key that the child is assigned to.
 *
 * ## Model Attributes
 * The following attributes of {@link ReadonlyVisElement.attributes}
 * affect the appearence of the visual.
 *
 * ### title
 * ```text
 * title: ''
 * ```
 * Title text that appears at the top of the table.
 */
export class KeyValueTableViewModel extends FlowViewModel {
    constructor(element: ReadonlyVisElement, context: ViewportContext) {
        const html = context.ownerDocument.createElement('div');
        const tableHtml = context.ownerDocument.createElement('table');
        const thead = context.ownerDocument.createElement('thead');
        const tbody = context.ownerDocument.createElement('tbody');
        const titleCell = thead.insertRow().insertCell();
        titleCell.colSpan = 2;
        html.className = CLASS_KVT;
        html.append(tableHtml);
        tableHtml.append(tbody);
        super(html);

        this.tbody = tbody;
        this.ownerDocument = context.ownerDocument;
        this.unhookOnDestroy(
            setAttributeBindings(element.attributes, {
                title: value => {
                    if (value) {
                        titleCell.textContent = value;
                        tableHtml.prepend(thead);
                    } else {
                        thead.remove();
                    }
                },
            }),
        );
    }
    createInlineSlot(child: ReadonlyVisElement, childModel: ViewModel): ElementViewSlot {
        const rowHtml = this.tbody.insertRow();
        const keyCell = rowHtml.insertCell();
        const valueCell = rowHtml.insertCell();
        const valueInner = this.ownerDocument.createElement('div');
        valueCell.append(valueInner);
        keyCell.className = CLASS_KVT_KEY;
        valueCell.className = CLASS_KVT_VALUE;
        valueInner.className = CLASS_KVT_VALUE_INNER;

        if (childModel instanceof KeyValueTableViewModel) {
            // Enable special styles for a nested KVT
            valueCell.classList.add(CLASS_KVT_NESTED);
        }

        const observer = setAttributeBindings(child.attributes, {
            key: bind.textContent(keyCell),
        });

        return {
            populator: {
                insertFlowHtml(html) {
                    valueInner.append(html);
                },
            },
            destroy() {
                rowHtml.remove();
                observer.unhook();
            },
        };
    }
    private readonly tbody: HTMLTableSectionElement;
    private readonly ownerDocument: Document;
}
