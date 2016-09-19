#include <iostream>
#include "Serial.h"

int main(int argc, char* argv[])
{
    try {

        Serial arduino("/dev/ttyUSB0",115200);

        arduino.writeString("Hello world\n");

        std::cout << arduino.readLine() << std::endl;

    } catch(boost::system::system_error& e)
    {
        std::cout << "Error: "<< e.what() << std::endl;
        return 1;
    }
}
