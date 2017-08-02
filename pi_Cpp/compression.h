#ifndef COMPR_H 
#define COMPR_H 

#if not ARDUINO >= 100
#include <cstring> //memset
#include <stdint.h>
#endif

//TODO make constexpr as soon as supported by c++
inline uint8_t mask1(const int bit_offset, const int length_bits){

	uint8_t unused_bits;
	uint8_t maski;
	//=total bits - (needed bits - (bits in first byte))	
	unused_bits = 8-(length_bits - (8-bit_offset));
	if(unused_bits>8){unused_bits=0;}
	maski = (~0);
	maski = maski >> unused_bits;

	return maski;
}

//TODO make constexpr as soon as supported by c++
inline uint8_t mask2(const int bit_offset, const int length_bits){

	uint8_t unused_bits;
	uint8_t maski;
	//=total bits - (needed bits - (bits in first and second byte))
	unused_bits = 8-(length_bits - (8-bit_offset +8));
	maski = (~0);
	maski = maski >> unused_bits;

	return maski;
}

//function that does the actual work
inline void encode2(uint8_t encoded[], uint16_t toEncode, const int byte_offset, 
									 const int bit_offset, const int length_bits){

	//TODO check if masks actually needed here (shouldnt as we shift in 0's right?)
	encoded[byte_offset] 		|= uint8_t(toEncode << bit_offset);
	encoded[byte_offset+1] 	|= uint8_t(toEncode >> (8-bit_offset)) & mask1(bit_offset, length_bits);
	encoded[byte_offset+2] 	|= uint8_t(toEncode >> (16-bit_offset)) & mask2(bit_offset, length_bits);
}

//with in memory offset
inline void encode(uint8_t encoded[], uint16_t toEncode, const int memory_offset_bytes, 
						 			  const int package_offset_bits, const int length_bits){

	int byte_offset = memory_offset_bytes+package_offset_bits/8;
	int bit_offset = package_offset_bits%8;

	encode2(encoded, toEncode, byte_offset, bit_offset, length_bits);
}

//without in memory offset
inline void encode(uint8_t encoded[], uint16_t toEncode, const int package_offset_bits, 
						 				const int length_bits){

	int byte_offset = package_offset_bits/8;
	int bit_offset = package_offset_bits%8;

	encode2(encoded, toEncode, byte_offset, bit_offset, length_bits);
}

//function that does the actual work
inline uint16_t decode2(const uint8_t encoded[], int byte_offset, 
											 int bit_offset, int length_bits){

	uint16_t decoded;
	decoded = ((uint16_t)encoded[byte_offset] >> bit_offset ) |
	  ((uint16_t)(encoded[byte_offset+1] & mask1(bit_offset, length_bits)) << (8-bit_offset) );
	  ((uint16_t)(encoded[byte_offset+2] & mask2(bit_offset, length_bits)) << (16-bit_offset) );

	return decoded;
}

//with in memory offset
inline uint16_t decode(const uint8_t encoded[], const int memory_offset_bytes, 
						 		 			 const int package_offset_bits, const int length_bits){

	int byte_offset = memory_offset_bytes+package_offset_bits/8;
	int bit_offset = package_offset_bits%8;

	return decode2(encoded, byte_offset, bit_offset, length_bits);
}

//without in memory offset
inline uint16_t decode(const uint8_t encoded[], const int package_offset_bits, 
								 			 const int length_bits){

	int byte_offset = package_offset_bits/8;
	int bit_offset = package_offset_bits%8;

	return decode2(encoded, byte_offset, bit_offset, length_bits);
}

//int main(){
//	
//	uint16_t test;
//	uint8_t encoded[10] = {0,0,0,0,0,0,0,0,0,0};
//	for(uint16_t i=100; i<320; i++){
//		memset(encoded, 0, 10);
//		encode(encoded, i, 0, 9);					
//		if(i-decode(encoded, 0, 0, 9)){std::cout<<"ERROR\n";}
//		//std::cout<<"decoded value:"<<decode(encoded, 0, 0, 9)<<"\n";
//	}
//}
#endif
