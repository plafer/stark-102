Coding up of [this](https://blog.lambdaclass.com/diving-deep-fri/).

We create a STARK about the sequence

a_0 = 3
a_{n+1} = (a_n)^2

over prime field with prime 17.


# TODO
+ Talk about some terms we'll see in winterfell (folding factor, blowup factor, etc), and what those values are in this example
    + blowup factor: the domain size multiplier during LDE (here: 2)
    + folding factor: by how much you divide in-between each FRI layer (here: 2)
+ Explain the Scalable and Transparent parts
