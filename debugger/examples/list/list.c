/**
 * Showcases the construction and traversal of a linked list structure.
 * 
 * Demonstrates the use of Aili for rendering pointer-based graph structures.
 * 
 * @file
 */

#include <stdlib.h>

typedef struct node {
    int value;
    struct node* next;
} node;

node* create_node(int value, node* next) {
    node* n = (node*)malloc(sizeof(*n));
    n->value = value;
    n->next = next;
}

node* create_list() {
    node* head = create_node(2, NULL);
    head = create_node(5, head);
    head = create_node(7, head);
    head = create_node(1, head);
    head = create_node(4, head);
    return head;
}

void free_list(node* head) {
    node* next;
    for (; head; head = next) {
        next = head->next;
        free(head);
    }
}

node* find_middle(node* head) {
    node* middle = head, * end = head;
    while (end && end->next) {
        middle = middle->next;
        end = end->next->next;
    }
    return middle;
}

int main(void) {
    node* head = create_list();
    node* middle = find_middle(head);
    free_list(head);
}
