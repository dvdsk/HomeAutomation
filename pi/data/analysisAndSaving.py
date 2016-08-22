#!/usr/bin/python3
import multiprocessing
import tables
import numpy as np
import os
import time
import threading
import datetime

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.animation as animation
import seaborn as sns
#import talib as ta #used for moving average?? #TODO maybe faster
from scipy.signal import savgol_filter

from matplotlib import rcParams
#rcParams.update({'figure.autolayout': True})

lw = 0.4
sns.set_context("paper",font_scale=0.9)

#does all kinds of data shizzle that can then be requested from any
#process or thread
#


bufferSize = 5 #default non debugging size= 100
hdf5_path = '/home/pi/bin/homeAutomation/data/sensors.hdf5'

def init():
    sensorGet = multiprocessing.Queue(maxsize=0)
    sensorGetBack = multiprocessing.Queue(maxsize=0)
    analysisRq = multiprocessing.Queue(maxsize=1)
    
    if not os.path.isfile(hdf5_path):
    #   if there is no database file yet for some reallyweird reason, make it       
        
        #create the classes to store data in
        class THBsensors(tables.IsDescription):
            time= tables.UInt32Col(dflt = 0, pos = 0) # need long to fit time
            #since we need a small range we store the temperature in integers
            #this gives higher accuracy at the same bit user as floats in the
            #specific range we need
            temperature100= tables.Int16Col(dflt=-40, pos = 1)
            humidity100= tables.Int16Col(dflt=-40, pos = 2)
            light= tables.Int16Col(dflt=-40, pos = 3)
            co2ppm= tables.Int16Col(dflt=-40, pos = 4)

        class soilSensing(tables.IsDescription):
            time =           tables.UInt32Col(dflt = 0, pos = 0) # need long to fit time
            soil_moisture1 = tables.Float16Col(dflt=-40, pos = 1) # small float
            soil_moisture2 = tables.Float16Col(dflt=-40, pos = 1) # small float
            soil_moisture3 = tables.Float16Col(dflt=-40, pos = 1) # small float
            soil_moisture4 = tables.Float16Col(dflt=-40, pos = 1) # small float
            soil_moisture5 = tables.Float16Col(dflt=-40, pos = 1) # small float
            soil_moisture6 = tables.Float16Col(dflt=-40, pos = 1) # small float
        
        # Open a file in "w"rite mode
        fileh = tables.open_file(hdf5_path, mode = "w")
        expected1 = 365*24*60*12
        tab1 = fileh.create_table(fileh.root, 'tempHumidBrightCo2', THBsensors, 
                                  "temp, humidity and brightness data. To store" 
                                  +" the data more efficiently temp and humidity"
                                  +" are multiplied by 100 then stored as int", 
                                  expectedrows = expected1)
        expected2 = 365*24
        tab2 = fileh.create_table(fileh.root, 'plantMoisture', soilSensing, 
                                  "Moisture sensors data", expectedrows = expected2)
        tab1.flush()
        tab2.flush()
        fileh.close()
    else:
#        fileh = tables.openFile(hdf5_path, mode = "a")
#        table = fileh.root.tempHumidBright
#        table.remove_row(1700)
#        fileh.close()

#        #find and print bad values
#        fileh = tables.openFile(hdf5_path, mode = "r")
#        table = fileh.root.tempHumidBright
#        data = table.read_where("""(temperature100 == 0)""") #returns as 1d alas
#        fileh.close()
#        print(data)
        pass
    return sensorGet, sensorGetBack, analysisRq








