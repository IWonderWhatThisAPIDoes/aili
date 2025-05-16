/**
 * Showcases the insertion and removal from a naive
 * [vector](https://en.cppreference.com/w/cpp/container/vector)
 * (dynamically-sized array) structure.
 * 
 * Demonstrates the use of Aili with dynamically allocated arrays
 * whose length can change.
 * 
 * @file
 */

#include <stdlib.h>

typedef struct vector {
    size_t len;
    size_t cap;
    int* ptr;
} vector;

vector vector_init(void) {
    vector v = { 0, 0, NULL };
    return v;
}

void vector_reserve(vector* v, size_t cap) {
    if (v->cap >= cap)
        return;
    int* p = malloc(cap * sizeof(int));
    for (size_t i = 0; i < v->len; ++i)
        p[i] = v->ptr[i];
    if (v->ptr)
        free(v->ptr);
    v->cap = cap;
    v->ptr = p;
}

void vector_push(vector* v, int value) {
    if (v->len == v->cap)
        vector_reserve(v, v->cap * 2 + 1);
    v->ptr[v->len++] = value;
}

void vector_pop(vector* v) {
    if (v->len)
        --v->len;
}

void vector_delete(vector* v) {
    if (v->ptr)
        free(v->ptr);
}

int main(void) {
    vector v = vector_init();
    vector_push(&v, 1);
    vector_push(&v, 4);
    vector_push(&v, 2);
    vector_pop(&v);
    vector_push(&v, 2);
    vector_push(&v, 7);
    vector_push(&v, 4);
    vector_push(&v, 5);
    vector_pop(&v);
    vector_pop(&v);
    vector_delete(&v);
}
