/**
 * Implementation of the [Duval algorithm](https://en.wikipedia.org/wiki/Lyndon_word#Duval_algorithm)
 * for Lyndon factorization of a string.
 * 
 * @file
 */

int main(void) {
    const char str[] = "hello world world abc";
    int w[sizeof(str) / sizeof(*str)] = {0};
    int wcount = 0;
    for (int i = 0; i < sizeof(str);) {
        int j = i + 1, k = i;
        for (; j < sizeof(str) && str[k] <= str[j]; ++j) {
            if (str[k] < str[j])
                k = i;
            else
                ++k;
        }
        while (i <= k) {
            for (int l = 0; l < j - k; ++l)
                w[i + l] = wcount;
            ++wcount;
            i += j - k;
        }
    }
}
