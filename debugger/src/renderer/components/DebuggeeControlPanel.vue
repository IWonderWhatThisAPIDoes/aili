<script lang="ts" setup>
    import { LengthHintSheet } from 'aili-jsapi';
    import { DebugSessionManager } from '../controllers/session-manager';
    import { DebugSessionStatus } from '../controllers/session';
    import { DEFAULT_HINT_SHEET } from '../utils/default-stylesheet';
    import StyleEditor from './StyleEditor.vue';

    const { session } = defineProps<{ session: DebugSessionManager }>();
</script>

<template>
    <div class="debuggee-controls">
        <label class="debuggee-path">
            Path to debuggee:
            <input
                v-model="session.pathToDebuggee"
                :disabled="session.status !== DebugSessionStatus.INACTIVE"
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
