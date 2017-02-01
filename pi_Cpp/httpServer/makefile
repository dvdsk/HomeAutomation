CXX = g++
CXXFLAGS= -Wall -pedantic -lpthread -std=c++14 #-pedantic

objects = mainServer.o mainState.o telegramBot.o httpGetPostPut.o testServer.o 

pi: $(objects)
	$(CXX) $(CXXFLAGS) $(ROOTFLAGS) -o pi $(objects) \
	-lmicrohttpd -lcurl

testServer.o: testServer.cpp
	$(CXX) $(CXXFLAGS) -c testServer.cpp

mainServer.o: mainServer.cpp mainServer.h
	$(CXX) $(CXXFLAGS) -lmicrohttpd -c mainServer.cpp

mainState.o: ../state/mainState.cpp ../state/mainState.h
	$(CXX) $(CXXFLAGS) -c ../state/mainState.cpp	

telegramBot.o: ../telegramBot/telegramBot.cpp ../telegramBot/telegramBot.h \
		../telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -c ../telegramBot/telegramBot.cpp

httpGetPostPut.o: ../telegramBot/httpGetPostPut.cpp ../telegramBot/httpGetPostPut.h
	$(CXX) $(CXXFLAGS) -lcurl -c ../telegramBot/httpGetPostPut.cpp	

.PHONY : clean
clean:
	rm pi $(objects)

#g++ -L/usr/lib/x86_64-linux-gnu -lmicrohttpd -Wall mainServer.cpp


