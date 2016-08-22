#!/usr/bin/python3

import os
import json
import subprocess

def systemStatus():
    load = list(os.getloadavg())
    return load

def turnOffPc():
#   open an ssh connection to my main computer powers it off
    command = ["ssh", "-o ConnectTimeout=5",
              "kleingeld@192.168.1.12",
              "sudo shutdown -h now"]
    popen = subprocess.Popen(command, stdout=subprocess.PIPE)

def startRedshift():
#   open an ssh connection to my main computer and run the light filtering system
#   redshift
    command = ["ssh", "-o ConnectTimeout=5", 
              "kleingeld@192.168.1.12",
              "if pgrep 'redshift' > /dev/null; "
                    "then echo "
                    "'Running already not starting new instance'; "
                    "else export DISPLAY=:0; "
                    "redshift -l 52.1:4.5 & "
              "fi; "
              "exit"]              
    popen = subprocess.Popen(command, stdout=subprocess.PIPE)
