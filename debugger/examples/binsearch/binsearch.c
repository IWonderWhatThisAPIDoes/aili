/**
 * Showcases a binary search over an array.
 * 
 * Demonstrates the use of Aili for rendering arrays
 * and labels that point to array elements.
 * 
 * @file
 */

int main(void) {
    int array[] = {1, 3, 4, 5, 7, 9, 10, 11, 14, 16, 17, 18, 20, 21, 22, 24, 25, 28, 29, 33, 34, 35, 37, 40, 41, 43, 46, 47, 49, 50};
    int find = 21, min = 0, max = sizeof(array) / sizeof(array[0]);
    while (min < max) {
        int mid = (min + max) / 2;
        if (array[mid] > find)
            max = mid;
        else if (array[mid] < find)
            min = mid + 1;
        else
            break;
    }
}
