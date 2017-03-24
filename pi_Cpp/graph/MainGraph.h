// graph one or multiple lines on the same canvas

#ifndef MAINGRAPH_H
#define MAINGRAPH_H

#include "TCanvas.h"
#include "TAxis.h"
#include "TFrame.h"
#include "TROOT.h"
#include "TGraphErrors.h"
#include "TGraph.h"
#include "TF1.h"
#include "TLegend.h"
#include "TArrow.h"
#include "TLatex.h"
#include "TPad.h"
#include "TMultiGraph.h"
#include "TGaxis.h"
#include "TText.h"

#include <string>
#include <bitset>
#include <memory>

const int MAXPLOTRESOLUTION = 1000; //for now 
const float AXISWITH = 0.1;

class PirData;
class SlowData;//FIXME is this needed?
#include "../dataStorage/PirData.h"
#include "../dataStorage/SlowData.h"
#include "../config.h"

class Graph
{
public:
  Graph(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT,
        std::shared_ptr<PirData> pirData, std::shared_ptr<SlowData> slowData);
private:
  //local cache of time data
  float y[MAXPLOTRESOLUTION]; //TODO check this and move everything back to floats if possible
  double x[MAXPLOTRESOLUTION];
  double yT; 
  double yH; 
  double yC;
  double yB; //one y value from each multigroup
             //yT then yH then yC then yB
  uint16_t len; //numb of datapoints to plot
  TCanvas* c1;

  TMultiGraph* mgT;
  TMultiGraph* mgH;
  TMultiGraph* mgC;
  TMultiGraph* mgB;

  TPad* padT; 
  TPad* padH;
  TPad* padC;
  TPad* padB;

  TPad* mpad;
  TLegend* leg;

  uint32_t startT, stopT;
  int yAxisesNumb;
	float yAxis_CumLabelWith;
  double x0[2]; //used for plotting fake lines

	//NOT USING CAUSE CANT BE DONE FAST ENOUGH/INEFFICIENT, HARD CODING INSTEAD  
//	/*find the maximum number of sig digets in the array*/
//	void maxWith(float y[MAXPLOTRESOLUTION], double& y_with){
//		//find max number		
//		int max = 0;		
//		for(int i=0; i<MAXPLOTRESOLUTION; i++){
//			//find maximum int			
//			if(y[i]>max){max = y[i];}
//		}
//		while something
//		{
//			float decimals = modf(y[i], NULL);
//			ndecimals = 1*(decimals*10 > 1) +	1*(decimals*100 > 1) + 1*(decimals*1000 > 1);
//			if(ndecimals>
//		}
//		return max;
//	}
	
  uint8_t mSensToPlot; //keep track of sensors to plot
  
  void initPlot();
  void finishPlot(uint8_t axisesToDraw);

  void plotPirData(uint8_t mSensToPlot, double x[MAXPLOTRESOLUTION], 
                   uint16_t y[MAXPLOTRESOLUTION], int len);
  void drawLine(double start, double stop, float h);
  
  std::string setupPirLegendPart(std::string text);
  
  TPad* addPad();
  TPad* setupPadsForPirPlot(std::string msensorLegend);
  
  void drawYAxis(TMultiGraph* mg, TPad* pad, double py1, double py2, 
                 float leftY, const char* axisTitle, int plot);
  void setMultiGroupXRange(TMultiGraph* mg, double y);
  
  void axisTimeFormatting(TMultiGraph* mg);
  void makeAxisInvisible(TMultiGraph* mg);
};

#endif // MAINGRAPH_H
