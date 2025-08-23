/**
 * Taking stylesheets as inputs from the user.
 *
 * @module
 */

import { BuiltinStylesheet, getSampleStylesheet } from './sample-stylesheet';

/**
 * Encapsulates the acceptance of input from the user.
 */
export class StylesheetInput {
    constructor(inputPanel: HTMLElement) {
        this.panel = inputPanel;
        this.resetStylesheet();
    }
    get stylesheetText(): string {
        return this.panel.innerText;
    }
    resetStylesheet(stylesheetId?: BuiltinStylesheet | undefined): void {
        stylesheetId ??= this.currentBaseStylesheet;
        this.currentBaseStylesheet = stylesheetId;
        this.panel.innerText = getSampleStylesheet(stylesheetId);
    }
    private panel: HTMLElement;
    private currentBaseStylesheet: BuiltinStylesheet = BuiltinStylesheet.DEFAULT;
}
