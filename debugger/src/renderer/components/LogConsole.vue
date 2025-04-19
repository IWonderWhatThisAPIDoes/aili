<!--
 * View over log messages with terminal aesthetics.
-->

<script lang="ts">
    import { Severity } from 'aili-hooligan';

    /**
     * Data of a single line of log.
     */
     export type LogEntry = readonly [readonly string[], Severity, string, string | undefined];
</script>

<script setup lang="ts">
    import { inject, nextTick, ref } from 'vue';
    import { INJECT_SCROLL_TO_BOTTOM } from './ScrollBox.vue';
    import Console from './Console.vue';
    import LogLine from './LogLine.vue';

    const logs = ref<LogEntry[]>([]);

    const scrollToBottomOfContainingScrollBox = inject(INJECT_SCROLL_TO_BOTTOM, undefined);

    defineExpose({
        /**
         * Displays a new line in the log.
         * 
         * @param entry Data that describes the new line.
         */
        addEntry(...entry: LogEntry) {
            logs.value.push(entry);
            // QOL feature: scroll to the bottom of your log console
            // when new data arrives
            nextTick(() => scrollToBottomOfContainingScrollBox?.());
        },
        /**
         * Clears the log console.
         */
        clear() {
            logs.value = [];
        },
    });
</script>

<template>
    <Console>
        <LogLine v-for="log of logs" :topic="log[0]" :severity="log[1]" :message="log[2]" />
    </Console>
</template>
