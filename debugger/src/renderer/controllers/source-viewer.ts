/**
 * Controller for loading and caching of source files for displaying.
 */

import { Logger, Severity } from 'aili-hooligan';
import '../../ipc';

/**
 * Provides access to source code files to display during debugging.
 */
export class SourceViewer {
    constructor() {
        this.cachedContents = {};
    }
    /**
     * Loads the contents of a file or reuses a cached content.
     * 
     * @param fileName Path to the file to read.
     * @returns Lines of the file.
     * @throws The file could not be read.
     */
    async loadFile(fileName: string): Promise<readonly string[]> {
        return this.cachedContents[fileName] ??= await this.getFileAsLines(fileName);
    }
    /**
     * Deletes all cached source files.
     */
    clearCache(): void {
        this.cachedContents = {};
    }
    /**
     * Loads the contents of a file and splits it into lines.
     * 
     * @param fileName Name of the file to read.
     * @returns Contents of the file, split into lines.
     * @throws The file could not be read.
     */
    private async getFileAsLines(fileName: string): Promise<readonly string[]> {
        try {
            const fileContents = await ipc.getFileContents(fileName);
            this.logger?.log(Severity.INFO, `Loaded source file '${fileName}'`);
            return fileContents.split(/\r?\n/);
        } catch (e) {
            this.logger?.log(Severity.ERROR, `Could not load source file '${fileName}': ${e}`);
            throw e;
        }
    }
    /**
     * Logger that will log messages from the viewer.
     */
    logger: Logger | undefined = undefined;
    /**
     * Cached source file contents, mapped by the full file name.
     */
    private cachedContents: Record<string, readonly string[]>;
}
