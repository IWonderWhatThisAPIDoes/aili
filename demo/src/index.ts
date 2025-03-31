import { DisplayLogger } from './logging';
import { BuiltinStylesheet, STYLESHEET_NAME, StylesheetInput } from './stylesheet';
import { Showcase } from './showcase';
import { Severity } from 'aili-vis';

/**
 * Shorthand for retrieving an element with a given ID.
 * 
 * @param id ID of the HTML element.
 * @returns The requested element.
 * @throws {@link Error} - The element does not exist.
 */
function interactiveElement(id: string): HTMLElement {
    const element = document.getElementById(id);
    if (!element) {
        throw new Error(`Element #${id} does not exist`);
    }
    return element;
}

// Get all relevant elements from the document
const stylesheetPanel = interactiveElement('stylesheet');
const logPanel = interactiveElement('log');
const viewportPanel = interactiveElement('viewport');
const applyButton = interactiveElement('apply');
const clearButton = interactiveElement('clear');
const resetButton = interactiveElement('reset-style');
const styleSelect = interactiveElement('base-style') as HTMLSelectElement;
const printTreeButton = interactiveElement('print-vis');
const printStyleButton = interactiveElement('print-mapping');

// Fill in options of the stylesheet dropdown
for (const key in STYLESHEET_NAME) {
    const newOption = document.createElement('option');
    newOption.value = BuiltinStylesheet[key];
    newOption.innerText = STYLESHEET_NAME[key];
    styleSelect.options.add(newOption);
}
styleSelect.value = BuiltinStylesheet[BuiltinStylesheet.DEFAULT];

// Set up the application
const logger = new DisplayLogger(logPanel);
const stylesheet = new StylesheetInput(stylesheetPanel);
const showcase = new Showcase(viewportPanel, logger);

function updateRendering() {
    showcase.stylesheetChanged(stylesheet.stylesheetText);
}

// Set up event listeners for action buttons
applyButton.addEventListener('click', updateRendering);
clearButton.addEventListener('click', () => logger.clear());
resetButton.addEventListener('click', () => stylesheet.resetStylesheet());
styleSelect.addEventListener('input', () => stylesheet.resetStylesheet(BuiltinStylesheet[styleSelect.value]));
printTreeButton.addEventListener('click', () => logger.log(Severity.DEBUG, showcase.prettyPrintTree()));
printStyleButton.addEventListener('click', () => logger.log(Severity.DEBUG, showcase.prettyPrintResolvedStyle()));

// Visualize everything initially
updateRendering();
