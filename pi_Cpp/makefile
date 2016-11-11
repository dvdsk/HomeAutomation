CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14

pi: main.o MainData.o PirData.o Serial.o 
	$(CXX) $(CXXFLAGS) -o pi main.o MainData.o PirData.o Serial.o \
	-lboost_system -lboost_thread -lpthread

main.o: main.cpp Serial.h dataStorage/MainData.h dataStorage/PirData.h
	$(CXX) $(CXXFLAGS) -c main.cpp
MainData.o: dataStorage/MainData.cpp dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) -c dataStorage/MainData.cpp
PirData.o: dataStorage/PirData.cpp dataStorage/PirData.h dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) -c dataStorage/PirData.cpp
Serial.o: Serial.cpp
	$(CXX) $(CXXFLAGS) -c Serial.cpp	
clean:
	rm main.o MainData.o PirData.o Serial.o
