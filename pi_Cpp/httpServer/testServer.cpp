#include <thread>
#include <mutex>

#include "../telegramBot/telegramBot.h"
#include "../state/mainState.h"
#include "mainServer.h"

std::shared_ptr<std::mutex> stop = std::make_shared<std::mutex>();
std::shared_ptr<TelegramBot> bot = std::make_shared<TelegramBot>();

int main(void)
{

	(*stop).lock();
	std::thread t1(thread_Https_serv, stop, bot);

	getchar();
	(*stop).unlock();
	
	t1.join();

  return 0;
}

