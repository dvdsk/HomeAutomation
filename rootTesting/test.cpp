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

void drawLine(int start, int stop) {
  TLine *line = new TLine(start,0.5,stop,0.5);
  line->SetLineWidth(2);
  line->SetLineColor(4);
  line->Draw();
}


void graph() {
  TCanvas *c1 = new TCanvas("c1","A Simple Graph Example",200,10,700,500);
  c1->SetGrid();
  const Int_t n = 5;
  //   Double_t x[n], y[n];
  //   for (Int_t i=0;i<n;i++) {
  //     x[i] = i*0.1;
  //     y[i] = 10*sin(x[i]+0.2);
  //     printf(" i %i %f %f \n",i,x[i],y[i]);
  //   }
  const int y1[n] = {1,1,0,0,1};
  const int x1[n] = {1,2,2,4,5};
  TGraph *gr = new TGraph(n,x1,y1);
  
  int y2[n] = {1,1,0,0,1};
  int x2[n] = {1,2,3,4,5};
  
  
  //gr->SetLineColor(2);
  //gr->SetLineWidth(1);
  //gr->SetMarkerColor(4);
  //gr->SetMarkerStyle(2);
  gr->SetTitle("Temperature");
  gr->GetXaxis()->SetTitle("A Date?");
  gr->GetYaxis()->SetTitle("Temp in C");
  gr->Draw("ACP");

  x2[3] = 10;

  TGraph *gr2 = new TGraph(n,x2,y2);
  gr2->Draw("AL");
  gr2->SetTitle("Temperature");

  x2[3] = 1;

  c1->Update();
  c1->GetFrame()->SetBorderSize(12);
  c1->Modified();



  c1->Print("test.pdf");
}

int main(){
  graph();
  }
