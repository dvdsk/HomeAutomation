MAKEFLAGS+="-j -l $(shell grep -c ^processor /proc/cpuinfo) "

CXX = g++-6
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

ODIR = .BUILD
OBJS = $(addprefix $(ODIR)/,main.o decode.o Cache.o MainData.o PirData.o SlowData.o Serial.o MainHeader.o  mainServer.o mainState.o telegramBot.o httpGetPostPut.o commandline.o webGraph.o plotly.o dygraphs.o mpd.o lamps.o default.o SleepInterrupt.o GoingToSleep.o Minimal.o Wakeup.o)


#these 3 lines allow 'make debug' to have debug prints and 'make' not
all: pi
debug: CXXFLAGS += -DDEBUG -g
debug: pi
nothread: CXXFLAGS += -DNOTHREAD
nothread: pi


pi: $(OBJS)
	$(CXX) $(CXXFLAGS) -o pi $(OBJS) \
	-lmicrohttpd -lcurl  -lpthread -lmenu -lncurses \
	-lboost_system -lboost_thread

$(ODIR)/main.o: main.cpp config.h arduinoContact/decode.h \
	dataStorage/MainData.h dataStorage/PirData.h dataStorage/SlowData.h \
	state/mainState.h telegramBot/telegramBot.h state/stateManagement.cpp\
	httpServer/mainServer.h arduinoContact/Serial.h commandLine/commandline.h debug.h
	$(CXX) $(CXXFLAGS) -o$@ -c main.cpp

$(ODIR)/commandline.o: commandLine/commandline.cpp commandLine/commandline.h \
	config.h state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c commandLine/commandline.cpp

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

$(ODIR)/decode.o: arduinoContact/decode.cpp arduinoContact/decode.h arduinoContact/Serial.h \
	compression.h encodingScheme.h
	$(CXX) $(CXXFLAGS) -o$@ -c arduinoContact/decode.cpp		
$(ODIR)/Serial.o: arduinoContact/Serial.cpp arduinoContact/Serial.h
	$(CXX) $(CXXFLAGS) -o$@ -c arduinoContact/Serial.cpp	

$(ODIR)/mainState.o: state/mainState.cpp state/mainState.h lamps/lamps.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/mainState.cpp	

$(ODIR)/default.o: state/majorStates/default.cpp state/majorStates/default.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/default.cpp	 
$(ODIR)/SleepInterrupt.o: state/majorStates/SleepInterrupt.cpp state/majorStates/SleepInterrupt.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/SleepInterrupt.cpp	
$(ODIR)/GoingToSleep.o: state/majorStates/GoingToSleep.cpp state/majorStates/GoingToSleep.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/GoingToSleep.cpp	
$(ODIR)/Minimal.o: state/majorStates/Minimal.cpp state/majorStates/Minimal.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/Minimal.cpp	
$(ODIR)/Wakeup.o: state/majorStates/Wakeup.cpp state/majorStates/Wakeup.h state/mainState.cpp state/mainState.h
	$(CXX) $(CXXFLAGS) -o$@ -c state/majorStates/Wakeup.cpp	

$(ODIR)/mainServer.o: httpServer/mainServer.cpp httpServer/mainServer.h \
	httpServer/pages/webGraph.h
	$(CXX) $(CXXFLAGS) -lmicrohttpd -o$@ -c httpServer/mainServer.cpp
$(ODIR)/telegramBot.o: telegramBot/telegramBot.cpp telegramBot/telegramBot.h \
	telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -o$@ -c telegramBot/telegramBot.cpp

$(ODIR)/webGraph.o: httpServer/pages/webGraph.cpp httpServer/pages/webGraph.h \
	httpServer/pages/dygraphs.h httpServer/pages/plotly.h config.h
	$(CXX) $(CXXFLAGS) -o$@ -c httpServer/pages/webGraph.cpp


$(ODIR)/httpGetPostPut.o: lamps/httpGetPostPut.cpp lamps/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -lcurl -o$@ -c lamps/httpGetPostPut.cpp	
$(ODIR)/mpd.o: mpd/mpd.cpp mpd/mpd.h
	$(CXX) $(CXXFLAGS) -o$@ -c mpd/mpd.cpp		


$(ODIR)/lamps.o: lamps/httpGetPostPut.h lamps/lamps.cpp lamps/lamps.h
	$(CXX) $(CXXFLAGS) -o$@ -c lamps/lamps.cpp

$(ODIR)/plotly.o: httpServer/pages/plotly.h httpServer/pages/plotly.cpp
	$(CXX) $(CXXFLAGS) -o$@ -c httpServer/pages/plotly.cpp

$(ODIR)/dygraphs.o: httpServer/pages/dygraphs.h httpServer/pages/dygraphs.cpp
	$(CXX) $(CXXFLAGS) -o$@ -c httpServer/pages/dygraphs.cpp


.PHONY : clean
clean:
	rm pi $(OBJS)
