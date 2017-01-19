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
#include "TGaxis.h"
#include "TText.h"

void drawLine(int start, int stop, float h) {
  TLine *line = new TLine(start,h,stop,h);
  line->SetLineWidth(1);
  line->SetLineColor(4);
  line->Draw();
}

void drawYAxises(TMultiGraph* mgs[], TPad* pads[], const char* axisTitles[], int nAxises){
  double xmin, ymin, xmax, ymax;
  double px1, py1, px2, py2;  

  //shrink all the pads to make space
  pads[i]->GetPadPar(px1,py1,px2,py2);
  pads[i]->SetPad(px1,py1,px2-nAxises*0.2,py2);
  

  for(int i =0; i<nAxises; i++){
  //make existing y axis invisible //FIXME can we not just delete it?
    mgs[i]->GetYaxis()->SetTickLength(0);
    mgs[i]->GetYaxis()->SetLabelOffset(999);
    mgs[i]->GetYaxis()->SetNdivisions(1);
  //get the range of the axis to add
    pads[i]->GetRangeAxis(xmin,ymin,xmax,ymax);  
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
}


Tpad* addPad(){
  TPad* pad = new TPad("pad1","",0,0,1,1);
  pad->SetFillStyle(4000);
  pad->SetFrameFillStyle(0);
  
  return pad;
}

void graph() {
  TMultiGraph* mgs[8];
  TPad* pads[8];
  const char* axisTitles[8];
  int nAxises = 0;
  
  int mSensToPlot = 1;

  double px1, py1, px2, py2; 
  double startT = 0;
  double stopT = 4;  

  double yInPlot = 3;

  //set the values for the
  double x0[2] = {startT, stopT};
  double y0[2] = {yInPlot, yInPlot};

  double x1[4] = {0,1,2,3};
  double y1[4] = {1,2,1,4};

  double y2[4] = {1,2,3,2};
  double x2[4] = {0,1,2,4};

  float width = 1.4;
  int baseResolution = 800;

//init main graph area
  TCanvas* c1 = new TCanvas("c1","",baseResolution*width,baseResolution);

  //graph to ensure full range for all plots, this graph is added to all
  //multiplot objects
  TGraph* gr0 = new TGraph(2,x0,y0);
  gr0->SetLineColorAlpha(0,0);//set line fully transparant
  gr0->SetMarkerColorAlpha(0,0);//set marker fully transparant

//setup pads and canvasses
  TPad* pad1 = addPad();
  TPad* pad2 = addPad();

  pad1->GetPadPar(px1,py1,px2,py2);
  TLegend* leg = new TLegend(px1+0.1, py2-0.1, px2-0.1, py2-0.05);
  leg-> SetNColumns(2);



  TGraph* gr1 = new TGraph(4,x1,y1);
  gr1->SetMarkerColor(4);
  gr1->SetMarkerStyle(21);
  mgs[nAxises]g
  leg->AddEntry(gr1,"other test data","l");

  TGraph* gr2 = new TGraph(4,x2,y2);
  gr2->SetMarkerColor(4);
  gr2->SetMarkerStyle(21);
  leg->AddEntry(gr2,"Test data","l");

  TMultiGraph* mg1  = new TMultiGraph();
  mgs[nAxises] = mg1;
  nAxises++;
  TMultiGraph* mg2 = new TMultiGraph();
  mgs[nAxises] = mg2;
  
//add individual graphs to theire respective multigraph group
  mg1->Add(gr1);
  mg1->Add(gr0);
  mg2->Add(gr2);
  mg2->Add(gr0);

  if(mSensToPlot > 0){ 
    //resize pad
    pad2->GetPadPar(px1,py1,px2,py2);
    std::cout<<px1<<", "<<py1<<", "<<px2<<", "<<py2<<"\n";
    
    pad1->SetPad(px1,py1+0.2,px2,py2);
    pad2->SetPad(px1,py1+0.2,px2,py2);//replace 0 with size to shrink



    pad2->GetPadPar(px1,py1,px2,py2);
    TPad* mpad = new TPad("mpad","movement report",px1,py1,px2,py1-0.2);
    mpad->Draw();
    
    //removing frame info might lurk here:
    //https://webcache.googleusercontent.com/search?q=cache:5-5rJ90JanUJ:https://root.cern.ch/phpBB3/viewtopic.php%3Ft%3D19143+&cd=1&hl=nl&ct=clnk&gl=nl

    mpad->cd();  
    
    double ym[2] = {0,1};
    TGraph* grm = new TGraph(2,x0,ym);
    grm->SetLineColorAlpha(0,0);//set line fully transparant
    grm->SetMarkerColorAlpha(0,0);//set marker fully transparant
    grm->SetTitle("Movement sensors, 1: bathroom, 2:bed, 3:door, 4:kitchen, 5:heater, 6: bed 7: kitchen window side");
    grm->Draw("AL");
    
    //remove the axis
    grm->GetYaxis()->SetTickLength(0);
    grm->GetYaxis()->SetLabelOffset(999);
    grm->GetYaxis()->SetNdivisions(1);
    grm->GetXaxis()->SetTickLength(0);
    grm->GetXaxis()->SetLabelOffset(999);
    grm->GetXaxis()->SetNdivisions(1);
    
    drawLine(2, 4, 0.5);
    drawLine(2, 4, 0.25);
    
  //  //add describtion to lines
    TText* line0 = new TText(-0.05,0.5,"1");
    line0->Draw();
    line0 = new TText(-0.05,0.25,"2");
    line0->Draw(); 
  }

//link (draw) everything up correctly
  
  pad1->cd();  
  mg1->Draw("AL");
  leg->Draw();

  pad2->cd();
  mg2->Draw("AL");
  //pad1->Update();//FIXME needed?
  
  c1->cd();
  pad1->Draw();
  pad2->Draw();

//manage extra axises;
  addYAxises(mgs, pads, axisTitles, nAxises);

    
//render everything  
  c1->Print("test.pdf");
}

int main(){
  graph();
  }
