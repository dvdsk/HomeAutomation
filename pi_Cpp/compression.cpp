#include <cstdint> //uint16_t
#include <iostream>
#include <bitset>

void encode(uint8_t encoded[], uint16_t toEncode, int byte_offset, 
						int bit_offset, int length_bits){

	uint8_t unused_bits;
	uint8_t mask;
	//=total bits - (needed bits - (bits in first byte))
	unused_bits = 8-(length_bits - (8-bit_offset));
	mask = (~0);//TODO shift here doesnt work for some reason
	mask = mask >> unused_bits;

	encoded[byte_offset] 		|= uint8_t(toEncode << bit_offset);
	encoded[byte_offset+1] 	|= uint8_t(toEncode >> (8-bit_offset)) & mask;
	
	std::cout<<+unused_bits<<"\n";
	std::bitset<8> x(mask);
	std::cout<<"mask: "<<x<<"\n";
	std::cout<<"uint8_t(toEncode >> (8-bit_offset)): "<<+uint8_t(toEncode >> (8-bit_offset))<<"\n";
	std::cout<<"encoded[byte_offset]: "<<+encoded[byte_offset+1]<<"\n";
}

uint16_t decode(uint8_t encoded[], int byte_offset, 
						int bit_offset, int length_bits){

	uint16_t decoded;
	uint8_t unused_bits;
	uint8_t mask;

	//=total bits - (needed bits - (bits in first byte))
	unused_bits = 8-(length_bits - (8-bit_offset));
	mask = (~0);//TODO shift here doesnt work for some reason
	mask = mask >> unused_bits;

	std::cout<<+unused_bits<<"\n";
	std::bitset<8> x(mask);
	std::cout<<"mask: "<<x<<"\n";

	decoded = ((uint16_t)encoded[byte_offset] >> bit_offset ) |
						((uint16_t)(encoded[byte_offset+1] & mask) << (8-bit_offset) );

//	std::cout<<"(uint16_t)(encoded[byte_offset] >> bit_offset): "<<(uint16_t)(encoded[byte_offset] >> bit_offset)<<"\n";
//	std::cout<<"(uint16_t)(encoded[byte_offset+1] << (8-bit_offset)) & mask: "
//	<< encoded[byte_offset+1] << (8-bit_offset)) <<"\n";

	return decoded;
}

int main(){
	
	uint16_t test = 0;
	uint8_t encoded[10] = {0,0,0,0,0,0,0,0,0,0};

	std::cout<<"unencoded value:"<<test<<"\n";	
	encode(encoded, test, 2, 4, 10);		

	for(int i=0; i<10; i++){
		//std::bitset<8> x(encoded[i]);	
		std::cout<<"raw bit encoded[i]: "<<+encoded[i]<<"\n";
	}

	std::cout<<"decoded value:"<<decode(encoded, 2, 4, 10)<<"\n";


}
