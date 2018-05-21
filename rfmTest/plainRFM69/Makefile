MAKEFLAGS+="-j -l $(shell grep -c ^processor /proc/cpuinfo) "

CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

DEPS = plainRFM69.h bareRFM69.h

%.o: %.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

Gateway : Minimal.cpp plainRFM69.o bareRFM69.o
	$(CXX) $(CXXFLAGS) -o Minimal Minimal.cpp plainRFM69.o bareRFM69.o -pthread -lpigpio -lrt
	
	
	


