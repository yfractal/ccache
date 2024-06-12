// example.c
#include <stdio.h>

// extern char* CallEncode(void* p1, void* p2, int* size);

void greet() {
    printf("Hello from c...............");
}

// int encode(void* dataPtr, void* typePtr) {
//     int size;

//     char *encodedBytes = CallEncode(dataPtr, typePtr, &size);

//     if (encodedBytes == NULL) {
//         fprintf(stderr, "Error encoding the person.\n");
//         return 1;
//     }

//     // Print the encoded bytes
//     printf("Encoded bytes (size %d): \n from go: ", size);

//     for (int i = 0; i < size; i++) {
//         printf("%d ", (unsigned char)encodedBytes[i]);
//     }

//     printf("......\n");
//     return -1024;
// }