/**
 * Jest configuration for unit tests
 */

export default {
    preset: 'ts-jest',
    testMatch: ['<rootDir>/**/*.test.[jt]s'],
    moduleNameMapper: {
        // Ignore CSS file imports
        '\\.css$': 'identity-obj-proxy',
    },
    globals: {
        'ts-jest': {
            tsConfig: 'typecheck.tsconfig.json',
        },
    },
};
