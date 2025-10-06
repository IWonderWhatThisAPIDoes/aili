/**
 * Construction of a border array (array of lengths of longest
 * border of each prefix of a string).
 * 
 * See [KMP algorithm](https://en.wikipedia.org/wiki/Knuth–Morris–Pratt_algorithm).
 * 
 * @file
 */

int main(void) {
    const char str[] = "abababcacacababa";
    int b[sizeof(str)] = {0};
    for (int i = 0; i < sizeof(str) - 2; ++i) {
        int j = b[i];
        // If the border cannot be extended, backtrack
        // to the next smaller border
        while (j > 0 && str[i + 1] != str[j])
            j = b[j - 1];
        // Extend the border
        if (str[i + 1] == str[j])
            b[i + 1] = j + 1;
        // Substring does not have a non-empty border
        else
            b[i + 1] = 0;
    }
}
