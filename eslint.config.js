import { defineConfig } from 'eslint/config';
import { includeIgnoreFile } from '@eslint/compat';
import js from '@eslint/js';
import ts from 'typescript-eslint';
import jsdoc from 'eslint-plugin-jsdoc';
import tsdoc from 'eslint-plugin-tsdoc';
import path from 'path';

export default defineConfig([
    includeIgnoreFile(path.resolve('.gitignore')),
    {
        files: ['**/*.ts'],
        plugins: { ts, jsdoc, tsdoc },
        extends: ['ts/recommended'],
        rules: {
            '@typescript-eslint/no-unused-vars': [
                2,
                {
                    args: 'none',
                    varsIgnorePattern: '^_$',
                },
            ],
            'tsdoc/syntax': 1,
            // Enabling this suppresses no-unused-vars when imported symbol
            // is used in tsdoc comments and nowhere else
            // Only at warning level because
            'jsdoc/no-undefined-types': 1,
        },
    },
    {
        files: ['**/*.js'],
        plugins: { js },
        extends: ['js/recommended'],
    },
]);
