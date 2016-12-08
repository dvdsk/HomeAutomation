#root files are located in: /usr/lib64/root (compiled libs) and
#/usr/include/root (h files)

CXX = g++
CXXFLAGS = -Wall -pedantic -std=c++14

#ROOT = -I/usr/include/root -L/usr/lib64/root -lGpad -lGraf -lHist -lCore
ROOT = `root-config --cflags --glibs`
#ROOT = -I/usr/include/root -L/usr/lib64/root/libGpad.so -L/usr/lib64/root/libGraf.so \
#-L/usr/lib64/root/libHist.so -L/usr/lib64/root/libCore.so

objects = test.o

all: pi

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOT) -o pi $(objects)

test.o: test.cpp 
	$(CXX) $(CXXFLAGS) $(ROOT) -c test.cpp
	
.PHONY : clean
clean:
	rm pi $(objects)
