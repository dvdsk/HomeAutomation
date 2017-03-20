#include "MainGraph.h"

Graph::Graph(std::vector<plotables> toPlot, uint32_t startT_, uint32_t stopT_,
             std::shared_ptr<PirData> pirData, std::shared_ptr<SlowData> slowData){

  std::cout<<"STARTING PLOTTING\n";
  
  startT = startT_;
  stopT = stopT_;

  x0[0]= startT;
  x0[1]= stopT;


  //bool onlyPir = true;
  uint8_t axisesToDraw = 0;
  
  //bool slowData_gotTimeAxis; IMPLEMENT THIS FUNCTIONALITY TO SAVE TIME
  //DECODING THE TIME
  uint16_t y_bin[MAXPLOTRESOLUTION];
  double y[MAXPLOTRESOLUTION];
  int len;
  yAxisesNumb=0;
  mSensToPlot=0;
  initPlot();
  std::string msensorLegend;

  //plot all the non movement data and count the number of movementsensors to plot
  for( auto &i : toPlot){
    switch(i){
      case MOVEMENTSENSOR0:
        mSensToPlot = mSensToPlot | 0b10000000;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;
      case MOVEMENTSENSOR1:
        mSensToPlot = mSensToPlot | 0b01000000;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;
      case MOVEMENTSENSOR2:
        mSensToPlot = mSensToPlot | 0b00100000;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;
      case MOVEMENTSENSOR3:
        mSensToPlot = mSensToPlot | 0b00010000;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;
      case MOVEMENTSENSOR4:
        mSensToPlot = mSensToPlot | 0b00001000;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;
      case MOVEMENTSENSOR5:
        mSensToPlot = mSensToPlot | 0b00000100;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;
      case MOVEMENTSENSOR6:
        mSensToPlot = mSensToPlot | 0b00000010;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;
      case MOVEMENTSENSOR7:
        mSensToPlot = mSensToPlot | 0b00000001;
        msensorLegend += setupPirLegendPart(msensorLegend);
        break;

      case TEMP_BED:
        {
          //onlyPir = false;
          axisesToDraw = axisesToDraw | 0b00000001;
          len = slowData->fetchSlowData(startT, stopT, x, y, i);//todo
          TGraph* gr1 = new TGraph(len,x,y);
          yT = y[0];
          leg->AddEntry(gr1,"temperature bed","l");
          mgT->Add(gr1);
        }
        break;
      case TEMP_BATHROOM:
        {
          //onlyPir = false;
          axisesToDraw = axisesToDraw | 0b00000001;
          len = slowData->fetchSlowData(startT, stopT, x, y, i);//todo
          TGraph* gr2 = new TGraph(len,x,y);
          yT = y[0];
          leg->AddEntry(gr2,"temperature bathroom","l");
          mgT->Add(gr2);
        }
        break;
      case TEMP_DOORHIGH:
        //onlyPir = false;
        slowData->fetchSlowData(startT, stopT, x, y, i);
        break;
      case HUMIDITY_BED:
        {
          //onlyPir = false;
          axisesToDraw = axisesToDraw | 0b00000010;
          len = slowData->fetchSlowData(startT, stopT, x, y, i);//todo
          TGraph* gr4 = new TGraph(len,x,y);
          yH = y[0];
          leg->AddEntry(gr4,"Humidity bed","l");
          mgH->Add(gr4);
        }
        break;
      case HUMIDITY_BATHROOM:
        //onlyPir = false;
        slowData->fetchSlowData(startT, stopT, x, y, i);
        break;
      case HUMIDITY_DOORHIGH:
        //onlyPir = false;
        slowData->fetchSlowData(startT, stopT, x, y, i);
        break;
      case CO2PPM:
        //onlyPir = false;
        slowData->fetchSlowData(startT, stopT, x, y, i);
        break;
      case BRIGHTNESS_BED:
        //onlyPir = false;
        //nonPirFastData;
        break;    
      default:
        break;
    }
  }
  
  if(mSensToPlot > 0){
    
    len = pirData->fetchPirData(startT, stopT, x, y_bin);
    TPad* mpad = setupPadsForPirPlot(msensorLegend);
    //mpad->cd();
    
    if(len == 0){
      std::cerr<<"NO DATAPOINTS WHERE PASSED TO THE GRAPH FUNCTION I AM"
               <<" GOING TO JUST RETURN IT NOW!!\n";
      return;
    }
    std::cout<<"len: "<<len<<"\n";
    std::cout<<"times: "<<x[0]<<", "<<x[len-1]<<"\n";
      
    //if(onlyPir){
      //updateLength(x[0], x[len-1]); 
    //}    
    mpad->cd();
    plotPirData(mSensToPlot, x, y_bin, len);
  }
  //updateLength(x[0], x[len-1]); 
  finishPlot(axisesToDraw); //COMMENT OUT WHEN NOT PLOTTING
  //c1->Print("test.pdf");
}

