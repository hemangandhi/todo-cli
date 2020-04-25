# A Theoretical Background on Rust

This is a opinionated lay of the land, explaining the scenery around which Rust has been built. Rust has not been created in a vacuum and knowing the ideas that existed before should guide people, hopefully, to two things: better ideas, and an ability to accept steady change.

So, the landscape.

## Simon Peyton Jones' Graph of Languages

Simon Peyton Jones has a graph he likes to use as a mental model for programming languages. So far, this chart holds true, particularly with today's trends. We have:

| Case | Unsafe | Safe |
| Fast | C, C++ | (Almost) Rust |
| Slow | N/A | Haskell |

(The languages are an approximation and we mustn't stop dreaming just because of how awesome Rust is. It isn't everything yet, as we may see later on.)

The graph is really about a process over time: aliasing became common, and has led to struggles with safety but it is a very powerful tool. Over time, the side favoring speed has realized that its lack of safety is too costly and the side favoring safety has come to realize that it is too slow for many applications. Rust isn't really some gloarious treaty between the warring factions: somebody independently solved many of the problems found on the speed-favoring side by applying the techniques used in safer languages.

We will follow a similar lay out in this lengthy aside, discussing the theory from safer languages before applying them to practical issues we see in C++. There will then be some discussion on the missing parts and more hypotheses on why other solutions were not chosen.

In order to explain this world Rust is in, there are three sections:

1. Some type theory
1. Some rants against C++ and Haskell
1. Extensions to logic that Rust may or may not yet support

## Some Type Theory

This is too broad and quick an overview of some type theory concepts that make good type systems an absolute joy to read. This goes over the basics of how type theory views types and then tries, very quickly, to apply that view to Rust.

### Predicates as Types

A first difference in perspective is to fundamentally shift what code means. To some enveloped in the theory of coding and proofs, code is a proof. This doesn't (yet) mean that all proofs have a corresponding program, or that it is useful to thing of all programs as the proof of some theorem. This frame of mind helps in the smaller functions and helpers one would write.

In particular, this approach converts types into facts. The basic facts are "I can make a T" where T is a type. If you have a tuple "(T, U)", you know that you can make a T and a U. Furthermore, if you have some sum type:

```rust
pub enum Either<T, U>{
    Left(T),
    Right(U)
}
```

you make a T or a U. Furthermore, functions `T -> U` are implications: once you have a T you can get a U.

