MAKEFLAGS+="-j -l $(shell grep -c ^processor /proc/cpuinfo) "

#CXX = g++-6
CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

LDFLAGS = -L/usr/local/ssl/lib
LDLIBS = -lssl -lcrypto

ODIR = .BUILD
OBJS = $(addprefix $(ODIR)/,main.o Cache.o MainData.o PirData.o SlowData.o MainHeader.o)
OBJS2 = $(addprefix $(ODIR)/,main2.o Cache.o MainData.o PirData.o SlowData.o MainHeader.o)

#these 3 lines allow 'make debug' to have debug prints and 'make' not
all: fill export
debug: CXXFLAGS += -DDEBUG -g
debug: fill export
nothread: CXXFLAGS += -DNOTHREAD
nothread: fill export


fill: $(OBJS)
	$(CXX) $(CXXFLAGS) -o fill $(OBJS) \
  -lpthread -lmenu \
	-lboost_system -lboost_thread $(LDFLAGS) $(LDLIBS)

export: $(OBJS2)
	$(CXX) $(CXXFLAGS) -o export $(OBJS2) \
  -lpthread -lmenu \
	-lboost_system -lboost_thread $(LDFLAGS) $(LDLIBS)

$(ODIR)/main.o: main.cpp config.h \
	dataStorage/MainData.h dataStorage/PirData.h dataStorage/SlowData.h
	$(CXX) $(CXXFLAGS) -o$@ -c main.cpp

$(ODIR)/main2.o: main2.cpp config.h \
	dataStorage/MainData.h dataStorage/PirData.h dataStorage/SlowData.h
	$(CXX) $(CXXFLAGS) -o$@ -c main2.cpp

$(ODIR)/Cache.o: dataStorage/Cache.cpp dataStorage/Cache.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/Cache.cpp
$(ODIR)/MainHeader.o: dataStorage/MainHeader.cpp dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/MainHeader.cpp	
$(ODIR)/MainData.o: dataStorage/MainData.cpp dataStorage/MainData.h dataStorage/Cache.h \
  dataStorage/MainHeader.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/MainData.cpp

$(ODIR)/PirData.o: dataStorage/PirData.cpp dataStorage/PirData.h dataStorage/MainData.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/PirData.cpp
$(ODIR)/SlowData.o: dataStorage/SlowData.cpp dataStorage/SlowData.h dataStorage/MainData.h \
	compression.h compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) -o$@ -c dataStorage/SlowData.cpp



.PHONY : clean
clean:
	rm homeAutomation $(OBJS)
