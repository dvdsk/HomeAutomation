#include "mainPage.h"

void mainPage(StateData* stateData, std::string &page){
//	page.reserve(strlen(AUDIOSYS)+strlen(ROOMSTATE)+strlen(TEMP)+strlen(HUMID)
//	             +strlen(CO2)+strlen(LIGHT)+strlen(MOVEMENT)+strlen(LAST)+7*10);
	
	
	std::string switchStr;
	switch(stateData->mpdState->playback){
		case PLAYING:
		switchStr = "playing, vol: ";
		break;
		case PAUSED:
		switchStr = "paused, vol: ";
		break;
		case STOPPED:
		switchStr = "stopped, vol: ";
		break;
	}
	int volume = (float(stateData->mpdState->volume)/(255.f/100.f) *100);	
	page += AUDIOSYS;
	page += switchStr;
	page += std::to_string(volume);


	page += ROOMSTATE;
	switch(stateData->stateName){
		case DEFAULT_S:
		switchStr = "default state";
		break;
		case WAKEUP_S:
		switchStr = "wakeup state";
		break;
		case MINIMAL_S:
		switchStr = "minimal state";
		break;
	}
	page += switchStr;


	page += TEMP;
	float temp = float( (stateData->sensorState->tempValues[temp::BED]
	                    +stateData->sensorState->tempValues[temp::DOOR]) -100)/10;
	page += std::to_string(temp);
	
	page += HUMID;
	float humid = float( (stateData->sensorState->humidityValues[hum::BED]
	                     +stateData->sensorState->humidityValues[hum::DOOR]) -100)/10;
	page += std::to_string(humid);	

	page += CO2;
	page += std::to_string(stateData->sensorState->CO2ppm);	

	page += LIGHT;
	page += std::to_string(stateData->sensorState->lightValues[lght::BED]);
	
	page += MOVEMENT;
	uint32_t oldest = 0;
	uint8_t i;
	for(i=0; i<	mov::LEN; i++)
		if(stateData->sensorState->movement[i] > oldest)
			 oldest = stateData->sensorState->movement[i];

	switch(i){
		case mov::DOOR:
		switchStr = "door sensor";
		break;
		case mov::KITCHEN:
		switchStr = "test";
		break;
		case mov::BED_l:
		switchStr = "test";
		break;
		case mov::BED_r:
		switchStr = "test";
		break;	
		case mov::RADIATOR:
		switchStr = "test";
		break;
		switchStr = "test";
		case mov::MIDDLEROOM:
		break;
		switchStr = "toilet sensor";
		case mov::BATHROOM_WC:
		break;	
		switchStr = "shower sensor";
		case mov::BATHROOM_SHOWER:
		break;	
	}
	page += switchStr;
	
	page += LAST;
}
