#include "MainGraph.h"

Graph::Graph(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT,
             PirData& pirData, SlowData& slowData){

  bool onlyPir = true;
  //bool slowData_gotTimeAxis; IMPLEMENT THIS FUNCTIONALITY TO SAVE TIME
  //DECODING THE TIME
  uint16_t y_bin[MAXPLOTRESOLUTION];
  double y[MAXPLOTRESOLUTION];
  int len;
  mSensToPlot=0;
  initPlot();

  //plot all the non movement data and count the number of movementsensors to plot
  for( auto &i : toPlot){
    switch(i){
      case MOVEMENTSENSOR0:
        mSensToPlot = mSensToPlot | 0b10000000;
        break;
      case MOVEMENTSENSOR1:
        mSensToPlot = mSensToPlot | 0b01000000;
        break;
      case MOVEMENTSENSOR2:
        mSensToPlot = mSensToPlot | 0b00100000;
        break;
      case MOVEMENTSENSOR3:
        mSensToPlot = mSensToPlot | 0b00010000;
        break;
      case MOVEMENTSENSOR4:
        mSensToPlot = mSensToPlot | 0b00001000;
        break;
      case MOVEMENTSENSOR5:
        mSensToPlot = mSensToPlot | 0b00000100;
        break;
      case MOVEMENTSENSOR6:
        mSensToPlot = mSensToPlot | 0b00000010;
        break;
      case MOVEMENTSENSOR7:
        mSensToPlot = mSensToPlot | 0b00000001;
        break;

      case TEMP_BED: {
        onlyPir = false;
        len = slowData.fetchSlowData(startT, stopT, x, y, 1);//todo
        std::cout<<"plotting shizzle\n";
        for(int i = 0; i<len; i++){
          std::cout<<"time: "<<x[i]<<"\t temp_data: "<<y[i]<<"\n";
        }
        gr = new TGraph(len,x,y);
        gr->Draw();
        }
        break;
      case TEMP_BATHROOM:
        onlyPir = false;
        slowData.fetchSlowData(startT, stopT, x, y, 2);
        break;
      case TEMP_DOORHIGH:
        onlyPir = false;
        slowData.fetchSlowData(startT, stopT, x, y, 3);
        break;
      case HUMIDITY_BED:
        onlyPir = false;
        slowData.fetchSlowData(startT, stopT, x, y, 4);
        break;
      case HUMIDITY_BATHROOM:
        onlyPir = false;
        slowData.fetchSlowData(startT, stopT, x, y, 5);
        break;
      case HUMIDITY_DOORHIGH:
        onlyPir = false;
        slowData.fetchSlowData(startT, stopT, x, y, 6);
        break;
      case CO2PPM:
        onlyPir = false;
        slowData.fetchSlowData(startT, stopT, x, y, 7);
        break;
      case BRIGHTNESS_BED:
        onlyPir = false;
        //nonPirFastData;
        break;    
      default:
        break;
    }
  }

  if(mSensToPlot > 0){ 
    //std::cerr<<"fetching some data\n";
    len = pirData.fetchPirData(startT, stopT, x, y_bin);
    //std::cerr<<"plotting some movement graphs for ya all\n";
	if(len == 0){
	  std::cerr<<"NO DATAPOINTS WHERE PASSED TO THE GRAPH FUNCTION I AM"
	           <<" GOING TO JUST RETURN IT NOW!!\n";
	  return;
	}
  std::cout<<"len: "<<len<<"\n";
	std::cout<<"times: "<<x[0]<<", "<<x[len-1]<<"\n";
    
    if(onlyPir){
      updateLength(x[0], x[len-1]); 
    }    
    plotPirData(mSensToPlot, x, y_bin, len);  
  }

  updateLength(x[0], x[len-1]); 
  finishPlot(); //COMMENT OUT WHEN NOT PLOTTING
  //c1->Print("test.pdf");
}

