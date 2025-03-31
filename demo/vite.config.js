import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import banner from 'vite-plugin-banner';
import bannerJs from './assets/banner-js.txt';
import bannerCss from './assets/banner-css.txt';

export default defineConfig({
    build: {
        outDir: 'out',
    },
    plugins: [
        wasm(),
        topLevelAwait(),
        banner(fileName => {
            if (fileName.endsWith('.js')) {
                return bannerJs;
            } else if (fileName.endsWith('.css')) {
                return bannerCss;
            }
        }),
    ],
});
