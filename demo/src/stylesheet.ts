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
        // Drop non-breaking spaces from input
        return this.panel.innerText.replace(/\xa0/g, ' ');
    }
    set stylesheetText(stylesheet: string) {
        // Make spaces non-breaking so they do not get omited
        this.panel.innerText = stylesheet.replace(/ /g, '\xa0')
    }
    resetStylesheet(stylesheetId?: BuiltinStylesheet | undefined): void {
        stylesheetId ??= this.currentBaseStylesheet;
        this.currentBaseStylesheet = stylesheetId;
        this.stylesheetText = getSampleStylesheet(stylesheetId);
    }
    private panel: HTMLElement;
    private currentBaseStylesheet: BuiltinStylesheet = BuiltinStylesheet.DEFAULT;
}
