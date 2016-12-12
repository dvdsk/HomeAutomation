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
};

#endif // MAINGRAPH_H
