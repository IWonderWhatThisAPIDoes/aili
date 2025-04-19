<!--
 * Stylesheet editor with a warning panel.
-->

<script setup lang="ts">
    import { nextTick, useTemplateRef } from 'vue';
    import { Severity } from 'aili-hooligan';
    import { Stylesheet, StylesheetParseError } from 'aili-jsapi';
    import ScrollBox from './ScrollBox.vue';
    import EditorConsole from './EditorConsole.vue';
    import LogConsole from './LogConsole.vue';

    const { content } = defineProps<{ content?: string }>();
    const emit = defineEmits<{ 'style-changed': [source: string, stylesheet: Stylesheet] }>();

    const logView = useTemplateRef('editor-log');

    function stylesheetChanged(newSource: string) {
        // Remove logs from the pase iteration, we will be creating new ones
        logView.value?.clear();
        let compiledStyle: Stylesheet | undefined = undefined;
        let reportedWarning = false;
        try {
            function warnAboutRecoveredError(err: StylesheetParseError) {
                logView.value?.addEntry([], Severity.WARNING, err.message, undefined);
                reportedWarning = true;
            }
            compiledStyle = Stylesheet.parse(newSource, warnAboutRecoveredError);
        } catch (e) {
            logView.value?.addEntry([], Severity.ERROR, String(e), undefined);
            return;
        }
        // Let the user know everything has gone well
        if (!reportedWarning) {
            logView.value?.addEntry([], Severity.INFO, 'Stylesheet compiled successfully', undefined);
        }
        // Bubble the compiled stylesheet further up
        emit('style-changed', newSource, compiledStyle);
    }

    nextTick(() => {
        if (content !== undefined) {
            stylesheetChanged(content);
        }
    });
</script>

<template>
    <div class="style-editor">
        <ScrollBox class="style-editor-console-scroll">
            <EditorConsole @input="stylesheetChanged" :content="content" />
        </ScrollBox>
        <div class="style-editor-log-wrapper">
            <LogConsole class="style-editor-log" ref="editor-log" />
        </div>
    </div>
</template>

<style>
    .style-editor {
        display: flex;
        flex-direction: column;
    }

    .style-editor > .style-editor-console-scroll {
        flex-grow: 1;
    }

    .style-editor-log-wrapper {
        margin-top: 0.25em;
        min-height: fit-content;
        overflow-x: auto;
    }

    .style-editor-log {
        min-width: fit-content;
    }
</style>
