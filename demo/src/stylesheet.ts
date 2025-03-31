/**
 * Taking stylesheets as inputs from the user.
 * 
 * @module
 */

import * as sample from './sample-stylesheet';

/**
 * Identifiers of pre-written stylesheets that can be
 * used out of the box.
 */
export enum BuiltinStylesheet {
    /**
     * Raw view of the state graph itself.
     */
    STATE_GRAPH,
    /**
     * State graph with color-coded and style-coded
     * elements for ease of reading.
     */
    STATE_GRAPH_PRETTY,
    /**
     * Stack trace. Stack frames are nodes in a graph.
     */
    TRACE_GRAPH,
    /**
     * Stack trace. Stack frames are columns.
     */
    TRACE_COLUMN,
    /**
     * Stylesheet for a simple vector structure.
     */
    VECTOR,
    /**
     * The default setting.
     */
    DEFAULT = STATE_GRAPH_PRETTY,
}

/**
 * Maps keys of built-in stylesheets to their display names.
 */
export const STYLESHEET_NAME = {
    [BuiltinStylesheet.TRACE_GRAPH]: 'Stack trace (graph)',
    [BuiltinStylesheet.TRACE_COLUMN]: 'Stack trace (column)',
    [BuiltinStylesheet.STATE_GRAPH]: 'Raw view',
    [BuiltinStylesheet.STATE_GRAPH_PRETTY]: 'State graph with highlights',
    [BuiltinStylesheet.VECTOR]: 'Vector',
}

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
    resetStylesheet(stylesheetId?: BuiltinStylesheet | undefined): void {
        stylesheetId ??= this.currentBaseStylesheet;
        this.currentBaseStylesheet = stylesheetId;
        this.panel.innerText = BUILTIN_STYLESHEETS[stylesheetId];
    }
    private panel: HTMLElement;
    private currentBaseStylesheet: BuiltinStylesheet = BuiltinStylesheet.DEFAULT;
}

const BUILTIN_STYLESHEETS = {
    [BuiltinStylesheet.STATE_GRAPH_PRETTY]: sample.PRETTY_GRAPH_STYLESHEET,
    [BuiltinStylesheet.TRACE_GRAPH]: sample.TRACE_STYLESHEET,
    [BuiltinStylesheet.TRACE_COLUMN]: sample.COLUMN_TRACE_STYLESHEET,
    [BuiltinStylesheet.STATE_GRAPH]: sample.RAW_STYLESHEET,
    [BuiltinStylesheet.VECTOR]: sample.VECTOR_STYLESHEET,
}
