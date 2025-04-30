<!--
 * View over log messages with terminal aesthetics.
-->

<script lang="ts">
    import { Severity } from 'aili-hooligan';

    /**
     * Data of a single line of log.
     */
    export type LogEntry = readonly [readonly string[], Severity, string, string | undefined];
    /**
     * How many lines of history should be kept by default.
     * 
     * History is deleted in buffers of this size,
     * so there will always be between {@link DEFAULT_HISTORY_BUFFER_SIZE}
     * and {@link DEFAULT_HISTORY_BUFFER_SIZE} `* 2` lines.
     */
    const DEFAULT_HISTORY_BUFFER_SIZE: number = 1000;
</script>

<script setup lang="ts">
    import { inject, nextTick, ref } from 'vue';
    import { INJECT_SCROLL_TO_BOTTOM } from './ScrollBox.vue';
    import Console from './Console.vue';
    import LogLine from './LogLine.vue';

    const { showTopic, history } = defineProps<{ showTopic?: boolean, history?: number }>();

    // Two swappable buffers
    // Once one of them is full, it is pushed back and the previous one is discarded
    const logs = ref<[LogEntry[], LogEntry[]]>([[], []]);

    const scrollToBottomOfContainingScrollBox = inject(INJECT_SCROLL_TO_BOTTOM, undefined);

    defineExpose({
        /**
         * Displays a new line in the log.
         * 
         * @param entry Data that describes the new line.
         */
        addEntry(...entry: LogEntry) {
            logs.value[1].push(entry);
            // Swap buffers if this one is full
            if (logs.value[1].length >= (history ?? DEFAULT_HISTORY_BUFFER_SIZE)) {
                logs.value[0] = logs.value[1];
                logs.value[1] = [];
            }
            // QOL feature: scroll to the bottom of your log console
            // when new data arrives
            nextTick(() => scrollToBottomOfContainingScrollBox?.());
        },
        /**
         * Clears the log console.
         */
        clear() {
            logs.value = [[], []];
        },
    });
</script>

<template>
    <Console>
        <template v-for="buffer of logs">
            <LogLine v-for="log of buffer"
                :topic="log[0]"
                :severity="log[1]"
                :message="log[2]"
                :showTopic="showTopic"/>
        </template>
    </Console>
</template>
