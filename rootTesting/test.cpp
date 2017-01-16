// Builds a graph with errors, displays it and saves it as
// image. First, include some header files
// (not necessary for Cling)

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
#include "TMultiGraph.h"

void drawLine(int start, int stop) {
  TLine *line = new TLine(start,0.5,stop,0.5);
  line->SetLineWidth(2);
  line->SetLineColor(4);
  line->Draw();
}




void graph() {

  double x1[4] = {0,1,2,3};
  double x2[4] = {0,1,2,3};

  double y1[4] = {1,2,1,2};
  double y2[4] = {1,2,3,4};

//setup pads and canvasses
  TCanvas* c1 = new TCanvas();
  TPad *pad1 = new TPad("pad1","",0,0,1,1);
  TPad *pad2 = new TPad("pad2","",0,0,1,1);

  // Makes pad2 transparant
  pad2->SetFillStyle(4000);
  pad2->SetFrameFillStyle(0);

//config all the graphs
  TGraph* gr1 = new TGraph(4,x1,y1);
  gr1->SetMarkerColor(4);
  gr1->SetMarkerStyle(21);

  TGraph* gr2 = new TGraph(4,x2,y2);
  gr2->SetMarkerColor(4);
  gr2->SetMarkerStyle(21);

  TMultiGraph *mg1  = new TMultiGraph();
  TMultiGraph *mg2 = new TMultiGraph();
  
//add individual graphs to theire respective multigraph group
  mg1->Add(gr1);
  mg2->Add(gr2);

//link (draw) everything up correctly
  
  pad1->cd();  
  mg1->Draw("AL");

  pad2->cd();
  mg2->Draw("AL");
  pad1->Update();
  
  c1->cd();
  pad1->Draw();
  pad2->Draw();

//remove the axis
  mg2->GetYaxis()->SetTickLength(0);
  mg2->GetYaxis()->SetLabelOffset(999);
  mg2->GetYaxis()->SetNdivisions(1);

  //create a new axis on the other side for pad 2
  TGaxis* axis2 = new TGaxis(pad2->GetUxmin(), pad2->GetUymin(), 
                             pad2->GetUxmax(), pad2->GetUymax(),
                         

//render everything  
  c1->Print("test.pdf");
}

int main(){
  graph();
  }
