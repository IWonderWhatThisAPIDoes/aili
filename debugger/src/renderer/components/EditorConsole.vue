<!--
 * Editor window with terminal aesthetics.
-->

<script lang="ts">
    /**
     * Calculates the number of lines in a string.
     * 
     * @param s The source string to examine.
     */
    function countLines(s: string): number {
        let length = (s.match(/\n/g)?.length ?? 0) + 1;
        if (s.endsWith('\n')) {
            length -= 1;
        }
        return length;
    }
</script>

<script setup lang="ts">
    import { ref, useTemplateRef } from 'vue';
    import Console from './Console.vue';

    const { content } = defineProps<{ content?: string }>();
    const emit = defineEmits<{ input: [content: string] }>();
    const lineCount = ref(content ? countLines(content) : 1);
    const inner = useTemplateRef('inner');

    function contentChanged(): void {
        emit('input', inner.value?.innerText ?? '');
        lineCount.value = inner.value ? countLines(inner.value.innerText) : 1;
    }
</script>

<template>
    <Console class="editor-console">
        <div class="editor-line-numbers">
            <div v-for="i in lineCount">{{ i }}</div>
        </div>
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
        gap: 1em;
    }

    .editor-line-numbers {
        color: grey;
        text-align: right;
        pointer-events: none;
        user-select: none;
    }

    .editor-console-inner {
        flex-grow: 1;
        min-height: fit-content;
    }

    .editor-console-inner:focus {
        outline: none;
    }
</style>