void Graph::plotPirData(uint8_t mSensToPlot, double x[MAXPLOTRESOLUTION], 
                        uint16_t y[MAXPLOTRESOLUTION], int len){
  
  //debug by showing the raw data fetched
  for(int i =0; i<len; i++){
    std::cout<<"x["<<i<<"]: "<<(uint32_t)x[i]<<" y["<<i<<"]: "<<y[i]<<"\n";
  }
  
  //std::cerr<<"we got len: "<<len<<"\n";
  bool hasRisen[8] = {false};
  double timeOfRise[8];
  uint8_t* array;
  
  int numbPlots = __builtin_popcount(mSensToPlot);
  //std::cout<<"numb of plots: "<<numbPlots<<"\n";
  float height[8];
  std::bitset<8> toPlot(mSensToPlot);

  //setup height
  int counter = 0;  
  for(int i=0; i<8; i++){
    float spacing = 1.0/(numbPlots+1); //TODO change 1 to something sensible
    if(toPlot.test(i)){counter++;}
    height[i] = spacing*(counter); 
  } 
  
  for(int i=0; i<len; i++){
    //decode values from float to bitset
    array = (uint8_t*) &y[i];
    std::bitset<8> movement(array[0]); //TODO from uint8_t to bool array
    std::bitset<8> checked(array[1]);
    
    for(int j = 0; j<8; j++){
      if(hasRisen[j]){
		//if(toPlot.test(j)){std::cout<<"checked: "<<checked.test(j)<<"\n";}
        if(!movement.test(j) && checked.test(j) && toPlot.test(j)){
          drawLine(timeOfRise[j], x[i], height[j]);
          hasRisen[j] = false;
        }
      }
      else if(movement.test(j) && checked.test(j)){ 
        timeOfRise[j] = x[i];
        //std::cout<<"time: "<<x[i]<<"\n";
        hasRisen[j] = true;
      }
    }
  }
  //draw the all lines that havent been 'grounded' yet.
  for(int j = 0; j<8; j++){
    if(hasRisen[j] && toPlot.test(j)){
      drawLine(timeOfRise[j], x[len-1], height[j]);
    }
  }
}

void Graph::drawLine(double start, double stop, float h) {
  std::cout<<"drawing line between: "<<(uint32_t)start<<"\tand: "<<(uint32_t)stop<<"\t height: "<<h<<"\n";
  TLine *line = new TLine(start, h, stop, h);
  line->SetLineWidth(2);
  line->SetLineColor(4);
  line->Draw();
}

void Graph::initPlot(){
  //c1 = new TCanvas("c1","A Simple Graph Example",200,10,700,500);
  c1 = new TCanvas();
  c1->SetGrid();
  
	//c1->SetRightMargin(0.); TODO tweak value for optimal usage of space
	//c1->SetLeftMargin(0.);
	//c1->SetTopMargin(0.);
	//c1->SetBottomMargin(0.); 
}

void Graph::updateLength(uint32_t startT, uint32_t stopT){
  std::cout<<"limiting axis range to: "<<startT<<" to "<<stopT<<"\n";
  
  const double x[2] = {(double)startT,(double)stopT};
  const double y[2] = {0,0};
  gr = new TGraph(2,x,y);
  gr->Draw();
  gr->GetXaxis()->SetLimits((double)startT, (double)stopT);
}

void Graph::axisTimeFormatting(){
  //gr->GetXaxis()->SetLabelSize(0.006);
  gr->GetXaxis()->SetNdivisions(-503);
  gr->GetXaxis()->SetTimeDisplay(1);
  gr->GetXaxis()->SetLabelOffset(0.02);
  gr->GetXaxis()->SetTimeFormat("#splitline{%H\:%M}{%d\/%m\/%y} %F 1970-01-01 00:00:00");       
}

void Graph::finishPlot(){
  //axisTimeFormatting();
  //c1->RedrawAxis();
  //c1->Update();
  //c1->GetFrame()->SetBorderSize(12);
  //c1->Modified();
  c1->Print("test.pdf");
}