std::string Graph::setupPirLegendPart(std::string text){  
  text = text+" ("+std::to_string(__builtin_popcount(mSensToPlot))
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
  double height[8];
  std::bitset<8> toPlot(mSensToPlot);

  //setup height and draw numbers
  int counter = 0;  
  for(int i=0; i<8; i++){
    double spacing = 1.0/(numbPlots+1); //TODO change 1 to something sensible
    if(toPlot.test(i)){
      counter++;
      height[i] = spacing*counter; 
      TText* line = new TText(-0.05,height[i],std::to_string(counter).c_str());
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
  //std::cout<<"drawing line between: "<<(uint32_t)start<<"\tand: "<<(uint32_t)stop<<"\t height: "<<h<<"\n";
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
  
  std::cout<<"numbPlots: "<<numbPlots<<"\n";
  toShrink = 0.2;//0.1*numbPlots;
  padT->GetPadPar(px1,py1,px2,py2);
  
  padT->SetPad(px1,py1+toShrink,px2,py2);
  padH->SetPad(px1,py1+toShrink,px2,py2);
  padC->SetPad(px1,py1+toShrink,px2,py2);
  padB->SetPad(px1,py1+toShrink,px2,py2);
  
  std::cout<<"x0[0]: "<<x0[0]<<" x0[1]: "<<x0[1]<<"\n";
  //mpad = new TPad("mpad","movement report",px1,py1,px2,py1-toShrink);
  mpad = new TPad("mpad","",px1,py1,px2,toShrink);
  mpad->Draw();
  mpad->cd();
 
  
 
  //add a graph to set up the pad
  TGraph* grm = new TGraph(2,x0,ym);
  //grm->SetLineColorAlpha(0,0);//set line fully transparant
  //grm->SetMarkerColorAlpha(0,0);//set marker fully transparant
  grm->SetTitle(msensorLegend.c_str() );
  grm->Draw("AL");
 
  //remove the axis
  grm->GetYaxis()->SetTickLength(0); //FIXME cant we delete axis object?
  grm->GetYaxis()->SetLabelOffset(999);
  grm->GetYaxis()->SetNdivisions(1);
  grm->GetXaxis()->SetLabelSize(0.14);
  //grm->GetXaxis()->SetTickLength(0);
  //grm->GetXaxis()->SetLabelOffset(999);
  //grm->GetXaxis()->SetNdivisions(1);
 
  return mpad;
}

TPad* Graph::addPad(){
  TPad* pad = new TPad("pad1","",0,0,1,1);
  pad->SetFillStyle(4000);
  pad->SetFrameFillStyle(0);
  
  return pad;
}

void Graph::initPlot(){
  double px1, py1, px2, py2;
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
  padT->GetPadPar(px1,py1,px2,py2);  
  leg = new TLegend(px1, py2-0.1, px2-0.1, py2-0.05);
  leg->SetNColumns(2);
}

void Graph::axisTimeFormatting(TMultiGraph* mg){
  mg->GetXaxis()->SetNdivisions(-503);
  mg->GetXaxis()->SetTimeDisplay(1);
  mg->GetXaxis()->SetLabelOffset(0.02);
  mg->GetXaxis()->SetTimeFormat("#splitline{%H\:%M}{%d\/%m\/%y} %F 1970-01-01 00:00:00");
}

void Graph::makeAxisInvisible(TMultiGraph* mg){
  //FIXME some runtime bug with axises I think 
  mg->GetXaxis()->SetNdivisions(1);
  mg->GetXaxis()->SetLabelOffset(999);
  mg->GetXaxis()->SetLabelSize(0); 
}

void Graph::drawYAxis(TMultiGraph* mg, TPad* pad, double py1, 
                      double py2, int nAxises, const char* axisTitle){
  double xmin, ymin, xmax, ymax;
  double xpos;

  mg->GetYaxis()->SetTickLength(0);
  mg->GetYaxis()->SetLabelOffset(999);
  mg->GetYaxis()->SetNdivisions(1);
  
  //get the range of the axis to add
  pad->GetRangeAxis(xmin,ymin,xmax,ymax);
  xpos = 0.9-(nAxises-1)*(AXISWITH-0.01) + (yAxisesNumb)*(AXISWITH);
  std::cerr<<"xpos "<<xpos<<"\n"
           <<"py1: "<<py1<<" py2: "<<py2<<"\n";
  TGaxis* axis = new TGaxis(xpos,py1+0.1, 
                            xpos,py2-0.1,
                            ymin,ymax,510,"+L");
  yAxisesNumb++;
  
  //setup axis visual paramaters
  axis->SetLabelOffset(0.01);
  axis->SetLabelSize(0.03);
  axis->SetLineColor(kRed);
  axis->SetLabelFont(42);
  axis->SetTitle(axisTitle);
  axis->SetTitleFont(42);
  axis->SetTitleSize(0.03);
  axis->Draw("AP"); 
}

void Graph::setMultiGroupXRange(TMultiGraph* mg, double y){
  double y0[2];
  
  y0[0] = y;
  y0[1] = y;
  
  TGraph* gr0 = new TGraph(2,x0,y0);
  gr0->SetLineColorAlpha(0,0);//set line fully transparant
  gr0->SetMarkerColorAlpha(0,0);//set marker fully transparant  
  
  mg->Add(gr0);
}

void Graph::finishPlot(uint8_t axisesToDraw){
  double px1, py1, px2, py2;
  int nAxises;
  float toShrink;

  //draw (connect) all graph parts in the right order
	// "AL" simple polyline, "AC" smooth curve
  if(axisesToDraw & 0b00000001){padT->cd(); mgT->Draw("AC");}
  if(axisesToDraw & 0b00000010){padH->cd(); mgH->Draw("AC");}
  if(axisesToDraw & 0b00000100){padC->cd(); mgC->Draw("AC");}
  if(axisesToDraw & 0b00001000){padB->cd(); mgB->Draw("AC");}

  c1->cd();
  padT->Draw();
  padH->Draw();
  padC->Draw();
  padB->Draw();
  //leg->Draw(); //FIXME LEGEND POS NEEDS FIXING

  //add to all multigraphs the invisible graph gr0 that has as fixed
  //range the full range of the data request, this makes sure all graphs
  //share an x axis
  setMultiGroupXRange(mgT, yT);
  setMultiGroupXRange(mgH, yH);
  setMultiGroupXRange(mgC, yC);
  setMultiGroupXRange(mgB, yB);

  //format multigroup axis
  if(axisesToDraw & 0b00000001){axisTimeFormatting(mgT); }
  if(axisesToDraw & 0b00000010){axisTimeFormatting(mgH); }
  if(axisesToDraw & 0b00000100){axisTimeFormatting(mgC); }
  if(axisesToDraw & 0b00001000){axisTimeFormatting(mgB); }

  //shrink all t epads to make space for extra axis
  nAxises = __builtin_popcount(axisesToDraw);
  toShrink = (nAxises-1)*AXISWITH;
  std::cout<<"toShrink: "<<toShrink<<"\n";
  padT->GetPadPar(px1,py1,px2,py2);
  std::cout<<","<<px1<<","<<px2<<","<<py1<<","<<py2<<"\n";
  
  padT->SetPad(px1,py1,px2-toShrink,py2);  
  padH->SetPad(px1,py1,px2-toShrink,py2);  
  padC->SetPad(px1,py1,px2-toShrink,py2);  
  padB->SetPad(px1,py1,px2-toShrink,py2);  
  padT->GetPadPar(px1,py1,px2,py2);
  std::cout<<","<<px1<<","<<px2<<","<<py1<<","<<py2<<"\n";

  //draw all the axis //TODO add location as variable and make some
  //formula to determine location from
  c1->cd();
  if(axisesToDraw & 0b00000001){
    drawYAxis(mgT, padT, py1, py2, nAxises, "Temperature (C)"); 
  }
  if(axisesToDraw & 0b00000010){
    drawYAxis(mgH, padH, py1, py2, nAxises, "Humidity (%)"); 
  }
  if(axisesToDraw & 0b00000100){
    drawYAxis(mgC, padC, py1, py2, nAxises, "CO2 concentration (ppm)"); 
  }
  if(axisesToDraw & 0b00001000){
    drawYAxis(mgB, padB, py1, py2, nAxises, "Brightness (relative)"); 
  }
  std::cout<<"PRINTING PLOT\n";
  c1->Print("test.pdf");
}