def analyseSensorData(analysisRq, resourceLocks):
    #Get request (block)
    smooth = True
    
    def getData(resourceLocks, t1, t2):
    #   gets the data from the database
        lastData = np.copy(sensorDataArray) #Get data via global the np.copy 
                                            #should be thread safe        
        
        resourceLocks['sensorDb'].acquire(blocking=True, timeout = 10)
        fileh = tables.openFile(hdf5_path, mode = "r")
        table = fileh.root.tempHumidBright
        data = table.read_where("""(time > t1) & (time < t2)""") #returns as 1d alas
        fileh.close()
        resourceLocks['sensorDb'].release()
        mask = lastData['time'] != 0 #take only the measured values
        lastData = lastData[mask]
        
        #select on time range
        mask = np.logical_and(lastData['time'] > t1, lastData['time'] < t2) #take the right timeframe
        lastComplyingData = lastData[mask]           
        
        totData = np.concatenate((data,lastComplyingData), axis=0)

        return totData
    
    def formatTime(ax, t1, t2, X):#TODO not used atm
    #   take the time unit and axis and return axis setup for time viewing              
        minorF = False

        deltaT = t2-t1
        print(deltaT)
        if deltaT < 60*60:
            #min+sec notation
            minuteFraction = 60/(deltaT/60)
            mj = matplotlib.dates.MinuteLocator(interval=int(10/minuteFraction))
            fmj = '%S'#formatting major ticks
            mn = matplotlib.dates.SecondLocator(interval=15)
            fmn = '%M'#formatting minor ticks
            label = "Minutes and Seconds"
        
        elif deltaT < 3600*24:
            #hour+minutes
            houreFraction = 24/(deltaT/3600)
            mj = matplotlib.dates.HourLocator(interval=int(4/houreFraction))
            fmj = '%H'
            if deltaT < 3600*5:
                minorF = True
                mn = matplotlib.dates.MinuteLocator(interval=15)
                fmn = '%M'
                label = "Hours and Minutes"
            else:
                label = "Hours"
            
        elif deltaT < 3600*24*30:
            #hours+days
            dayFraction = 30/(deltaT/3600*24)
            mj = matplotlib.dates.DayLocator(interval=int(5/dayFraction))
            fmj = '%d'        
            mn = matplotlib.dates.HourLocator(interval=4)
            fmn = '%H'
            label = "Day and Hour"
        else :
            dayFraction = 365/(deltaT/3600*24)
            mj = matplotlib.dates.DayLocator(interval=int(5/dayFraction))
            fmj = '%d-%m'
            label = "Date"
        
        if X:
            ax.xaxis.set_major_locator(mj)
            ax.xaxis.set_major_formatter(matplotlib.dates.DateFormatter(fmj))
            if minorF:
                ax.xaxis.set_minor_locator(mn)
                ax.xaxis.set_minor_formatter(matplotlib.dates.DateFormatter(fmn))
        else:
            ax.yaxis.set_major_locator(mj)
            ax.yaxis.set_major_formatter(matplotlib.dates.DateFormatter(fmj))
            if minorF:
                ax.yaxis.set_minor_locator(mn)
                ax.yaxis.set_minor_formatter(matplotlib.dates.DateFormatter(fmn))
        
        return ax, label
        

    while True:  
        x, yList, typePlot, t1, t2 = analysisRq.get()
        #[x, yList, typePlot, t1,t2] options for x and y(elements):
        # temperature, humidity, light, time.
        data = getData(resourceLocks, t1, t2)
        
        #get the x-data for the plot:
        if x in ['temperature', 'humidity']:
            xData = data[x+'100']/100#convert back to float
        elif x == 'time':
            xData = np.tile(datetime.datetime(1900, 1, 1), len(data[x]))
            for i, stamp in enumerate(data[x]):
                date = datetime.datetime.fromtimestamp(stamp)#TODO make efficient
                xData[i] = date
        else:
            xData = data[x]
        
        #smooth x-data for plotting
        
        #get and smooth y-data for the plot:
        yDataList = []
        for y in yList:
            if y in ['temperature', 'humidity']:
                yData = data[y+'100']/100
            else:
                yData = data[y] 
            #smooth y-data     
            if smooth and len(yData) > 100:#TODO tweak window size futher
                window = int(len(yData)/25)
                if window%2==0:
                    window += 1
                y_smooth = savgol_filter(yData, window, 3)
            else:
                y_smooth = yData
            
            if y == 'time': #convert time data back to datetime objects
                yData = np.tile(datetime.datetime(1900, 1, 1), len(y_smooth))
                for i, stamp in enumerate(y_smooth):
                    date = datetime.datetime.fromtimestamp(stamp)#TODO make efficient
                    yData[i] = date
            else:
                yData = y_smooth
            
            yDataList.append(yData)

        #units translation dict 
        units = {'temperature': ' (°C)',
                 'humidity': ' (%)',
                 'light': ''}
                 #for time we do complicated shit(no more, its wip)
        
        #'''plot the data:'''###########################################
        fig, ax =  plt.subplots()
        fig.subplots_adjust(right=0.75)
        plt.ticklabel_format(style='plain', axis='y', scilimits=(0,0))
        plt.ticklabel_format(style='plain', axis='x', scilimits=(0,0))
        axes = [ax]

        #setup the colors for the plots
        color=iter(sns.color_palette("deep"))
        
        #make the plot(s)
        #'''setup first line/plot'''#
        prevQuantity = yList[0].split(' ', 1)[0]
        plotAx = ax
        #setup axis
        if prevQuantity == 'time':
#            plotAx, label = formatTime(plotAx, t1, t2, False)
            label = 'time'
        else:
#            if prevQuantity == 'temperature':#TODO should not be nesescairy with the data
#                plotAx.set_ylim(-5,40)
#            elif prevQuantity == 'humidity':
#                plotAx.set_ylim(0,100)
            label = yList[0]
            plotAx.set_ylabel(prevQuantity+units[prevQuantity])
        
        #actually plot
        yData=yDataList[0]
        c=next(color)
        if typePlot == 'line':                            
            lns = plotAx.plot(xData, yData, c=c, label=label, linewidth=lw, zorder=10)
        elif typePlot == 'scatter': 
            plotAx.set_xlim([np.amin(xData), np.amax(xData)])
            lns = plotAx.scatter(xData, yData, c=c, s=lw, label=label, marker='+')
        elif typePlot == 'histogram':
            lns = sns.distplot(xData, ax=plotAx)
        
        #'''repeat for other lines/plots'''#
        for yData, label in zip(yDataList[1:], yList[1:]):   
            quantity = label.split(' ', 1)[0]
            if quantity != prevQuantity: 
            #   check if we need to go to the next axis
                plotAx = ax.twinx()
                axes.append(plotAx)
                c=next(color)
                if quantity == 'time':
                    label = 'time'
