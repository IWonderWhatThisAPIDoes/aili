/**
 * Rollup configuration for building demos
 */

import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import css from 'rollup-plugin-import-css';
import htmlTemplate from 'rollup-plugin-generate-html-template';

export default {
    // Note: this configuration is reused for multiple examples, so it does not
    // specify input and output. These must be specified when calling rollup.
    output: {
        format: 'cjs',
    },
    plugins: [
        nodeResolve(),
        typescript(),
        css({ inject: true }),
        htmlTemplate({
            template: 'examples/template.html',
            target: 'index.html',
        })
    ],
}
