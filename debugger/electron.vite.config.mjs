import { defineConfig } from 'electron-vite';
import wasm from 'vite-plugin-wasm';
import vue from '@vitejs/plugin-vue';
import banner from 'vite-plugin-banner';
import bannerNoDeps from './assets/banners/no-deps.txt';
import bannerAllDeps from './assets/banners/all-deps.txt';

export default defineConfig({
    main: {
        plugins: [banner(bannerNoDeps)],
    },
    preload: {
        plugins: [banner(bannerNoDeps)],
    },
    renderer: {
        plugins: [
            wasm(),
            vue(),
            banner(fileName => {
                if (fileName.endsWith('.js')) {
                    return bannerAllDeps;
                } else {
                    return bannerNoDeps;
                }
            }),
        ],
    },
});
