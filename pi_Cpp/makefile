all: 
	g++ -Wall -std=c++11 Serial.cpp dataStorage/PirData.cpp dataStorage/MainData.cpp main.cpp -lboost_system -lboost_thread -lpthread -o pi_Cpp

#CXX = g++
#CXXFLAGS=-g -Wall -pedantic

#pi_Cpp: main.o StoreData.o Serial.o
#	$(CXX) $(CXXFLAGS) -o expressie StoreData.o Serial.o main.o #"-o expressie" needed
#main.o: main.cpp
#	$(CXX) $(CXXFLAGS) -c main.cpp
#StoreData.o: StoreData.cpp
#	$(CXX) $(CXXFLAGS) -c StoreData.cpp
#Serial.o: Serial.cpp
#	$(CXX) $(CXXFLAGS) -c Serial.cpp	
#clean:
#	rm Boom.o main.o

#http://www.gnu.org/software/make/manual/make.html#Introduction

