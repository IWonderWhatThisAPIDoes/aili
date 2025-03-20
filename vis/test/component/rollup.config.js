/**
 * Rollup configuration for building the browser test fixture
 */

import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import css from 'rollup-plugin-import-css';
import { rollupPluginHTML as html } from '@web/rollup-plugin-html';

export default {
    input: 'test/component/testbed.html',
    output: {
        dir: 'test/component/out',
        format: 'cjs',
    },
    plugins: [
        nodeResolve(),
        typescript(),
        css({ inject: true }),
        html()
    ],
}
