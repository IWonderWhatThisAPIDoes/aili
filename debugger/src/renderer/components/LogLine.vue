<!--
 * Single log line displayed by log views.
-->

<script lang="ts">
    import { Severity } from 'aili-hooligan';
    import { colorFromString } from '../utils/auto-colors';

    const SEVERITY_CLASS: Readonly<Record<Severity, string>> = {
        [Severity.ERROR]: 'error',
        [Severity.WARNING]: 'warning',
        [Severity.INFO]: 'info',
        [Severity.DEBUG]: 'debug',
    };

    function severityClass(severity: Severity): string {
        return `severity-${SEVERITY_CLASS[severity]}`;
    }

    function topicClasses(topicList: readonly string[] | undefined): string[] {
        return (topicList ?? []).filter(t => /^[a-z\-]+/.test(t)).map(t => `topic-${t}`);
    }
</script>

<script setup lang="ts">
    defineProps<{
        severity: Severity;
        message: string;
        topic?: readonly string[];
        showTopic?: boolean;
    }>();
</script>

<template>
    <div :class="['log-line', severityClass(severity), ...topicClasses(topic)]">
        <span
            v-if="showTopic"
            v-for="topic in topic"
            :class="['log-line-topic', `topic-${topic}`]"
            :style="{ color: colorFromString(topic) }"
        >
            {{ topic }} </span
        >{{ message }}
    </div>
</template>

<style>
    .log-line::before {
        display: inline-block;
        width: 3em;
        margin-right: 0.5em;
        text-align: center;
        border: 1px solid;
    }

    .log-line.severity-error::before {
        content: 'ERROR';
        color: #f55;
    }

    .log-line.severity-warning::before {
        content: 'WARN';
        color: orange;
    }

    .log-line.severity-info::before {
        content: 'INFO';
        color: aquamarine;
    }

    .log-line.severity-debug::before {
        content: 'DEBUG';
        color: grey;
    }

    .log-line-topic {
        font-size: smaller;
        border: 1px solid;
        margin-right: 0.5em;
        pointer-events: none;
        user-select: none;
    }
</style>
