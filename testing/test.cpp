#include <string>
#include <ctime>
#include <cstdint> //uint16_t
#include <iostream>
#include <thread>
#include <atomic>
#include <mutex>

enum MajorStates {
	DEFAULT_S,			
	MINIMAL_S
};

struct HttpState{
	HttpState(){updated=false;}
	~HttpState(){std::cout<<"HttpState HAS BEEN DESTORYED\n";}
	std::mutex m;
	std::string url;
	bool updated;
};

class StateData{
	public:
		StateData(HttpState* httpState_){httpState = httpState_; dataInt = 42;}
		int dataInt;	
		HttpState* httpState;
		std::atomic<MajorStates> newState;
};

class State {
	public:
		State(StateData* data_){
			data = data_;
		}
		virtual ~State() = default;
		std::atomic<MajorStates> stateName;

		bool updateOnHttp(){
			data->httpState->updated = false;
			bool updateState = true;

			std::string url = data->httpState->url;
			data->httpState->m.unlock();//unlock to indicate url has been read

			if(url == "/|state/default"){
				if(stateName != DEFAULT_S){data->newState = DEFAULT_S;}
				else{updateState=false;}		
			}
			else if(url == "/|state/minimal"){
				if(stateName != MINIMAL_S){data->newState = MINIMAL_S;}
				else{updateState=false;}		
			}
			else
				updateState=false;

			std::cout<<"updateState is returning: "<<updateState<<"\n";
			return updateState;		
		}

	protected:
		StateData* data;
};



class Default : public State
{
	public:
		Default(StateData* data_) : State(data_){stateName = DEFAULT_S; std::cout<<"created Default\n";	}
		~Default(){std::cout<<"removed Default\n";}
		
	private:
		int a;
};

class Minimal : public State
{
	public:
		Minimal(StateData* data_) : State(data_){stateName = MINIMAL_S; std::cout<<"created Minimal\n"; }
		~Minimal(){std::cout<<"removed Minimal\n";}
	private:
		int a;
};





void startNewState(State* &currentState, StateData* data){
	switch(data->newState){
		case DEFAULT_S:
		currentState = new Default(data);
		break;
		case MINIMAL_S:
		currentState = new Minimal(data);
		break;
	}
}

int main(){
	State* currentChild;
	HttpState* httpState = new HttpState;

	StateData* data = new StateData(httpState);
	currentChild = new Default(data); 

	httpState->url = "/|state/minimal";
	if(currentChild->updateOnHttp()){
		delete currentChild;

//		//>>>>>>>>>> WORKING CODE <<<<<<<<<<
//		switch(data->newState){
//			case DEFAULT_S:
//				currentChild = new Default(data);
//				break;
//			case MINIMAL_S:
//				currentChild = new Minimal(data);
//				break;
//		}		

//		>>>>>>>>>>SAME CODE IN FUNCT (CRASHES) <<<<<<<<<<<<<
		startNewState(currentChild, data); 

	}
	httpState->url = "/|state/default";
	if(currentChild->updateOnHttp()){
		delete currentChild;
		currentChild = new Default(data);
		//startNewState(currentChild, data);  
	}
//	httpState->url = "/|state/minimal";
//	if(currentChild->updateOnHttp()){
//		delete currentChild;
//		startNewState(currentChild, data); 
//	}
//	httpState->url = "/|state/default";
//	if(currentChild->updateOnHttp()){
//		delete currentChild;
//		startNewState(currentChild, data); 
//	}

	delete currentChild;
	delete data;
	delete httpState;
}







//class thread_Child : public Parant
//{
//	private:
//		int a;
//		std::atomic<bool> stop;
//		std::thread* id;


//		int get() {return a;}
//		static void* threadFunction(Default* arg) {
//			while(!arg->stop){}
//			std::cout<<"by\n";
//			return 0;}

//	public:
//		Default(){stop = false; id = new std::thread(threadFunction, this);	}
//		~Default() {stop = true; id->join();}

//};


//int main(int argc, char** argv) {
//	Parant* currentState = new Default();
//	delete currentState;
//	std::cout<<"deleted currentState 1\n";
//	currentState = new Default();
//	delete currentState;
//	std::cout<<"deleted currentState 2\n";
//	currentState = new Default();
//	delete currentState;
//	std::cout<<"deleted currentState 3\n";
//}
