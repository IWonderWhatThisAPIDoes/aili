/**
 * Showcases a breadth-first traversal of a graph structure.
 * 
 * Demonstrates the use of Aili for rendering graphs.
 * 
 * @file
 */

#include <stdlib.h>
#include <stdarg.h>

typedef struct node {
    size_t* outEdges;
    size_t nOutEdges;
    int state;
} node;

node make_node(size_t outEdges, ...) {
    node n = {
        malloc(outEdges * sizeof(size_t)),
        outEdges,
    };
    va_list args;
    va_start(args, outEdges);
    for (size_t i = 0; i < outEdges; ++i) {
        n.outEdges[i] = va_arg(args, size_t);
    }
    va_end(args);
    return n;
}

void bfs(node* graph, size_t nodeCount, size_t origin) {
    if (!nodeCount) return;
    // Making the queue this long is overkill,
    // but for the sake of simple demonstration...
    size_t* q = malloc(nodeCount * sizeof(*q));
    q[0] = origin;
    graph[0].state = 1;
    size_t qstart = 1, qend = 0;
    while (qstart != qend) {
        size_t current = q[qend];
        for (size_t i = 0; i < graph[current].nOutEdges; ++i) {
            size_t next = graph[current].outEdges[i];
            if (graph[next].state == 0) {
                graph[next].state = 1;
                q[qstart++] = next;
            }
        }
        ++qend;
        graph[current].state = 2;
    }
}

int main(void) {
    node graph[] = {
        make_node(2, 1, 6),
        make_node(2, 2, 5),
        make_node(2, 5, 10),
        make_node(3, 1, 11, 6),
        make_node(1, 2),
        make_node(2, 0, 8),
        make_node(3, 1, 7, 4),
        make_node(1, 4),
        make_node(2, 7, 9),
        make_node(2, 0, 11),
        make_node(1, 3),
        make_node(1, 8),
    };
    bfs(graph, sizeof(graph) / sizeof(graph[0]), 0);
}
