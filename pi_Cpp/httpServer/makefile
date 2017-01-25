#root files are located in: /usr/lib64/root (compiled libs) and
#/usr/include/root (h files)

CXX = g++
CXXFLAGS = -Wall -pedantic -std=c++14 -lmicrohttpd

objects = test.o

all: pi

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOT) -o pi $(objects)

test.o: test.cpp 
	$(CXX) $(CXXFLAGS) $(ROOT) -c test.cpp
	
.PHONY : clean
clean:
	rm pi $(objects)
