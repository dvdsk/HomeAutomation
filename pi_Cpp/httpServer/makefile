CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic

objects = mainServer.o state.o telegramBot.o httpGetPostPut.o

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o pi $(objects) \
	-lmicrohttpd -lcurl

mainServer.o: mainServer.cpp ../state/mainState.cpp ../state/mainState.h
	$(CXX) $(CXXFLAGS) -lmicrohttpd -c mainServer.cpp

mainState.o: ../mainState.cpp ../state/mainState.h
	$(CXX) $(CXXFLAGS) -c ../mainState.cpp	

telegramBot.o: ../telegramBot/telegramBot.cpp ../telegramBot/telegramBot.h \
		../telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -c ../telegramBot/telegramBot.cpp

httpGetPostPut.o: ../telegramBot/httpGetPostPut.cpp ../telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -lcurl -c ../telegramBot/httpGetPostPut.cpp	



#g++ -L/usr/lib/x86_64-linux-gnu -lmicrohttpd -Wall mainServer.cpp


