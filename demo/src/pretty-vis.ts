/**
 * Pretty-printing the visualization tree.
 * 
 * @module
 */

import { ReadonlyVisConnector, ReadonlyVisElement } from "aili-vis";

/**
 * Formats the structure of a visualization tree
 * for displaying to the user.
 * 
 * @param root Root element of the tree to show.
 */
export function prettyPrintVisTree(root: ReadonlyVisElement): string {
    let printout = '';
    const connectorNumbers = new Map<ReadonlyVisConnector, number>();

    function prettyPrintSubtree(root: ReadonlyVisElement, indent: string = '') {
        printout += `${indent}<${root.tagName}>`;
        let hadChildren = false;
        for (const child of root.children) {
            hadChildren = true;
            printout += '\n';
            prettyPrintSubtree(child, indent + '|\xa0');
        }
        if (hadChildren) {
            printout += `\n${indent}</${root.tagName}>`;
        } else {
            printout += `</${root.tagName}>`;
        }
        for (const pin of root.pins) {
            let connectorNumber = connectorNumbers.get(pin.connector);
            if (!connectorNumber) {
                connectorNumbers.set(pin.connector, connectorNumber = connectorNumbers.size + 1);
            }
            let pinString;
            if (pin === pin.connector.start) {
                pinString = '>';
            } else {
                pinString = '<';
            }
            printout += `\n${indent}\xa0'${pinString}-- ${connectorNumber}`;
        }
    }

    prettyPrintSubtree(root);
    return printout;
}
