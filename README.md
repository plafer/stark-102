Coding up of [this](https://blog.lambdaclass.com/diving-deep-fri/).

We create a STARK about the sequence

a_0 = 3
a_{n+1} = (a_n)^2

over prime field with prime 17.


# TODO
+ Philosophy: 
    + Many loops are unrolled, try to give names to all values.
    + The goal is not to be efficient or use the best algo; just to get the whole STARK idea across from start to finish
        + e.g. we do lagrange interpolation instead of FFT
    + Some things are hardcoded (e.g. `CyclicGroup`) when I believe useful.
        + In the case of cyclic group, if the reader wants to debug things, I
          believe seeing the values directly can help and free up parts of the
          brain
+ Explain how this builds on stark 101
    + Stark 101 has no verifier
    + Many details unexplained to focus on the core; we'll focus on those here
+ Things stark 102 won't focus on (this can be for stark 103!)
    + DEEP-FRI: it's an improvement on FRI, but not strictly needed to get the
      big picture. Uses a variation on the composition polynomial (called DEEP
      CP)
        + Recommend talk, and refer to winterfell?
+ Talk about some terms we'll see in winterfell (folding factor, blowup factor, etc), and what those values are in this example
    + blowup factor: the domain size multiplier during LDE (here: 2)
    + folding factor: by how much you divide in-between each FRI layer (here: 2)
+ Explain the Scalable and Transparent parts
+ Talk about the channel
    + Notably, the pseudorandom params are *not* in the `StarkProof` struct; they're rederived by the verifier.
+ The verifier checks the evaluation *at a point* (the "query point")
    + Give the intuition why this works (LDE, etc)
    + to increase security, do more queries
+ Reader challenge: Make `3` a public param. Requires
    + changing boundary constraint (hint: you will need to implement polynomial division)
    + Initialize channel with public param instead of `CHANNEL_SALT`
