#!/usr/bin/python3

# HG2ECZ, 2018
import math
import serial
import signal
import tkinter
import argparse

class Oscilloscope(tkinter.Tk):
    def __init__(self, device):
        tkinter.Tk.__init__(self)
        signal.signal(signal.SIGINT, self.sigint_handler)
        self.winfo_toplevel().title("Oszcilloszkóp - v0.13")

        self.width = 1300
        self.height = 550
        self.center = self.height*2//3
        self.c = tkinter.Canvas(width=self.width, height=self.height, bg='white')
        self.c.pack()

        self.ser = serial.Serial(device)
        self.run()

    def sigint_handler(self, sig, frame):
        self.quit()
        self.update()

    def run(self):
        blen =  self.width
        xy1 = []
        for i in range(30):             # legalább 20 blokkot beolvasunk és eldobunk, mert az X nem bírja megjeleníteni
            s = self.ser.read(2*4096)
        for x in range(blen):
            xy1.append(x)
            xy1.append(self.center-1.9*(s[2*x]-85))
        self.c.delete('all')

        ct = -1
        for y in range(self.center-350, self.center+176, 25):
            if y == self.center:
                fcolor='black'
            elif ct & 1:
                fcolor='orange'
            else:
                fcolor='lightgreen'
            self.c.create_line(0, y, self.width, y, fill=fcolor)
            if fcolor != 'lightgreen':
                self.c.create_text(5, y, anchor=tkinter.SW, text="%.1f V"%((self.center-y)/50))
            ct+=1

        ct=0
        for x in range(0, self.width, 23):
            if ct % 5:
                fcolor='lightgreen'
            else:
                fcolor='orange'
            self.c.create_line(x, self.center-350, x, self.center+175, fill=fcolor)
            if ct and fcolor == 'orange':
                self.c.create_text(x, self.center, anchor=tkinter.SW, text="%.2f ms"%(ct/20.))
            ct+=1

        # Görbe megjelenítése
        sin_line = self.c.create_line(xy1, fill='blue')
        self.after(1, self.run)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Process some integers.')
    parser.add_argument('-d', '--device', help='device, e.g. -d /dev/ttyACM0', default='/dev/ttyACM0')

    args = parser.parse_args()
    root = Oscilloscope(args.device)
    root.mainloop()


# Set signal before starting
