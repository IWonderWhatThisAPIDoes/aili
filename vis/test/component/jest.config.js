/**
 * Jest configuration for tests that must be run with Puppeteer
 */

import merge from 'merge';
import tsPreset from 'ts-jest/jest-preset.js';
import puppeteer from 'jest-puppeteer/jest-preset.js';

export default merge.recursive(
    tsPreset,
    puppeteer,
    {
        testMatch: ['<rootDir>/**/*.test.[jt]s'],
        moduleNameMapper: {
            // Ignore CSS file imports
            '\\.css$': 'identity-obj-proxy',
        },
    }
);
