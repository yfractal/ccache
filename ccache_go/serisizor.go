package main

import (
	"C"
	"bytes"
	"encoding/gob"
	"reflect"
	"fmt"
	"unsafe"
)

func encode_helper(data interface{}) ([]byte, error) {
	var buffer bytes.Buffer
	encoder := gob.NewEncoder(&buffer)
	err := encoder.Encode(data)
	if err != nil {
        fmt.Println("error: ", err)
		return nil, err
	}
	fmt.Println("encode_helper bytes", buffer.Bytes())
	return buffer.Bytes(), nil
}

//export Encode
func Encode(dataPtr unsafe.Pointer, typePtr unsafe.Pointer, size *C.int) (*C.char) {
	fmt.Printf("dataPtr %v\n", dataPtr)
	fmt.Printf("typePtr %v\n", typePtr)

	retrievedType := *(*reflect.Type)(typePtr)
	val := reflect.NewAt(retrievedType, dataPtr).Elem()
	any := val.Interface()
	encodedBytes, err := encode_helper(any)

	if err != nil {
		fmt.Println("errr: ", err)
		return nil
	}

	*size = C.int(len(encodedBytes))
	encodedPtr := C.CBytes(encodedBytes)
	return (*C.char)(encodedPtr)
}