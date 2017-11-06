CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

SOURCES=createStringHeader.cpp
OBJECTS=$(SOURCES:.cpp=.o)
EXECUTABLE=../pi_Cpp/httpServer/pages/readIn
	
all: $(SOURCES) $(EXECUTABLE)
	
$(EXECUTABLE): $(OBJECTS) 
	$(CXX) $(OBJECTS) $(CXXFLAGS) -o $@

.cpp.o:
	$(CXX) -c $(CXXFLAGS) $< -o $@

clean:
	rm -rf *o $(EXECUTABLE)
