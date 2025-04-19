<!--
 * Editor window with terminal aesthetics.
-->

<script setup lang="ts">
    import { useTemplateRef } from 'vue';
    import Console from './Console.vue';

    defineProps<{ content?: string }>();
    const emit = defineEmits<{ input: [content: string] }>();
    const inner = useTemplateRef('inner');

    function contentChanged(): void {
        emit('input', inner.value?.innerText ?? '');
    }
</script>

<template>
    <Console class="editor-console">
        <div class="editor-console-inner"
            contenteditable="plaintext-only"
            @input="contentChanged"
            ref="inner">
            {{ content }}
        </div>
    </Console>
</template>

<style>
    .editor-console {
        display: flex;
    }

    .editor-console-inner {
        flex-grow: 1;
        min-height: fit-content;
    }

    .editor-console-inner:focus {
        outline: none;
    }
</style>
