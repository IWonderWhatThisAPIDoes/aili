/**
 * Showcases sorting of an array with
 * [bubble sort](https://en.wikipedia.org/wiki/Bubble_sort).
 * 
 * Demonstrates the use of Aili for rendering arrays
 * and labels that point to array elements.
 * 
 * @file
 */

int main(void) {
    int arr[10] = { 5, 1, 2, 8, 3, 6, 10, 7, 9, 4 };
    int dirty = 1, i, j, t;
    for (i = 10; i > 0 && dirty; --i) {
        dirty = 0;
        for (j = 1; j < i; ++j) {
            if (arr[j] < arr[j - 1]) {
                t = arr[j];
                arr[j] = arr[j - 1];
                arr[j - 1] = t;
                dirty = 1;
            }
        }
    }
}
