/**
 * Definition of the default sample graph
 * on which stylesheets are showcased.
 * 
 * @module
 */

import { EdgeLabel, StateGraph, StateNode, NodeTypeClass } from 'aili-jsapi';

/**
 * Keys of built-in sample graphs for demonstration.
 */
export enum SampleGraph {
    /**
     * Program that works with a simple implementation of a vector.
     */
    VECTOR_APP,
    /**
     * Function-programming-style recursive approach
     * to splitting a linked list in half.
     */
    LIST_SPLIT,
    /**
     * The default option.
     */
    DEFAULT = VECTOR_APP,
}

/**
 * Display names of sample graphs.
 */
export const SAMPLE_GRAPH_NAMES: Record<SampleGraph, string> = {
    [SampleGraph.VECTOR_APP]: 'Vector',
    [SampleGraph.LIST_SPLIT]: 'Linked list',
}

/**
 * Constructs a sample graph by its key.
 * 
 * @param key Identifier of the graph to construct.
 */
export function createSampleGraph(key: SampleGraph): StateGraph {
    return SAMPLE_GRAPH_CONSTRUCTORS[key]();
}

/**
 * Constructs the graph of a sample application with a vector.
 * 
 * @returns New sample graph.
 */
function vectorApp(): StateGraph {
    const v: StateNode = {
        typeKind: NodeTypeClass.Struct,
        typeName: 'vector',
        outEdges: [
            {
                edgeLabel: EdgeLabel.named('ptr'),
                node: {
                    typeKind: NodeTypeClass.Ref,
                    outEdges: [
                        {
                            edgeLabel: EdgeLabel.DEREF,
                            node: {
                                typeKind: NodeTypeClass.Array,
                                outEdges: [
                                    {
                                        edgeLabel: EdgeLabel.LENGTH,
                                        node: {
                                            typeKind: NodeTypeClass.Atom,
                                            typeName: 'int',
                                            value: BigInt(8),
                                        }
                                    },
                                    ...[2, 3, 5, 8, 13, 0xffff, 0xffff, 0xffff].map((a, i) => {
                                        return {
                                            edgeLabel: EdgeLabel.index(i),
                                            node: {
                                                typeKind: NodeTypeClass.Atom,
                                                typeName: 'int',
                                                value: BigInt(a),
                                            }
                                        };
                                    })
                                ]
                            }
                        }
                    ]
                }
            },
            {
                edgeLabel: EdgeLabel.named('len'),
                node: {
                    typeKind: NodeTypeClass.Atom,
                    typeName: 'int',
                    value: BigInt(5),
                }
            },
            {
                edgeLabel: EdgeLabel.named('cap'),
                node: {
                    typeKind: NodeTypeClass.Atom,
                    typeName: 'int',
                    value: BigInt(8),
                }
            }
        ]
    };
    return new StateGraph({
        typeKind: NodeTypeClass.Root,
        outEdges: [
            {
                edgeLabel: EdgeLabel.MAIN,
                node: {
                    typeKind: NodeTypeClass.Frame,
                    typeName: 'main',
                    outEdges: [
                        {
                            edgeLabel: EdgeLabel.NEXT,
                            node: {
                                typeKind: NodeTypeClass.Frame,
                                typeName: 'push',
                                outEdges: [
                                    {
                                        edgeLabel: EdgeLabel.named('vec'),
                                        node: {
                                            typeKind: NodeTypeClass.Ref,
                                            outEdges: [
                                                {
                                                    edgeLabel: EdgeLabel.DEREF,
                                                    node: v,
                                                }
                                            ]
                                        },
                                    },
                                    {
                                        edgeLabel: EdgeLabel.named('item'),
                                        node: {
                                            typeKind: NodeTypeClass.Atom,
                                            typeName: 'int',
                                            value: BigInt(42)
                                        }
                                    },
                                ]
                            }
                        },
                        {
                            edgeLabel: EdgeLabel.named('vec'),
                            node: v
                        },
                        {
                            edgeLabel: EdgeLabel.named('argv'),
                            node: {
                                typeKind: NodeTypeClass.Ref,
                                outEdges: [
                                    {
                                        edgeLabel: EdgeLabel.DEREF,
                                        node: {
                                            typeKind: NodeTypeClass.Array,
                                            outEdges: [
                                                {
                                                    edgeLabel: EdgeLabel.LENGTH,
                                                    node: {
                                                        typeKind: NodeTypeClass.Atom,
                                                        typeName: 'int',
                                                        value: BigInt(2),
                                                    }
                                                },
                                                ...['a.out', '-h'].map((s: string, i: number) => {
                                                    return {
                                                        edgeLabel: EdgeLabel.index(i),
                                                        node: {
                                                            typeKind: NodeTypeClass.Ref,
                                                            outEdges: [
                                                                {
                                                                    edgeLabel: EdgeLabel.DEREF,
                                                                    node: {
                                                                        typeKind: NodeTypeClass.Array,
                                                                        outEdges: [
                                                                            {
                                                                                edgeLabel: EdgeLabel.LENGTH,
                                                                                node: {
                                                                                    typeKind: NodeTypeClass.Atom,
                                                                                    typeName: 'int',
                                                                                    value: BigInt(s.length),
                                                                                }
                                                                            },
                                                                            ...[].map.call(s, (c: string, i: number) => {
                                                                                return {
                                                                                    edgeLabel: EdgeLabel.index(i),
                                                                                    node: {
                                                                                        typeKind: NodeTypeClass.Atom,
                                                                                        typeName: 'char',
                                                                                        value: BigInt(c.charCodeAt(0)),
                                                                                    }
                                                                                }
                                                                            })
                                                                        ]
                                                                    }
                                                                }
                                                            ]
                                                        }
                                                    };
                                                })
                                            ]
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        ]
    });
}

function listSplit(): StateGraph {
    const LENGTH: number = 6;
    const listNodes: StateNode[] = Array.from({ length: LENGTH }, (_, i) => {
        return {
            typeKind: NodeTypeClass.Struct,
            typeName: 'node',
            outEdges: [
                {
                    edgeLabel: EdgeLabel.named('value'),
                    node: {
                        typeKind: NodeTypeClass.Atom,
                        typeName: 'int',
                        value: BigInt(Math.floor(Math.random() * 100)),
                    },
                },
            ],
        };
    });
    listNodes.slice(1).forEach((node, i) => listNodes[i].outEdges?.push({
        edgeLabel: EdgeLabel.named('next'),
        node: {
            typeKind: NodeTypeClass.Ref,
            outEdges: [
                {
                    edgeLabel: EdgeLabel.DEREF,
                    node,
                },
            ],
        },
    }));
    const stackFrames: StateNode[] = Array.from({ length: LENGTH / 2 }, (_, i) => {
        return {
            typeKind: NodeTypeClass.Frame,
            typeName: 'split',
            outEdges: [
                {
                    edgeLabel: EdgeLabel.named('turtle'),
                    node: {
                        typeKind: NodeTypeClass.Ref,
                        outEdges: [
                            {
                                edgeLabel: EdgeLabel.DEREF,
                                node: listNodes[i],
                            },
                        ],
                    },
                },
                {
                    edgeLabel: EdgeLabel.named('hare'),
                    node: {
                        typeKind: NodeTypeClass.Ref,
                        outEdges: [
                            {
                                edgeLabel: EdgeLabel.DEREF,
                                node: listNodes[i * 2 + 1],
                            },
                        ],
                    },
                },
            ],
        };
    });
    listNodes[LENGTH - 1].outEdges?.push({
        edgeLabel: EdgeLabel.named('next'),
        node: {
            typeKind: NodeTypeClass.Atom,
            typeName: 'nullptr',
        },
    });
    stackFrames.slice(1).forEach((node, i) => stackFrames[i].outEdges?.push({
        edgeLabel: EdgeLabel.NEXT,
        node,
    }));
    return new StateGraph({
        typeKind: NodeTypeClass.Root,
        outEdges: [
            {
                edgeLabel: EdgeLabel.MAIN,
                node: {
                    typeKind: NodeTypeClass.Frame,
                    typeName: 'main',
                    outEdges: [
                        {
                            edgeLabel: EdgeLabel.named('list'),
                            node: {
                                typeKind: NodeTypeClass.Struct,
                                typeName: 'list',
                                outEdges: [
                                    {
                                        edgeLabel: EdgeLabel.named('head'),
                                        node: {
                                            typeKind: NodeTypeClass.Ref,
                                            outEdges: [
                                                {
                                                    edgeLabel: EdgeLabel.DEREF,
                                                    node: listNodes[0],
                                                },
                                            ],
                                        },
                                    },
                                ],
                            },
                        },
                        {
                            edgeLabel: EdgeLabel.NEXT,
                            node: stackFrames[0],
                        },
                    ],
                },
            },
        ],
    });
}

const SAMPLE_GRAPH_CONSTRUCTORS: Record<SampleGraph, () => StateGraph> = {
    [SampleGraph.VECTOR_APP]: vectorApp,
    [SampleGraph.LIST_SPLIT]: listSplit,
}
