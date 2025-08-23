/**
 * Displaying logs to the user.
 *
 * @module
 */

import { Logger, Severity } from 'aili-hooligan';

/**
 * CSS class for a log entry.
 */
export const CLASS_LOG_ENTRY: string = 'log-line';
/**
 * Prefix for CSS classes that indicate log severity.
 */
export const CLASS_SEVERITY_PREFIX: string = 'severity';

/**
 * Logger that displays logs in an HTML element.
 */
export class DisplayLogger implements Logger {
    /**
     * Constructs a new logger and binds it to an HTML element.
     *
     * @param panelElement The HTML element that will display the logs.
     */
    constructor(panelElement: HTMLElement) {
        this.panel = panelElement;
    }
    /**
     * Logs a new message.
     *
     * @param severity Message severity.
     * @param message Message text.
     */
    log(severity: Severity, message: string): void {
        const line = this.panel.ownerDocument.createElement('div');
        line.innerText = message;
        line.classList.add(CLASS_LOG_ENTRY, `${CLASS_SEVERITY_PREFIX}-${severity}`);
        this.panel.append(line);
    }
    /**
     * Clears the displayed log history.
     */
    clear(): void {
        this.panel.innerHTML = '';
    }
    private panel: HTMLElement;
}
