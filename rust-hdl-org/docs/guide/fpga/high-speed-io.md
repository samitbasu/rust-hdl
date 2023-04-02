# High Speed I/O

Another area where FPGAs excel is at high speed predictable I/O.  Suppose you are
building yourself a radio, and want to sample some intermediate signal at a couple 
of MSPS (Million samples per second).  It happens.  That means that some dedicated
circuitry is going to be sending you data at some pretty high rate.  How high?  
Well, assume the samples are 16 bits each, and you are shooting for 2 MSPS.  That 
means you need about 32 Mbits/second to keep up with the incoming data.  Sounds easy, 
right?  I mean, we have ubiquitous 1 Gbit/second ethernet everywhere.  What's a 
32 Mbit signal stream?

But it is actually not so easy.  That chip (an analog digital converter chip) will send 
a constant stream of data at high speed.  You might be able to read that with a microcontroller,
but it won't be easy.  Even if the chip satisfies some standard protocol, like SPI,
it may not send data the way your hardware SPI controller expects.  The chip manufacturer
has probably assumed that this is Your Problem.

FPGAs can solve these types of problems through a combination of 3 pieces:

- They typically have dedicated circuitry that deals with high speed I/O, and most high
speed interfaces operate the same or similar way.  These circuits speak the various electrical
protocols used by standards such as LVDS, etc.
- They allow you to quickly pack high speed, narrow data streams, into lower speed, wide
data streams.  A standard technique is to take that 32 Mbit/second stream and turn it
into a 2 million words/second stream, where the words are 16 bits wide.
- They typically contain hardware to do the reverse too - to take wide, slow streams and
convert them back into fast, narrow ones.