#                    plotAx, label = formatTime(plotAx, t1, t2, False)
                else:
#                    if quantity == 'temperature':#TODO should not be nesescairy with the data
#                        plotAx.set_ylim(-5,40)
#                    elif quantity == 'humidity':
#                        plotAx.set_ylim(0,100)
                    plotAx.set_ylabel(quantity+units[quantity])
                prevQuantity = quantity
            else:
                pass
                #TODO change line style
            
            if typePlot == 'line':                           
                lns = lns+plotAx.plot(xData, yData, c=c, label=label, linewidth=lw, zorder=10)
            elif typePlot == 'scatter': 
                lns = lns+plotAx.scatter(xData, yData, c=c, label=label)
            #TODO 3d histogram 


        #move the axis so they dont overlap
        if len(axes) > 2:
            axes[2].spines["right"].set_position(("axes", 1.12))
        if len(axes) > 3:#currently dont really use this
            axes[3].spines["right"].set_position(("axes", 1.4))

        #setup x-axis labels
        quantity = x.split(' ', 1)[0]
        if quantity == 'time':
            fig.autofmt_xdate()
#            axes[0], label = formatTime(axes[0], t1, t2, True)
#            axes[0].set_xlabel(label)        
        else:
            ax.set_xlabel(quantity+'('+units[quantity]+')')
        
        #set y-axis colors to line colors
        if len(axes) > 1 and len(axes) == len(lns): #TODO change for line styles etc
            for ax, ln in zip(axes, lns):
                lc = ln.get_color()
                ax.yaxis.label.set_color(lc)
                for tl in ax.get_yticklabels():
                    tl.set_color(lc)
        
#        ax.legend(lns, [l.get_label() for l in lns])#TODO FIX THIS
        plt.savefig('/home/pi/bin/homeAutomation/data/graph.png', dpi=300)
        analysisRq.put(True)
    return
    









    
def sensorSchedual(sensorRequest):
    RCfromHR = {'temparature and humidity' :b'00',
                'lorum ipsum': b'01'}
    while True:
        time.sleep(5)
        sensorRequest.put(RCfromHR['temparature and humidity'])
    return
    
    
def process(sensorData, sensorGet, sensorGetBack, analysisRq, resourceLocks):
    global sensorDataArray 

    sensorDataArray = np.full(bufferSize, 0, dtype=[('time','u4'), 
                             ('temperature100','i2'), 
                             ('humidity100','i2'), 
                             ('light','i2'),
                             ('co2ppm','i2')])    
    rowCounter = 0

    #awnser code human readable to machine
    ACfromHR = {'temparature and humidity' :b'rt',#the r signals this is seperately requested data
                'lorum ipsum': b'r01',
                'None': b'None'}#have to use bytes obj since we can only 
                                #compare bytes with bytes
    request = 'None'
    
    t = threading.Thread(target = analyseSensorData,
                          args   = (analysisRq, resourceLocks))
    t.start()
    
    while True:
        raw = sensorData.get()
        #manage requests
        if not sensorGet.empty():
            request = sensorGet.get() #hier zit altijd maar 1 ding in thus
            #needs to be locked carefully to make sure that happens

        if ACfromHR[request] in raw:
            request = 'None'
            sensorGetBack.put(raw)#sends back raw data otherwise things will
                                  #get really messy here (deal with it hard
                                  #coded somewhere else) 
               
        if raw[0] == 116: #116 is ascii t
            raw = raw.decode()
            h = raw.index('h')
            l = raw.index('l')
            c = raw.index('c')
            sensorDataArray[rowCounter][1] = float(raw[1:h] )*100#temp
            sensorDataArray[rowCounter][2] = float(raw[h+1:l] )*100#humidity
            sensorDataArray[rowCounter][3] = 1023-int(raw[l+1:c] )
            sensorDataArray[rowCounter][4] = float(raw[c+1:-1] ) #co2ppm     

            sensorDataArray[rowCounter][0] = time.time()#last filled as we use this != 0 to 
                                               #check if there are values in the sensorarray
            rowCounter += 1 
        
        #if we have quite some data in memory, write it to disk
        if rowCounter == bufferSize:
            rowCounter = 0
            resourceLocks['sensorDb'].acquire(blocking=True, timeout = 10)
            fileh = tables.openFile(hdf5_path, mode = "a")
            tab = fileh.root.tempHumidBrightCo2
            tab.append(sensorDataArray)
            fileh.close()
            resourceLocks['sensorDb'].release()
            sensorDataArray = np.full(bufferSize, 0, dtype=[('time','u4'), 
                                     ('temperature100','i2'), 
                                     ('humidity100','i2'), 
                                     ('light','i2'),
                                     ('co2ppm','i2')])       
