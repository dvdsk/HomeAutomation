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

const int MAXPLOTRESOLUTION = 1000; //for now 

class PirData;
#include "../dataStorage/PirData.h"

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
        PirData& pirData);
private:
  //local cache of time data
  float y[MAXPLOTRESOLUTION];
  uint32_t x[MAXPLOTRESOLUTION];
  uint16_t len; //numb of datapoints to plot
  TCanvas* c1;
  TGraph* gr;

  int numbOfMovementPlots;
  int nMPlotted; //number of already plotted movement sensors
  float spacing;
  
  void plotPirData(std::string name, uint32_t x[MAXPLOTRESOLUTION], float y[MAXPLOTRESOLUTION]);
  void drawLine(uint32_t start, uint32_t stop, float h);
  void initPlot();
  void finishPlot();
  void updateLength(uint32_t start_T, uint32_t stop_T);
  void axisTimeFormatting();
};

#endif // MAINGRAPH_H
