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

const int MAXPLOTRESOLUTION = 1000; //for now 
const float AXISWITH = 0.1;

class PirData;
class SlowData;//FIXME is this needed?
#include "../dataStorage/PirData.h"
#include "../dataStorage/SlowData.h"

enum plotables{
  MOVEMENTSENSOR0,
  MOVEMENTSENSOR1,
  MOVEMENTSENSOR2,
  MOVEMENTSENSOR3,
  MOVEMENTSENSOR4,
  MOVEMENTSENSOR5,
  MOVEMENTSENSOR6,
  MOVEMENTSENSOR7,

  TEMP_BED,
  TEMP_BATHROOM,
  TEMP_DOORHIGH,

  HUMIDITY_BED,
  HUMIDITY_BATHROOM,
  HUMIDITY_DOORHIGH,

  CO2PPM,

  BRIGHTNESS_BED,
  BRIGHTNESS_BEYONDCURTAINS,
  BRIGHTNESS_KITCHEN,
  BRIGHTNESS_DOORHIGH
};

class Graph
{
public:
  Graph(std::vector<plotables> toPlot, uint32_t startT, uint32_t stopT,
        PirData& pirData, SlowData& slowData);
private:
  //local cache of time data
  float y[MAXPLOTRESOLUTION];
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
  TLegend* leg;
  uint32_t startT, stopT;
  int yAxisesNumb;
  double x0[2]; //used for plotting fake lines
  
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
                 int nAxises, const char* axisTitle);
  void setMultiGroupXRange(TMultiGraph* mg, double y);
  
  void axisTimeFormatting(TMultiGraph* mg);
  void makeAxisInvisible(TMultiGraph* mg);
};

#endif // MAINGRAPH_H
