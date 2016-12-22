#include "MainGraph.h"

Graph::Graph(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT,
             PirData& pirData){

  bool onlyPir = true;
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

      case TEMP_BED:
        onlyPir = false;
        //fetchSlowData(0);//todo
        break;
      case TEMP_BATHROOM:
        onlyPir = false;
        //fetchSlowData(0);
        break;
      case TEMP_DOORHIGH:
        onlyPir = false;
        //fetchSlowData(0);
        break;
      case HUMIDITY_BED:
        onlyPir = false;
        //fetchSlowData(1);
        break;
      case CO2PPM:
        onlyPir = false;
        //TODO plot CO2
        break;
      case BRIGHTNESS_BED:
        onlyPir = false;
        //TODO plot brightness
        break;    
      default:
        break;
    }
  }

  if(onlyPir){updateLength(startT, stopT); }
  else {int x[2] = {0,0}; int y[2] = {0,0}; gr = new TGraph(2,x,y);}
  //else line only here as we always need a gr while testing 

  if(mSensToPlot > 0){ 
    std::cerr<<"fetching some data\n";
    len = pirData.fetchPirData(0, startT, stopT, x, y);
    std::cerr<<"plotting some movement graphs for ya all\n";
    plotPirData(mSensToPlot, x, y, len);
  }

  finishPlot();
}

void Graph::plotPirData(uint8_t mSensToPlot, uint32_t x[MAXPLOTRESOLUTION], 
                        float y[MAXPLOTRESOLUTION], int len){
  std::cerr<<"we got len: "<<len<<"\n";
  bool hasRisen[8];
  uint32_t timeOfRise[8];
  uint8_t* array;
  
  int numbPlots = __builtin_popcount(mSensToPlot);
  std::cout<<"numb of plots: "<<numbPlots<<"\n";
  float height[8];
  std::bitset<8> toPlot(mSensToPlot);

  //setup height
  int counter = 0;  
  for(int i; i<8; i++){
    float spacing = 1.0/numbPlots; //TODO change 1 to something sensible
    if(toPlot.test(i)){counter++;}
    height[i] = spacing*(counter); 
  } 
  
  for(int i=0; i<len; i++){
    //decode values from float to bitset
    array = (uint8_t*) &y[i];

    std::bitset<8> movement(array[1]); //TODO from uint8_t to bool array
    std::bitset<8> confirmed(array[0]);
    
    for(int j = 0; j<8; j++){
      //std::cout<<y[i]<<"\n";
      if(hasRisen[j]){
        if(movement.test(j) && confirmed.test(j) && toPlot.test(j)){
          drawLine(timeOfRise[j], x[i], height[j]);
          hasRisen[j] = false;
        }
      }
      else if(!movement.test(j) || !confirmed.test(j)){ 
        timeOfRise[j] = x[i];
        hasRisen[j] = true;
      }
    }
  }
}

void Graph::drawLine(uint32_t start, uint32_t stop, float h) {
  std::cout<<"drawing line between: "<<start<<"\tand: "<<stop<<"\t height: "<<h<<"\n";
  TLine *line = new TLine((double)start, h, (double)stop, h);
  line->SetLineWidth(2);
  line->SetLineColor(4);
  line->Draw();
}

void Graph::initPlot(){
  c1 = new TCanvas("c1","A Simple Graph Example",200,10,700,500);
  c1->SetGrid();
}

void Graph::updateLength(uint32_t startT, uint32_t stopT){
  //float yMax = numbOfMovementPlots*spacing+spacing;
  const double x[2] = {(double)startT,(double)stopT};
  const double y[2] = {0,0};
  
  gr = new TGraph(2,x,y);
  gr->Draw();
}

void Graph::axisTimeFormatting(){
  //gr->GetXaxis()->SetLabelSize(0.006);
  gr->GetXaxis()->SetNdivisions(-503);
  gr->GetXaxis()->SetTimeDisplay(1);
  gr->GetXaxis()->SetTimeFormat("%Y %H:%M %F 1970-01-01 00:00:00");
}

void Graph::finishPlot(){
  axisTimeFormatting();
  c1->RedrawAxis();
  c1->Update();
  c1->GetFrame()->SetBorderSize(12);
  c1->Modified();
  c1->Print("test.pdf");
}



//void scalePirData(float y_float[MAXPLOTRESOLUTION]){
//  //TODO
//}







//void graph() {
//  TCanvas *c1 = new TCanvas("c1","A Simple Graph Example",200,10,700,500);
//  c1->SetGrid();
//  const Int_t n = 5;
//  //   Double_t x[n], y[n];
//  //   for (Int_t i=0;i<n;i++) {
//  //     x[i] = i*0.1;
//  //     y[i] = 10*sin(x[i]+0.2);
//  //     printf(" i %i %f %f \n",i,x[i],y[i]);
//  //   }
//  int y1[n] = {1,1,0,0,1};
//  int x1[n] = {1,2,2,4,5};
//  TGraph *gr = new TGraph(n,x1,y1);
//  
//  int y2[n] = {1,1,0,0,1};
//  int x2[n] = {1,2,3,4,5};
//  
//  
//  //gr->SetLineColor(2);
//  //gr->SetLineWidth(1);
//  //gr->SetMarkerColor(4);
//  //gr->SetMarkerStyle(2);
//  gr->SetTitle("Temperature");
//  gr->GetXaxis()->SetTitle("A Date?");
//  gr->GetYaxis()->SetTitle("Temp in C");
//  //gr->Draw("ACP");
//  gr->Draw("AL");//this determins interpolation etc smoothness etc
//  // TCanvas::Update() draws the frame, after which one can change it
//  
//  drawLine(1,2);
//  drawLine(3,4);

//  c1->Update();
//  c1->GetFrame()->SetBorderSize(12);
//  c1->Modified();



//  c1->Print("test.pdf");
//}
