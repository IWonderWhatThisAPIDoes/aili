<script lang="ts" setup>
    import { ref, useTemplateRef } from 'vue';
    import { LengthHintSheet } from 'aili-jsapi';
    import { DebugSessionManager } from '../controllers/session-manager';
    import { DebugSessionStatus } from '../controllers/session';
    import { DEFAULT_HINT_SHEET } from '../utils/default-stylesheet';
    import StyleEditor from './StyleEditor.vue';

    const { session } = defineProps<{ session: DebugSessionManager }>();
    const isHintEditorDirty = ref(false);
    const isSessionActive = ref(false);
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

    function hintSheetChanged(sheet: LengthHintSheet) {
        session.hintSheet = sheet;
        if (session.status !== DebugSessionStatus.INACTIVE) {
            isHintEditorDirty.value = true;
        }
    }

    session.onStatusChanged.hook(status => {
        isSessionActive.value = status !== DebugSessionStatus.INACTIVE;
        if (!isSessionActive.value) {
            isHintEditorDirty.value = false;
        }
    });
</script>

<template>
    <div class="debuggee-controls">
        <label class="debuggee-path">
            Debuggee command line:
            <input
                @change="commandLineChanged"
                :disabled="isSessionActive"
                placeholder="a.out --help"
                ref="command-line"
            />
        </label>
        <div v-if="isHintEditorDirty" class="debuggee-dirty-notice">
            Hints have been modified; Restart debug session for changes to take effect.
        </div>
        <StyleEditor
            class="debuggee-style"
            :content="DEFAULT_HINT_SHEET"
            :compile="LengthHintSheet.parse"
            @style-changed="(_, s) => hintSheetChanged(s)"
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
        min-height: 0;
    }

    .debuggee-dirty-notice {
        border: 1px solid #544;
        background-color: khaki;
        padding: 0.25em;
    }
</style>
