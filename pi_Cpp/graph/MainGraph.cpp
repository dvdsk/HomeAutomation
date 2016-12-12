#include "MainGraph.h"

Graph::Graph(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT,
             PirData& pirData){

  int numbOfPlots=0;

  //plot all the non movement data and count the number of movementsensors to plot
  for( auto &i : toPlot){
    switch(i){
      case MOVEMENTSENSOR0:
        numbOfPlots++;
        break;
      case MOVEMENTSENSOR1:
        numbOfPlots++;
        break;
      case MOVEMENTSENSOR2:
        numbOfPlots++;
        break;
      case MOVEMENTSENSOR3:
        numbOfPlots++;
        break;
      case MOVEMENTSENSOR4:
        numbOfPlots++;
        break;
      case MOVEMENTSENSOR5:
        numbOfPlots++;
        break;
      case MOVEMENTSENSOR6:
        numbOfPlots++;
        break;
      case MOVEMENTSENSOR7:
        numbOfPlots++;
        break;

      case TEMP_BED:
        //fetchSlowData(0);//todo
        break;
      case TEMP_BATHROOM:
        //fetchSlowData(0);
        break;
      case TEMP_DOORHIGH:
        //fetchSlowData(0);
        break;
      case HUMIDITY_BED:
        //fetchSlowData(1);
        break;
      case CO2PPM:
        //TODO plot CO2
        break;
      case BRIGHTNESS_BED:
        //TODO plot brightness
        break;    
      default:
        break;
    }
  }

  //TODO figure out dimensions of plot
  //Now knowing the dimensions of the plot and the number of pir sensors. Plot the movement data.
  for( auto &i : toPlot) {
    switch (i) {
      case MOVEMENTSENSOR0:
        std::cout<<"MOVEMENTSENSOR0\n";
        //pirData.fetchPirData(0, startT, stopT, x, y);
        //plotPirData("sensor0", x, y);//plot funct has its own pointer for each sensor
        break;
      case MOVEMENTSENSOR1:
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
}

//void scalePirData(float y_float[MAXPLOTRESOLUTION]){
//  //TODO
//}


//void plotPirData(std::string name, uint32_t x[], int y[]){
//  //draw many lines etc

//}


//void drawLine(int start, int stop) {
//  TLine *line = new TLine(start,0.5,stop,0.5);
//  line->SetLineWidth(2);
//  line->SetLineColor(4);
//  line->Draw();
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
