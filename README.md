Coding up of [this](https://blog.lambdaclass.com/diving-deep-fri/).

We create a STARK about the sequence

a_0 = 3
a_{n+1} = (a_n)^2

over prime field with prime 17.


# TODO
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