Simply put then, every function is a proof of some implication. More concretely, it is a construction (this is the key limitation of type theory in mathematics: that it can't frame the non-constructive matters simply).

Concretely, this means that a function like:

```
pub fn discard_error<T, E>(in: Result<T, E>) -> Option<T> {
    match in {
        Ok(t) => Some(t)
	Err(_) => None
    }
}
```

is a proof that for all result types, there is a corresponding option type. The function body is the following proof:

> Let in be a Result<T, E>. Then, WLOG, it is an Ok(t) for some T t or an Err(e) for some E e. Suppose it is an Ok(t). We can then construct Some(t) as an Option<T>. Otherwise, we can construct None as our option type.

Of course, this is a little meaningless at this level. But, this means that there is a strong contract enforced in the function signature. In particular, we can read a library function's signature to understand what it does. For example, here's `fold`'s signature in Rust:

```rust
// Self is an Iterator
fn fold<B, F>(self, init: B, f: F) -> B
where
    F: FnMut(B, Self::Item) -> B
```

This tells us a few things right off the bat: we need an iterator instance (which we consume), we need a `B` and an `f` that can combine a `B` with the `Item` type we're iterating over to get a new `B` and this will give us a `B`.

Naturally, providing the inductive proof of how `fold` is actually implemented is beyond the scope of Rust's type system. In fact, only extensions of Haskell (or a real proof assistant like Coq, LEAN, Idris, or Adga) could dare take on such a task. However, we can build some smaller gadgets that are rather neat, like fixed-length arrays where the compiler can verify that concatenation indeed adds the sizes (Rust is not yet capable of this cleanly -- I will get back to you after a lot of experimentation).

The main take-away, though, is that theoreticians don't look at types as mere markers like we do, but as propositions about the reality of the world the code is modelling.

### Impls as Theorems

This is really the juicy part. Let's say we have a trait. A trait really is a definition of a class of objects -- like "numbers" or "lists". And the nicest part about Rust's `impl` statement is that it proves theorems about traits. For instance, if the following were an `impl`:

```rust
impl<T: Ord> Ord for ToDoList<T> { ... }
```

we'd be saying:

> For all T that can be totally ordered, the ToDoList<T> can be totally ordered.

This puts us in even better shape for saying general things about our model of the world and having an automatic verification tool (`rustc`, our best friend).

### GADTs, HKTs and the Rest

The above are the concepts Rust fully implements. This section is to mention somethings that remain.

We talk [about sum types](General-Asides.md#aside-sum-types) at length. Product types are structs or tuples. These are fully implemented by Rust and are called **A**glebraic **D**ata **T**ypes. Here we will extend them some.

In particular, what is a `ToDoList<T>`? It's not just the product of some `Vec<T>` and a `String`. For all `T`, the `ToDoList<T>` is a separate type (each perhaps with its own properties). To generalize this, we look into HKTs.

To start with HKTs, we can ask a foundational question. Where do all the types live? This, in set theory, is a problem, and so that Bertrand Russell doesn't haunt us, we use a layered universe in type theory. In particular, types inhabit a universe. We know these types: they're the ADTs mentioned above or functions. But, to keep our theory simple, universes should really be types too (lest we be set theorists and come up with a new word everytime we need an infinity that much fundamentally larger). We can do this. If the universe, U0 is a type, we put it in U1, a larger universe of types. We then have an infinite hierarchy and all types we can talk about fit in there somewhere. In particular, we can talk about functions to and from Un. These would live in U(n + 1). In fact, Haskell does this and we shall use its notation hereafter.

Let us define Haskell's equivalent to Rust's `Result`:

```hs
data Either a b = Left a | Right b
```

If we ask for what the type of `Either` is (pronounced `:k Either` on ghci, Haskell's interactive REPL), we see `* -> * -> *` which really means it's an arity 2 higher-kinded type (HKT). What elevates this "kind" so that we call it "higher"? The level of the universe it must inhabit!

Rust has no built-in formalism to discuss this. In particular, in Haskell, we can partially invoke HKTs (currying, as the acolytes know) and state facts about them:

```hs
instance Monad (Either e) where
  Left l  >>= _ = Left l
  Right r >>= k = k r
```

This is to say (as we try to earlier) that for a fixed error (`Left e`), we have a Monad instance. Rust cannot express this without explicitly involving the error type. Furthermore, in Haskell, we don't have to state the arity too explicity in the theorems (link in the `instance` block above) until we have to provide the constructions in the implementations themselves.

GADTs are also thought of as dependent types. These are not just higher-kinded types, but those that have lower type parameters. This would be something like `FixedVec: Int -> * -> *`. That is, the type is distinguished by a value from another type. This means that the type checker can, while checking types, make statements about _values_. This is the holy grail of convenient verifiable coding. There is more to verifying all code and this is really a frontier in programming language (and even mathematical in some circles) theory.

(In a cruel twist of fate, C++ templates do allow this. [Example here.](https://gist.github.com/amrali/716d4c342a8f7fc3f23fee8c2b82e300 "And honestly, it's not even ugly."))

## Some Rants Against C++ and Haskell

This is going in-depth on why the status quo actually needed fixing.

### Rant about C++

C++ is complicated. It has a few really good excuses to be, but nobody knows all of it and it's beyond the point where anybody should. In fact, most recommendations leave you with a completely usable subset of the programming language. The rest is relegated for only desparate needs.

Worse yet, C++ is unsafe. And not just in the sense that "Rust is safe -- yay!" We need unsafe code sometimes, but in the sense that Rust uses `unsafe` and C++ drops the distinction between safe and unsafe code in this sense. Really, it means that all code C++ is unsafe. This isn't to say all C++ code is wrong (after all, we're not yet extinct), but that there's no distinction between parts that the compiler has verified and parts where it's trusting the programmer to do the right thing.

Rust has this distinction: it is very unlikely to have a memory error in safe code. And it lets you have full control in distinguished `unsafe` blocks if you need it.

Hence, simply, C++ is fast but dangerous.

### Rant about Haskell

Haskell is harder to want to rant about, ["avoiding success at all costs,"](https://youtu.be/re96UgMk6GQ?t=715 "A bit more than needed, but Simon Peyton Jones is a personal hero, so ignore me and watch his talks.") it does not have the goals that Rust does. In fact, none of these complaints should tell a Haskell developer to necessarily bother with Rust. It depends on their goals -- in particular: do they care about speed?

Haskell is problematic in this way: it exists for mathematical interest, really bridging the gap between proof assistants and executable code. However, in its purity, it is very difficult to profile. Haskell tries to execute as one would like to execute a mathematical proof: it only evaluates what it absolutely must. Hence, Haskell lists can be infinite and the code can be as slow as one pleases. Here, for instance is a linear search to find the mininal value in an array:

```hs
qs :: (Ord a) => [a] -> [a]
qs []     = []
qs (x:xs) = (filter (< x) xs) ++ [x] ++ (filter (>= x) xs)

min = head . qs
```

The name `qs` may not have been leading enough: it means quick-sort and we will violate the purpose of Haskell and ponder its performance. To find the min is actually linear since to get the `head` (zeroth) element of the list, we have to compute  only the first value, so we only try the left hand side of `++` (the concatenation operator for lists). Recursively, this amounts to comparing each `x` in the list only once. However, `qs`, if evaluated in its entirity, is quadratic time: the `++` operator is linear to compute and we must do such a computation per element of the list.

This is infeasible for critical code. Rust, while losing some of the mathematical purity and elegance of Haskell, does gain a lot of speed and deterministic performance.

## Extensions

This is the most free-form of the sections: wherein I mention all sorts of various topics in mathematical logic that may or may not become relevant to Rust, but is a great influence in ways of thinking about programming languages, particularly future ones. (The antecedent of the _I_ here is Heman -- I've been tasked to add some personal notes, so why not just use first person so you know who's bitching about the theory?)

### Rust and HKTs and GADTs

Full disclosure: I've tried this. [See here.](https://github.com/JasonShin/fp-core.rs/blob/master/fp-core/src/hkt.rs) These are a mess since Rust has yet to really support this. The current progress is documented [here](https://github.com/rust-lang/rfcs/issues/324).

For GADTs, [phantom types](https://doc.rust-lang.org/stable/rust-by-example/generics/phantom.html) are useful since they do not appear at runtime, so the abstraction is (wait for it...) zero cost! This means we can have phantom types that act similarly to the values we'd want to use. In particular, the following (advanced) insanity is supported in [a crate](https://docs.rs/type-operators/0.3.5/type_operators/). This insanity gives us numbers:

```rust
pub trait Nat {
    fn reify(Self) -> u32;
}

// Zero is a number
pub struct ZNat;
impl Nat for ZNat {
     fn reify(self) -> u32 { 0 }
};

// 1 + a number is a number (the successor)
pub struct SNat<A: Nat = ZNat>(<A>);
impl<A: Nat> Nat for SNat<A> {
    fn reify(self) -> u32 {
        let Self(n) = self;
        1 + reify(n)
    }
}
```

So GADTs are just weaker and messier in Rust, but not inaccessible per say.