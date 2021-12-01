# Spectrogram

![demo](demo.gif)

## Try it out

Executables for Windows and Linux are available on the [releases page](https://github.com/JoshuaPostel/spectrogram/releases)

## Limitations

The .WAV parser is far from complete, as it only supports 16 bit samples and only RIFF, FMT, and DATA headers.  To avoid heavy workloads, the .WAV file is limited to 1MB.  If an incompatible file is opened, a dialog box with a descriptive error message shold appear.

## Motivation

This is a self imposed capstone project for [this DSP course](https://www.coursera.org/learn/dsp1).  The goal was to visualize the tradeoff between resolution in frequency and resolution in time.

## Acknowledgements

I implemented a nieve DFT for this project, however, the application uses [FFTW](https://www.fftw.org/) bindings.

* _Matteo Frigo and Steven G. Johnson, “The design and implementation of FFTW3,” Proc. IEEE 93 (2), 216–231 (2005)_
