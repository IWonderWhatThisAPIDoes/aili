import { createApp } from 'vue';
import App from './App.vue';

const container = document.getElementById('app');
if (!container) {
    throw new Error('Application container is missing');
}

const app = createApp(App);
app.mount(container);
