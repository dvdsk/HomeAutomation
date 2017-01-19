#include "MainGraph.h"

Graph::Graph(std::vector<plotables> toPlot, uint32_t startT_, uint32_t stopT_,
             PirData& pirData, SlowData& slowData){

  startT = startT_;
  stopT = stopT_;

  bool onlyPir = true;
  uint8_t axisesToDraw = 0;
  
  //bool slowData_gotTimeAxis; IMPLEMENT THIS FUNCTIONALITY TO SAVE TIME
  //DECODING THE TIME
  uint16_t y_bin[MAXPLOTRESOLUTION];
  double y[MAXPLOTRESOLUTION];
  bool tempPlotted, 
  int len;
  mSensToPlot=0;
  initPlot();
  std::string msensorLegend;

  //plot all the non movement data and count the number of movementsensors to plot
  for( auto &i : toPlot){
    switch(i){
      case MOVEMENTSENSOR0:
        mSensToPlot = mSensToPlot | 0b10000000;
        msensorLegend += setupPirLegendPart();
        break;
      case MOVEMENTSENSOR1:
        mSensToPlot = mSensToPlot | 0b01000000;
        msensorLegend += setupPirLegendPart();
        break;
      case MOVEMENTSENSOR2:
        mSensToPlot = mSensToPlot | 0b00100000;
        msensorLegend += setupPirLegendPart();
        break;
      case MOVEMENTSENSOR3:
        mSensToPlot = mSensToPlot | 0b00010000;
        msensorLegend += setupPirLegendPart();
        break;
      case MOVEMENTSENSOR4:
        mSensToPlot = mSensToPlot | 0b00001000;
        msensorLegend += setupPirLegendPart();
        break;
      case MOVEMENTSENSOR5:
        mSensToPlot = mSensToPlot | 0b00000100;
        msensorLegend += setupPirLegendPart();
        break;
      case MOVEMENTSENSOR6:
        mSensToPlot = mSensToPlot | 0b00000010;
        msensorLegend += setupPirLegendPart();
        break;
      case MOVEMENTSENSOR7:
        mSensToPlot = mSensToPlot | 0b00000001;
        msensorLegend += setupPirLegendPart();
        break;

      case TEMP_BED:
        {
          onlyPir = false;
          toDraw = toDraw | 0b00000001;
          len = slowData.fetchSlowData(startT, stopT, x, y, 1);//todo
          TGraph* gr1 = new TGraph(len,x,y);
          leg->AddEntry(gr1,"temperature sensor at bed","l");
          mgT->Add(gr1);
        }
        break;
      case TEMP_BATHROOM:
        {
          onlyPir = false;
          toDraw = toDraw | 0b00000001;
          len = slowData.fetchSlowData(startT, stopT, x, y, 2);//todo
          TGraph* gr2 = new TGraph(len,x,y);
          leg->AddEntry(gr1,"temperature sensor at bed","l");
          mgT->Add(gr2);
        }
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
    
    len = pirData.fetchPirData(startT, stopT, x, y_bin);
    TPad* mpad = setupPadsForPirPlot(msensorLegend)
    mpad->cd();
    
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

  //updateLength(x[0], x[len-1]); 
  finishPlot(); //COMMENT OUT WHEN NOT PLOTTING
  //c1->Print("test.pdf");
}

std::string Graph::setupPirLegendPart(){  
  std::string text = msensorLegend+" ("
                    +std::to_string(__builtin_popcount(mSensToPlot))
                    +") :movementsensor0";
  return text;
}

void Graph::plotPirData(uint8_t mSensToPlot, double x[MAXPLOTRESOLUTION], 
                        uint16_t y[MAXPLOTRESOLUTION], int len){
  
  //debug by showing the raw data fetched
  for(int i =0; i<len; i++){
    //std::cout<<"x["<<i<<"]: "<<(uint32_t)x[i]<<" y["<<i<<"]: "<<y[i]<<"\n";
  }
  
  //std::cerr<<"we got len: "<<len<<"\n";
  bool hasRisen[8] = {false};
  double timeOfRise[8];
  uint8_t* array;
  
  int numbPlots = __builtin_popcount(mSensToPlot);
  //std::cout<<"numb of plots: "<<numbPlots<<"\n";
  float height[8];
  std::bitset<8> toPlot(mSensToPlot);

  //setup height and draw numbers
  int counter = 0;  
  for(int i=0; i<8; i++){
    float spacing = 1.0/(numbPlots+1); //TODO change 1 to something sensible
    if(toPlot.test(i)){
      counter++
      height[i] = spacing*counter; 
      TText* line = new TText(-0.05,height[i],itoa(counter));
      line->Draw();
    }
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

TPad* Graph::setupPadsForPirPlot(std::string msensorLegend){
  double px1, py1, px2, py2;  
  float toShrink;
  int numbPlots = __builtin_popcount(mSensToPlot); 
  double ym[2] = {0,1};
  
  toShrink = 0.1*numbPlots;
  padT->GetPadPar(px1,py1,px2,py2);
  
  padT->SetPad(px1,py1+toShrink,px2,py2);
  padH->SetPad(px1,py1+toShrink,px2,py2);
  padC->SetPad(px1,py1+toShrink,px2,py2);
  padB->SetPad(px1,py1+toShrink,px2,py2);
  
  TPad* mpad = new TPad("mpad","movement report",px1,py1,px2,py1-toShrink);
 
  //add a graph to set up the pad
  TGraph* grm = new TGraph(2,x0,ym);
  grm->SetLineColorAlpha(0,0);//set line fully transparant
  grm->SetMarkerColorAlpha(0,0);//set marker fully transparant
  grm->SetTitle(msensorLegendc_str() );
  grm->Draw("AL");
 
  //remove the axis
  grm->GetYaxis()->SetTickLength(0); //FIXME cant we delete axis object?
  grm->GetYaxis()->SetLabelOffset(999);
  grm->GetYaxis()->SetNdivisions(1);
  grm->GetXaxis()->SetTickLength(0);
  grm->GetXaxis()->SetLabelOffset(999);
  grm->GetXaxis()->SetNdivisions(1);
 
  return mpad;
}

Tpad* Graph::addPad(){
  TPad* pad = new TPad("pad1","",0,0,1,1);
  pad->SetFillStyle(4000);
  pad->SetFrameFillStyle(0);
  
  return pad;
}

void Graph::initPlot(){
  c1 = new TCanvas();
  c1->SetGrid();
  
  //init all the multigraphs these group data with the same unit. It 
  //forces all the graps in a multigraph to use the same axis.
  mgT = new TMultiGraph(); //Temp
  mgH = new TMultiGraph(); //Humidity
  mgC = new TMultiGraph(); //CO2
  mgB = new TMultiGraph(); //Brightness
  
  //add Pads to draw the diffrent multigraphs on
  padT = addPad();//this adds a pad and makes it transparant
  padH = addPad();
  padC = addPad(); 
  padB = addPad();
  	
  //setup the legend;
  leg = new TLegend(px1+0.1, py2-0.1, px2-0.1, py2-0.05);
  leg-> SetNColumns(2);
  
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
  gr->Draw("AP");
  gr->GetXaxis()->SetLimits((double)startT, (double)stopT);
}

void Graph::axisTimeFormatting(){
  //gr->GetXaxis()->SetLabelSize(0.006);
  gr->GetXaxis()->SetNdivisions(-503);
  gr->GetXaxis()->SetTimeDisplay(1);
  gr->GetXaxis()->SetLabelOffset(0.02);
  gr->GetXaxis()->SetTimeFormat("#splitline{%H\:%M}{%d\/%m\/%y} %F 1970-01-01 00:00:00");       
}

void Graph::drawYAxis(TMultiGraph* mg, TPad* pad, double py1, double py2){
  double xmin, ymin, xmax, ymax;

  //make existing y axis invisible //FIXME can we not just delete it?
  mg->GetYaxis()->SetTickLength(0);
  mg->GetYaxis()->SetLabelOffset(999);
  mg->GetYaxis()->SetNdivisions(1);
  //get the range of the axis to add
  pad->GetRangeAxis(xmin,ymin,xmax,ymax);  
  TGaxis* axis = new TGaxis(0.9,py1+0.1,0.9,py2-0.1,ymin,ymax,510,"+L");
  //setup axis visual paramaters
  axis->SetLabelOffset(0.01);
  axis->SetLabelSize(0.03);
  axis->SetLineColor(kRed);
  axis->SetLabelFont(42);
  axis->SetTitle(axisTitles[i]);
  axis->SetTitleFont(42);
  axis->SetTitleSize(0.03);
  axis->Draw("AP"); 
}

void Graph::finishPlot(){
  //axisTimeFormatting(); solve for multi plots
  double px1, py1, px2, py2;    
  int nAxises;
  float toShrink;
  //shrink all t epads to make space
  nAxises = __builtin_popcount(axisesToDraw);
  toShrink = nAxises*0.2
  padT->GetPadPar(px1,py1,px2,py2);
  
  padT->SetPad(px1,py1,px2-toShrink,py2);  
  padH->SetPad(px1,py1,px2-toShrink,py2);  
  padC->SetPad(px1,py1,px2-toShrink,py2);  
  padB->SetPad(px1,py1,px2-toShrink,py2);  

  //FIXME needs to be moved to finish plot as we need data for
  //each plot type to be able to fill in y0
  TGraph* gr0 = new TGraph(2,x0,y0);
  gr0->SetLineColorAlpha(0,0);//set line fully transparant
  gr0->SetMarkerColorAlpha(0,0);//set marker fully transparant

  //add to all multigraphs the invisible graph gr0 that has as fixed
  //range the full range of the data request, this makes sure all graphs
  //share an x axis
  mgT->Add(gr0);
  mgH->Add(gr0);
  mgC->Add(gr0);
  mgB->Add(gr0);

  //draw all the axis //TODO add location as variable and make some
  //formula to determine location from
  c1->cd();
  if(axisesToDraw & 0b00000001){drawYAxis(mgT, padT, py1, py2); }
  if(axisesToDraw & 0b00000010){drawYAxis(mgH, padH, py1, py2); }
  if(axisesToDraw & 0b00000100){drawYAxis(mgC, padC, py1, py2); }
  if(axisesToDraw & 0b00001000){drawYAxis(mgB, padB, py1, py2); }
  
  c1->Print("test.pdf");
}
