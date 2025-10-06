<script lang="ts" setup>
    import { useTemplateRef } from 'vue';
    import { LengthHintSheet } from 'aili-jsapi';
    import { DebugSessionManager } from '../controllers/session-manager';
    import { DebugSessionStatus } from '../controllers/session';
    import { DEFAULT_HINT_SHEET } from '../utils/default-stylesheet';
    import StyleEditor from './StyleEditor.vue';

    const { session } = defineProps<{ session: DebugSessionManager }>();
    const commandLineInput = useTemplateRef('command-line');

    function commandLineChanged() {
        const commandLine = commandLineInput.value?.value ?? '';
        let firstSpace = commandLine.indexOf(' ');
        if (firstSpace == -1) {
            session.pathToDebuggee = commandLine;
        } else {
            session.pathToDebuggee = commandLine.substring(0, firstSpace);
            session.argumentsToDebuggee = commandLine.substring(firstSpace + 1);
        }
    }
</script>

<template>
    <div class="debuggee-controls">
        <label class="debuggee-path">
            Debuggee command line:
            <input
                @change="commandLineChanged"
                :disabled="session.status !== DebugSessionStatus.INACTIVE"
                placeholder="a.out --help"
                ref="command-line"
            />
        </label>
        <StyleEditor
            class="debuggee-style"
            :content="DEFAULT_HINT_SHEET"
            :compile="LengthHintSheet.parse"
            @style-changed="(_, s) => (session.hintSheet = s)"
        />
    </div>
</template>

<style>
    .debuggee-controls {
        display: flex;
        flex-direction: column;
        gap: 0.25em;
    }

    .debuggee-path {
        display: flex;
        align-items: baseline;
        gap: 0.25em;
    }

    .debuggee-path > input,
    .debuggee-style {
        flex-grow: 1;
    }
</style>
