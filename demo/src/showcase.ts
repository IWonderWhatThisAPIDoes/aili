/**
 * The main logic of presenting renderings to the user.
 * 
 * @module
 */

import { DEFAULT_MODEL_FACTORY, Logger, Severity, Viewport, VisConnector, VisElement } from 'aili-vis';
import { applyStylesheet, StateGraph, Stylesheet, VisTreeRenderer } from 'aili-jsapi';
import * as sample from './sample-graph';
import { prettyPrintVisTree } from './pretty-vis';

/**
 * Encapsulates the logic of showcasing visualizations.
 */
export class Showcase {
    constructor(viewportPanel: HTMLElement, logger: Logger) {
        const viewport = new Viewport(viewportPanel, DEFAULT_MODEL_FACTORY);
        this.renderer = new VisTreeRenderer({
            createElement: (tagName: string) => new VisElement(tagName),
            createConnector: () => new VisConnector(),
            set root(root: VisElement) { viewport.root = root; }
        });
        this.logger = logger;
        this.viewport = viewport;
        this.stateGraph = sample.vectorApp();
    }
    stylesheetChanged(stylesheetSource: string): void {
        let stylesheet: Stylesheet;
        try {
            stylesheet = Stylesheet.parse(stylesheetSource, (e: Error) => this.logger.log(Severity.WARNING, e.message));
        } catch (e) {
            this.logger.log(Severity.ERROR, 'Stylesheet failed to parse: ' + e.message);
            return;
        }
        applyStylesheet(stylesheet, this.stateGraph, this.renderer);
    }
    prettyPrintTree(): string {
        if (this.viewport.root) {
            return prettyPrintVisTree(this.viewport.root);
        } else {
            return '[empty tree]';
        }
    }
    prettyPrintResolvedStyle(): string {
        return this.renderer.prettyPrint().replace(/ /g, '\xa0');
    }
    private logger: Logger;
    private viewport: Viewport;
    private renderer: VisTreeRenderer;
    private stateGraph: StateGraph;
}
