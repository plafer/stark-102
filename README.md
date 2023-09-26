# STARK 102

This repository contains an implementation of a STARK, and serves as an accompanying codebase to the excellent [Diving DEEP FRI in the STARK world](https://blog.lambdaclass.com/diving-deep-fri/) blog post by LambdaClass. It is called STARK 102 as it is meant to be a follow-up to Starkware's [STARK 101](https://starkware.co/stark-101/). Ultimately, the goal for this repository is to serve as a stepping stone to understanding and contributing to real world STARK libraries such as [Winterfell](https://github.com/facebook/winterfell).

## Problem statement 

Specifically, we implement a STARK that proves the following statement:

I computed the following sequence:

$$
a_0 = 3
a_{n+1} = (a_n)^2
$$

over the prime field with prime 17. 

Unlike STARK libraries such as [Winterfell](https://github.com/facebook/winterfell), this STARK implementation is completeley hardcoded to this problem.

## Philosophy

This repository builds on STARK 101. Specifically, STARK 101 only shows how the prover works; in STARK 102, we also implement the verifier. STARK 101 also left some nitty-gritty details unexplained, rightly so, to focus on the most important aspects. We try to fill that gap here and explain every little detail, either in this document or in comments in the source code. Also, this implementation is in Rust, the language typically used for production implementations of STARKs.

Similar to STARK 101, this is meant as a resource to learn about STARKs. The goal is not to be efficient; rather, it is to get the whole STARK idea across from start to finish for a simple problem. We agree with LambdaClass that doing a "pen and paper" example of a complex topic is the best way to learn it. Tailoring the implementation to the abovementioned problem allows the reader to easily play around with the code. For example, we hardcode the domain values for the trace (and low-degree extended) polynomials (see `src/domain.rs`). If the reader prints out domain values to inspect the program at runtime, they can refer back to the definition of the domain and *see* their printed value in the source file. We believe this can be helpful in relieving the brain to focus on actually learning STARKs; it certainly was for us.

Where appropriate, we choose the simpler of 2 valid options. For example, we use Lagrange interpolation instead of Fast Fourier Transforms, and FRI instead of DEEP FRI. There are no dependencies other than `blake3` for a hash function, and `anyhow` for convenient errors. We wanted every last detail about what makes STARKs tick to be contained in this repository, whether it's how to compute the logarithm of a field element, how Lagrange interpolation works, or how Merkle tree proof verification actually works. We strongly believe that having everything in one place, where the focus is *ease of understanding* as opposed to efficiency, is very helpful. This is similar in philosophy to STARK 101. Finally, some loops are unrolled, such as when computing FRI layers. This allows us to give a name to each FRI layer, and makes the number of layers explicit. We believe this can help readers identify shortcomings in their understanding. Maybe they expected there to be 4 layers, where in reality there are 3; they probably wouldn't have realized that if we stored the layers as `Vec<FriLayer>`.


# TODO
+ Talk about some terms I'll see in winterfell (folding factor, blowup factor, etc), and what those values are in this example
    + blowup factor: the domain size multiplier during LDE (here: 2)
    + folding factor: by how much you divide in-betIen each FRI layer (here: 2)
+ Explain the Scalable and Transparent parts
+ Talk about the channel
    + Notably, the pseudorandom params are *not* in the `StarkProof` struct; they're rederived by the verifier.
+ The verifier checks the evaluation *at a point* (the "query point")
    + Give the intuition why this works (LDE, etc)
    + to increase security, do more queries
+ Reader challenge: Make `3` a public param. Requires
    + changing boundary constraint (hint: you will need to implement polynomial division)
    + Initialize channel with public param instead of `CHANNEL_SALT`
+ Describe repo layout, and how to navigate it?
+ If something is unclear, please open an issue and we'll improve the docs
