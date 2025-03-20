import { ViewLayoutMode, ViewModel } from '../../src/model';
import { ElementViewSlot, ViewSlotPopulator } from '../../src/slots';
import { ViewportContext } from '../../src/viewport-dom';

/**
 * CSS class of an element rendered with the test view.
 */
export const CLASS_ELEMENT: string = 'test-element';

/**
 * Simple view model for testing viewports.
 */
export class TestViewModel implements ViewModel {
    constructor(_, context: ViewportContext) {
        this.element = context.ownerDocument.createElement('div');
        this.element.className = CLASS_ELEMENT;
        this.pinContainer = this.element;
        this.companionSlot = {
            populator: {
                insertFlowHtml: (html: HTMLElement) => this.element.append(html)
            },
            destroy(): void {},
        };
        // Add some inline styles for convenience
        this.element.style.border = 'solid 1px';
        this.element.style.display = 'flex';
        this.element.style.gap = '0.5em';
        this.element.style.padding = '0.5em';
        this.element.style.alignItems = 'center';
    }
    useSlot(populator: ViewSlotPopulator | undefined): void {
        if (populator) {
            populator.insertFlowHtml(this.element);
        } else {
            this.element.remove();
        }
    }
    createInlineSlot(): ElementViewSlot {
        // Use the same slot for inline elements too
        return this.companionSlot;
    }
    destroy(): void {
        // Nothing to clean up
    }
    readonly preferredLayoutMode = ViewLayoutMode.INLINE;
    readonly companionSlot: ElementViewSlot;
    readonly pinContainer: HTMLElement;
    private element: HTMLDivElement;
}
