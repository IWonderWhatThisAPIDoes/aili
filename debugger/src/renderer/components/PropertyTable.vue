<!--
 * Table that displays the resolved style applied to the program state.
-->

<script setup lang="ts">
    import { PropertyMap } from 'aili-jsapi';

    defineProps<{ mapping: PropertyMap[] }>();

    /**
     * Constructs an HTML anchor key from an ID of a selectable entity.
     *
     * @param nodeId Selectable entity ID.
     * @return HTML anchor corresponding to the entity.
     */
    function htmlAnchorFromNodeId(nodeId: string): string {
        return `prop-map-${encodeURIComponent(nodeId)}`;
    }
</script>

<template>
    <table class="prop-mapping">
        <template v-for="props in mapping" :key="props.nodeId">
            <tr
                v-for="(prop, i) in props.properties"
                :key="`${prop.keyType}-${prop.attributeName}-${prop.fragmentKey}`"
                :class="{
                    'prop-last-row': i == props.properties.length - 1,
                    'prop-attribute': prop.keyType === 'attr',
                    'prop-display': prop.keyType === 'display',
                    'prop-reference': prop.keyType === 'parent' || prop.keyType === 'target',
                }"
            >
                <!-- Node ID column -->
                <td
                    v-if="i == 0"
                    :rowspan="props.properties.length"
                    class="prop-col-node-id"
                    :id="htmlAnchorFromNodeId(props.nodeId)"
                >
                    {{ props.nodeId }}
                </td>
                <!-- Property key column -->
                <td class="prop-col-key">
                    <span v-if="prop.fragmentKey !== undefined" class="prop-key-fragment">
                        {{ prop.fragmentKey }}/</span
                    >{{ prop.keyType === 'attr' ? prop.attributeName : prop.keyType }}
                </td>
                <!-- Property value column -->
                <td class="prop-col-value">
                    <a
                        v-if="prop.keyType === 'parent' || prop.keyType === 'target'"
                        :href="'#' + htmlAnchorFromNodeId(prop.value)"
                    >
                        {{ prop.value }}
                    </a>
                    <template v-else>
                        {{ prop.value }}
                    </template>
                </td>
            </tr>
        </template>
    </table>
</template>

<style>
    table.prop-mapping {
        border: 1px solid #544;
        border-spacing: 0;
    }

    table.prop-mapping td {
        min-width: 100px;
        white-space: nowrap;
        padding: 0.1em 0.5em;
        border-top: 1px solid #f0e8e8;
    }

    table.prop-mapping td:not(:last-of-type) {
        border-right: 1px solid #807474;
    }

    table.prop-mapping td {
        border-bottom: 1px solid #807474;
    }

    table.prop-mapping td.prop-col-node-id,
    table.prop-mapping tr.prop-last-row > td {
        border-bottom: 1px solid #544;
    }

    table.prop-mapping td.prop-col-node-id {
        font-weight: bolder;
    }

    tr.prop-display > td.prop-col-key,
    tr.prop-display > td.prop-col-value,
    tr.prop-reference > td.prop-col-key {
        color: navy;
    }

    tr.prop-attribute > td.prop-col-key {
        color: maroon;
    }

    .prop-key-fragment {
        color: rgb(187, 109, 109);
    }
</style>
