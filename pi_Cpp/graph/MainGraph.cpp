#include "MainGraph.h"

Graph::Graph(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT,
             PirData& pirData){

  bool onlyPir = true;
  nMPlotted=0;
  initPlot();

  //plot all the non movement data and count the number of movementsensors to plot
  for( auto &i : toPlot){
    switch(i){
      case MOVEMENTSENSOR0:
        numbOfMovementPlots++;
        break;
      case MOVEMENTSENSOR1:
        numbOfMovementPlots++;
        break;
      case MOVEMENTSENSOR2:
        numbOfMovementPlots++;
        break;
      case MOVEMENTSENSOR3:
        numbOfMovementPlots++;
        break;
      case MOVEMENTSENSOR4:
        numbOfMovementPlots++;
        break;
      case MOVEMENTSENSOR5:
        numbOfMovementPlots++;
        break;
      case MOVEMENTSENSOR6:
        numbOfMovementPlots++;
        break;
      case MOVEMENTSENSOR7:
        numbOfMovementPlots++;
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

  //TODO figure out dimensions of plot
  //Now knowing the dimensions of the plot and the number of pir sensors. Plot the movement data.
  spacing = 0.2;
  for( auto &i : toPlot) {
    switch (i) {
      case MOVEMENTSENSOR0:
        std::cout<<"MOVEMENTSENSOR0\n";
        len = pirData.fetchPirData(0, startT, stopT, x, y);
        plotPirData("sensor0", x, y);//plot funct has its own pointer for each sensor
        break;
      case MOVEMENTSENSOR1:
        std::cout<<"MOVEMENTSENSOR1\n";
        len = pirData.fetchPirData(1, startT, stopT, x, y);
        plotPirData("sensor1", x, y);//plot funct has its own pointer for each sensor
        break;
      case MOVEMENTSENSOR2:
        break;
      case MOVEMENTSENSOR3:
        break;
      case MOVEMENTSENSOR4:
        break;
      case MOVEMENTSENSOR5:
        break;
      case MOVEMENTSENSOR6:
        break;
      case MOVEMENTSENSOR7:
        break;
      default:
        break;
    }
  }
  if(onlyPir){updateLength(startT, stopT); }
  else {int x[2] = {0,0}; int y[2] = {0,0}; gr = new TGraph(2,x,y);}
  //else line only here as we always need a gr
  finishPlot();
}

void Graph::plotPirData(std::string name, uint32_t x[MAXPLOTRESOLUTION], float y[MAXPLOTRESOLUTION]){
  const static int CONFIRMED_ZERO = 1;
  const static int CONFIRMED_ONE = 3;
  
  float h = nMPlotted*spacing+spacing;
  bool hasRisen = false;
  uint32_t timeOfRise;
  //draw many lines etc
  std::cout<<"drawing: "<<name<<"\n";
  for(int i=0; i<len; i++){
    //std::cout<<y[i]<<"\n";
    if(hasRisen){
      if(y[i] == CONFIRMED_ZERO){
        drawLine(timeOfRise, x[i], h);
        hasRisen = false;
      }
    }
    else if(y[i] == CONFIRMED_ONE){ 
      timeOfRise = x[i];
      hasRisen = true;
    }
  }
}

void Graph::drawLine(uint32_t start, uint32_t stop, float h) {
  std::cout<<"drawing line\n";
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
