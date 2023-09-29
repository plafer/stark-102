# STARK 102

This repository contains an implementation of a STARK, and serves as an accompanying codebase to the excellent [Diving DEEP FRI in the STARK world](https://blog.lambdaclass.com/diving-deep-fri/) blog post by LambdaClass. It is called STARK 102 as it is meant to be a follow-up to Starkware's [STARK 101](https://starkware.co/stark-101/). Ultimately, the goal for this repository is to serve as a stepping stone to understanding and contributing to real world STARK libraries such as [Winterfell](https://github.com/facebook/winterfell).

Please open an issue or start a new Discussion if anything is not clear and requires further explanation. If something is confusing to you, it probably is for many others! We'll gladly use the feedback to improve the documentation.

## Problem statement 

Specifically, we implement a STARK that proves the following statement:

I computed the following sequence:

$$
\begin{align}
& a_0 = 3           \\
& a_{n+1} = (a_n)^2 
\end{align}
$$

over the prime field with prime 17. 

Unlike STARK libraries such as [Winterfell](https://github.com/facebook/winterfell), this STARK implementation is completeley hardcoded to this problem.

## Philosophy

This repository builds on STARK 101. Specifically, STARK 101 only shows how the prover works; in STARK 102, we also implement the verifier. STARK 101 also left some nitty-gritty details unexplained, rightly so, to focus on the most important aspects. We try to fill that gap here and explain every little detail, either in this document or in comments in the source code. Also, this implementation is in Rust, the language typically used for production implementations of STARKs.

Similar to STARK 101, this is meant as a resource to learn about STARKs. The goal is not to be efficient; rather, it is to get the whole STARK idea across from start to finish for a simple problem. We agree with LambdaClass that doing a "pen and paper" example of a complex topic is the best way to learn it. Tailoring the implementation to the abovementioned problem allows the reader to easily play around with the code. For example, we hardcode the domain values for the trace (and low-degree extended) polynomials (see `src/domain.rs`). If the reader prints out domain values to inspect the program at runtime, they can refer back to the definition of the domain and *see* their printed value in the source file. We believe this can be helpful in relieving the brain to focus on actually learning STARKs; it certainly was for us.

Where appropriate, we choose the simpler of 2 valid options. For example, we use Lagrange interpolation instead of Fast Fourier Transforms, and FRI instead of DEEP FRI. There are no dependencies other than `blake3` for a hash function, and `anyhow` for convenient errors. We wanted every last detail about what makes STARKs tick to be contained in this repository, whether it's how to compute the logarithm of a field element, how Lagrange interpolation works, or how Merkle tree proof verification actually works. We strongly believe that having everything in one place, where the focus is *ease of understanding* as opposed to efficiency, is very helpful. This is similar in philosophy to STARK 101. Finally, some loops are unrolled, such as when computing FRI layers. This allows us to give a name to each FRI layer, and makes the number of layers explicit. We believe this can help readers identify shortcomings in their understanding. Maybe they expected there to be 4 layers, where in reality there are 3; they probably wouldn't have realized that if we stored the layers as `Vec<FriLayer>`.

## How to approach the repository
`lib.rs` contains the definition of `StarkProof`, the type that defines what a proof looks like. You should first head over to `prover::generate_proof()` to see how a proof is constructed. This will introduce you to all our core types, such as `field::BaseField`, `poly::Polynomial`, `merkle::MerkleTree`, etc.

Then, you can head over to `verifier::verify()` to see how the verifier uses the `StarkProof` struct to accept or reject a proof.

The test at the bottom of `lib.rs` demonstrates the usage of the library. You can also run `cargo doc --open` to generate the docs.

## Discussion
In this section we will discuss important topics in detail. We will focus on the ones that weren't fully explored in STARK 101.

### Commit and Query pattern
If if it is not clear to you why the "commit and query" strategy used in STARKs is a valid way to verify if the prover does indeed have the claimed polynomial, then I recommend you read [this article](https://vitalik.ca/general/2017/11/09/starks_part_1.html) by Vitalik. I found it did a great job at conveying the intuition.

### The Channel abstraction
The `Channel`, defined in `src/channel.rs`, is the type that implements the Fiat-Shamir transform. You will find an equivalent type both in STARK 101 and Winterfell. It is a core piece of the STARK implementations.

The Fiat-Shamir transform is a widely used technique to convert an interactive protocol into a non-interactive one. STARKs are defined as an interactive protocol turned non-interactive using the Fiat-Shamir transform. I recommend watching the first 7 minutes of [this video](https://youtu.be/9cagVtYstyY?si=85sINdOOvwxhTRio) to get a concise description of the Fiat-Shamir transform.

The `Channel` works in the following way. Creating a `Channel` with `Channel::new()` initializes it with a fixed value (currently the hash of 42). The prover can send messages to the verifier using `Channel::commit()`. Internally, this updates the `Channel`'s state by hashing the prover's message with its current state. Then, a verifier can send messages back (which as mentioned in the video are defined to be uniformly random values in an Interactive Argument) with either `Channel::random_element()` or `Channel::random_integer()`. This works simply by interpreting the `Channel`'s current hash as a field element or an integer, respectively. We then make sure to update the internal hash to a new value so that the `random_*()` methods can be called multiple times and return different values for each call.

The `Channel` is a very clean abstraction to turn an interactive protocol into a non-interactive one. You should now go re-read the implementation of `prover::generate_proof()`, and pay attention to all the calls to `channel.commit()` and `channel.random_element()`/`channel.random_integer()`. In your head, you should now see those as messages being sent back and forth between the prover and the verifier!

Finally, let's turn our attention to how the verifier uses the `Channel` in `verifier::verify()`. First, it must interact with the `Channel` in exactly the same way that the prover did. That way, it ensures to draw the same values from `Channel::random_element()` and `Channel::random_integer()`. This is critical. Notice that the random values (from `random_element/integer()`) are *not* included in the `StarkProof`. Rather, they are re-derived by the verifier. Pause and ponder why this is the only way that the verifier can ensure that the values are indeed random, and that the prover didn't pick convenient values. Think about how the prover could trick the verifier if all "random" values were included in the `StarkProof` as opposed to being rederived by the verifier.

### Why the prover doesn't need to send the Merkle commitment and proof of the last FRI layer
You might have noticed that we don't send a `MerkleRoot` of the last FRI layer of degree 0, and hence nor don't we send a `MerklePath` along with the [last queried element](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/lib.rs#L55).

The last layer (degree 0) has 2 elements that have the same value (remember: a
degree 0 polynomial is a constant function `f(x) = c`). We don't build a Merkle
tree for that layer as it is unnecessary: assuming that the prover sends the
right value for the last layer to the verifier (if it doesn't the proof fails
anyway), then the Merkle root is deterministic and doesn't provide any
information.

Try it yourself. Build a Merkle tree of 2 or 4 of the same elements, and build a `MerklePath` for each of them. Notice that all the `MerklePath`s are equal. Hence, in the scenario that we sent a `MerklePath` along with the element of the last FRI layer, the verifier could take the value that it expects for the last layer ([here](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/verifier.rs#L125)), build the `MerklePath` itself, and ensure that the `MerklePath` it got is equal to the one included in the proof. But, it might as well do the equality check on the value directly; why bother with the `MerklePath`? This is the intuition behind what it means to "not provide any information".

### Prover query phase: computing the correct indices
When constructing the query phase of the proof, the prover needs to send $layer(x)$ and $layer(-x)$ for each FRI layer, where $layer$ is the polynomial of a given FRI layer. However, it must also send the Merkle proof for $layer(-x)$. Therefore, it must know the index in the FRI layer evaluations which corresponds to $layer(-x)$ so that it can [construct that Merkle proof](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/merkle.rs#L27). This is done [here](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/prover.rs#L159). Then, it must also compute the index of $next_layer(x^2)$, where $next_layer$ is the polynomial of the next FRI layer. This is done [here](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/prover.rs#L172).

Both of these problems are solved in STARK 101, but not explained. Explaining *why* the way we compute these is correct is the goal of this section.

#### Computing the index of $layer(-x)$
If the FRI layer has $N$ elements, and the index of $layer(x)$ is $idx$, then the index of $layer(-x)$ is $idx + (N/2) % N$. We'll explore why that is in two stages. First, we'll make sure that it makes sense by looking at actual values. Then, we'll show a proof for why it works when the prime is 17 (as is the case for our problem). This turns out to hold for all [Proth primes](https://en.wikipedia.org/wiki/Proth_prime), but we will not show that here.

The first thing to realize is that computing the index of $layer(-x)$ based on the index of $layer(x)$ is the same as computing the index of $-x$ based on the index of $x$. Make sure you convince yourself that this is the case. Remember that $-x$ is defined as $x + (-x) = 0$; that is, for any $x$, $-x$ is the field element that when added to $x$ yields $0$. 

Let's start with the first FRI layer (degree 3), the LDE domain.

$$
[3, 10, 5, 11, 14, 7, 12, 6]
$$

We can see that the relationship holds. $3 + 14 = 0$, $10 + 17 = 0$, $5 + 12 = 0$, and $11 + 6 = 0$. Great.

Let's look at the domain of the next FRI layer.

$$
[9, 15, 8, 2]
$$

The relationship holds once more. $9 + 8 = 0$, and $15 + 2 = 0$. Even though we don't compute the index of $layer(-x)$ for the last layer, let's still make sure that the relationship holds as expected.

$$
[13, 4]
$$

And indeed, $13 + 4 = 0$.

So we've confirmed numerically that our relationship holds. Next, we'll give a proof of that our relationship holds. This will give us insight into *why* it works.

Recall that the LDE domain is constructed in 2 steps

1. Construct the subgroup $D_0 = \{g^0, g^1, g^2, g^3, g^4, g^5, g^6, g^7\}$, where $g=9$
2. Shift every element in $D_0$ by 3, $LDE = \{3g^0, 3g^1, 3g^2, 3g^3, 3g^4, 3g^5, 3g^6, 3g^7\}$

To make the proof easier to follow, we will prove the relationship over $D_0$, and then finally show that shifting by 3 doesn't change anything, such that the proof applies to $LDE$ as well.

Therefore, we have $D_0$, a cyclic group of size $N=8$ with generator $g=9$.

Notice that $g^{N/2} = 9^4 = 16$. Then, $\forall i \in \{0, ..., 7\}$,

$$
g^i + g^{i+4} = g^i + g^i \times g^4 = g^i(1 + 16) = g^i \times 0 = 0
$$

Hence, $g^{i+4}$ is the additive inverse of $g^i$. But, notice that $g^i$ is at index $i$ in $D_0$, and $g^{i+4}$ is at index $i+4$. Therefore, we just showed that if $x$ is at index $i$, then $-x$ is at index $i+4$. And it follows that if $layer(x)$ is at index $i$, $layer(-x)$ is at index $i+4$!

Can you show that $3g^i$ instead of $g^i$ doesn't change this result?

Lastly, we need to show that this holds for every FRI layer, not just the first one. We'll only give a proof sketch for why this is true. Once again, we will ignore the fact that every element is shifted by 3, because it doesn't change the result and makes the proof easier to read. Remember that we construct the subsequent FRI layer as

$$
D_1 = \{g^0, g^2, g^4, g^6\}
$$

Convince yourself that the generator is $g_1 = g^2 = 13$. The size of $D_1$ is half the size of $D_0$; that is, $N_1 = N/2 = 4$.

Notice once again that $g_1^{N_1 / 2} = 13^2 = 16$. We're back where we started! That is, the generator of $D_1$, when exponentiation to half the length of the group ($N_1/2$), yields $16$. Hence, the rest of the proof applies unchanged. Finally, notice that we will get the same result once again for $D_2 = \{g^0, g^4\}$.

This completes the proof sketch. As an exercise, use this proof sketch to write a complete [proof by induction](https://en.wikipedia.org/wiki/Mathematical_induction).

#### Compute the index of $next_layer(x^2)$

TODO

### Q: How does the verifier ensure that the prover interpolated the trace polynomial over the predetermined DOMAIN_TRACE?

The STARK protocol states that the prover should

1. Run the program and record the 4-element trace $T$
2. Interpolate a polynomial $P_T$ that maps the elements of [`DOMAIN_TRACE`](https://github.com/plafer/stark-102/blob/d56243a1d37f398d417c656a9723d1e21e5f7ff3/src/domain.rs#L8) to $T$
    + That is, $P_T: BaseField \rightarrow BaseField$ is a polynomial such that 
        + $P_T(1) = T[0]$, 
        + $P_T(13) = T[1]$,
        + $P_T(16) = T[2]$, and 
        + $P_T(4) = T[3]$
3. Evaluate $P_T$ over [`DOMAIN_LDE`](https://github.com/plafer/stark-102/blob/d56243a1d37f398d417c656a9723d1e21e5f7ff3/src/domain.rs#L44-L55) and use that "extended trace" in the rest of the protocol.

This part of the protocol is crucial for the zero-knowledge property. That is, by evaluating over `DOMAIN_LDE` and committing to that instead of the evaluations over `DOMAIN_TRACE`, we ensure to not leak any of the elements of the original trace. However, how does the verifier know that the prover indeed executed step 2 properly? Specifically, how does it know that the prover used `DOMAIN_TRACE` as a domain for interpolating the polynomial, as opposed to any other domain? In other words, what part of the verifier's algorithm will fail if the prover uses a different `DOMAIN_TRACE`? Try to [answer on your own](https://terrytao.wordpress.com/career-advice/ask-yourself-dumb-questions-and-answer-them/)!

<details>
<summary>Show answer</summary>

The key is to realize that the original domain is encoded in the boundary and transition constraints. As a reminder,

+ **boundary constraint:** $C_1(x) = \frac{P_T(x) - 3}{x - \texttt{DOMAIN\\_TRACE}[0]}$ is a polynomial

+ **transition constraint:** $C_2(x) = \frac{P_T(gx) - P_T(x)}{(x - \texttt{DOMAIN\\_TRACE}[0])(x - \texttt{DOMAIN\\_TRACE}[1])(x - \texttt{DOMAIN\\_TRACE}[2])}$ is a polynomial

So what will go wrong if the prover interpolated $P_T$ over a different domain? Let's focus on the boundary constraint for the argument; the transition constraint will fail for exactly the same reason.

Well, this other polynomial (call it $H_T$) will have totally different coefficients, and will have different zeros of $P_T$. And it will almost certainly not evaluate to 0 when evaluated at `DOMAIN_TRACE[0]`, and hence $C_1(x)$ will *not* be a polynomial (why this is true is explained in STARK 101). If for the purpose of this discussion we take for granted that FRI will fail if $C_1(x)$ is not a polynomial, then verification will fail.

**Follow-up question**: Can you identify *how* FRI will fail when you feed in a $C_1(x)$ which is not a polynomial?

</details>

### Q: At query time, how does the verifier ensure that the prover sent the trace element at the requested query index?

At query time, the verifier [sends to the prover the "query index"](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/prover.rs#L81) (i.e. the index in the extended trace that the prover needs to send to the verifier). The (honest) prover sends [that value, along with a proof](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/prover.rs#L148-L150) that the value is indeed in the extended trace. However, the Merkle proof only proves that the value sent is *somewhere* in the trace, but it doesn't prove that the value is indeed the value at the queried index. How does the verifier then ensure that the prover did indeed send the value at the queried index?

<details>
<summary>Show answer</summary>

The answer lies in the fact that the verifier will use the queried index when verifying the query. Similar to the previous question, we'll only consider the boundary constraint in this answer, but the same reasoning applies to the transition constraint.

+ **boundary constraint:** $C_1(x) = \frac{P_T(x) - 3}{x - \texttt{DOMAIN\\_TRACE}[0]}$ is a polynomial

The key lies in remembering that the verifier [queries a random `x`](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/verifier.rs#L92) (derived directly from the queried index), which it uses to evaluate the boundary and transition constraints (and all the way down to the [last FRI layer](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/verifier.rs#L134)). If the prover sent `P_T(x')` (where $x'$ corresponds to a different query index), then the equations simply won't check out, and the [final check](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/verifier.rs#L134) will fail.

If this is not totally clear to you, then try it out! Write a malicious prover which when the verifier queries `query_idx`, the malicious prover supplies the value at index `query_idx + 1`. This should be a very small modification from the existing prover. Then add print statements to the verifier, run the [`proof_verification` test](https://github.com/plafer/stark-102/blob/d18fb008cd16ec68c9e4bb1ce87ec1ac5eb78bed/src/lib.rs#L72), and try to get an intuitive sense of exactly *where* and *why* things go wrong. Notably, confirm that the Merkle proof verification checks out just fine.

</details>

## Exercise to the reader
There's nothing like getting your hands dirty to truly understand something, and hence this exercise. We modify the original statement to:

I computed the following sequence:

$$
\begin{align}
& a_0 = x            \\
& a_{n+1} = (a_n)^2  
\end{align}
$$

over the prime field `F_p` with prime 17, for some public `x ∈ F_p`.

Essentially, modify the codebase to make the first value in the sequence any value in the set `{0, ..., 16}`. Or in other words, any element of `BaseField`. Notably, this will require you to
+ Change `constraints::boundary_constraint()` to take a parameter `x: BaseField`
    + Hint: you will need to implement polynomial division
+ We can now make `Channel::new()` to take the parameter `x: Basefield`, and initialize the hash using `x` as opposed to `CHANNEL_SALT`.
