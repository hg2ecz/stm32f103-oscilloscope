#!/usr/bin/python3

# HG2ECZ, 2018
import math
import serial
import signal
import tkinter
import argparse
import numpy

class Spectrum(tkinter.Tk):
    def __init__(self, device, amplification):
        self.amplification = float(amplification)
        tkinter.Tk.__init__(self)
        signal.signal(signal.SIGINT, self.sigint_handler)
        self.winfo_toplevel().title("Spectrumanalyzer - v0.10")

        self.width = 1300
        self.height = 380
        self.center = 360
        self.phasecenter = 470
        self.c = tkinter.Canvas(width=self.width, height=self.height, bg='white')
        self.c.pack()

        self.ser = serial.Serial(device)
        self.run()

    def sigint_handler(self, sig, frame):
        self.quit()
        self.update()

    def run(self):
        blen = self.width*2
        fftwindow = numpy.blackman(blen)
        for i in range(30):             # legalább 20 blokkot beolvasunk és eldobunk, mert az X nem bírja megjeleníteni
            s = self.ser.read(2*4096)
        sample = []
        for x in range(blen):
            sample.append(1.9*(s[2*x]-85))
        xy_fft = numpy.fft.fft(sample * fftwindow)
        xy_amp = []
        for x in range(blen//2):
            amp = abs(xy_fft[x]*5./blen)
            xy_amp.append(x)
            xy_amp.append(self.center-amp*self.amplification)

        self.c.delete('all')

        # Amplitude
        ct = -1
        for y in range(self.center-350, self.center+0, 25):
            if y == self.center:
                fcolor='black'
            elif ct & 1:
                fcolor='orange'
            else:
                fcolor='lightgreen'
            self.c.create_line(0, y, self.width, y, fill=fcolor)
            if fcolor == 'orange':
                self.c.create_text(5, y, anchor=tkinter.SW, text="%.1f V"%((self.center-y)/50/self.amplification))
            ct+=1

        ct=0
        for x in range(0, self.width, 28):
            if ct % 4:
                fcolor='lightgreen'
            else:
                fcolor='orange'
            self.c.create_line(x, self.center-350, x, self.center+5, fill=fcolor)
            self.c.create_line(x, self.phasecenter-90, x, self.phasecenter+90, fill=fcolor) # phase vertical scale
            if ct and fcolor == 'orange':
                self.c.create_text(x-20, self.center+20, anchor=tkinter.SW, text="%.1f kHz"%(5*ct))
            ct+=1
        self.c.create_line(xy_amp, fill='blue')

        self.after(1, self.run)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Process some integers.')
    parser.add_argument('-d', '--device', help='device, e.g. -d /dev/ttyACM0', default='/dev/ttyACM0')
    parser.add_argument('-a', '--amplification', help='device, e.g. -a 10', default='1')

    args = parser.parse_args()
    root = Spectrum(args.device, args.amplification)
    root.mainloop()
