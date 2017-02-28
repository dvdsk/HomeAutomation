#include <cstdint> //uint16_t
#include <iostream>
#include <bitset>

constexpr uint8_t mask(const int bit_offset, const int length_bits){

//	uint8_t unused_bits;
//	uint8_t mask;
//	//=total bits - (needed bits - (bits in first byte))
//	unused_bits = 8-(length_bits - (8-bit_offset));
//	mask = (~0);//TODO shift here doesnt work for some reason
//	mask = mask >> unused_bits;
	//implemented in single return statement below

	return (~0) >> 8-(length_bits - (8-bit_offset));
}


void encode(uint8_t encoded[], uint16_t toEncode, const int byte_offset, 
						const int bit_offset, const int length_bits){

	encoded[byte_offset] 		|= uint8_t(toEncode << bit_offset);
	encoded[byte_offset+1] 	|= uint8_t(toEncode >> (8-bit_offset)) & mask(bit_offset, length_bits);
	
}

uint16_t decode(uint8_t encoded[], int byte_offset, 
						int bit_offset, int length_bits){

	uint16_t decoded;

	decoded = ((uint16_t)encoded[byte_offset] >> bit_offset ) |
						((uint16_t)(encoded[byte_offset+1] & mask(bit_offset, length_bits)) << (8-bit_offset) );

	return decoded;
}

int main(){
	
	uint16_t test1 = 210;
	uint8_t encoded[10] = {0,0,0,0,0,0,0,0,0,0};

	std::cout<<"unencoded value:"<<test1<<"\n";	
	encode(encoded, test1, 2, 4, 10);			
	std::cout<<"decoded value:"<<decode(encoded, 2, 4, 10)<<"\n";
	decode(encoded, 2, 4, 10);
}
