/**
 * Definition of the default sample graph
 * on which stylesheets are showcased.
 * 
 * @module
 */

import { EdgeLabel, StateGraph, StateNode, NodeTypeClass } from 'aili-jsapi';

/**
 * Constructs the graph of a sample application with a vector.
 * 
 * @returns New sample graph.
 */
export function vectorApp(): StateGraph {
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
