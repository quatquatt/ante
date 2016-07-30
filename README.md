﻿# Ante
The compile-time language

## Features
* Systems language that feels like an interpreted language
* Expression-based syntax, no statements
* Support for functional, imperative, and object-oriented paradigms
* Strongly typed with a detailed algebraic type system and type inferencing
* Full control given to users allowing them to specify specific requirements for a given
type and issue a compile-time error if it is invalidated
    -  Extremely diverse and powerful compile-time analysis that can be custom programmed into
any datatype creating eg. iterator invalidation, pointer-autofree, or even an ownership system.
    - These compile-time functions are checked at compile-time and not compiled into the binary.
* Module system allowing the setting of compiler flags on a per-module basis.
```go
var i = 55        ~create i, a mutable 32-bit integer

~Create j, an immutable integer
let j = 0

let myTuple = (5, 5.0, "five")

~tuples can also be destructured and stored into multiple variables
let (x, y) = (4, 5)

~Arrays:
var myArray = [0, 1, 2, 3, 4]

~Return type inference:
fun add: i32 x y = x + y

~Sum types:
type Maybe =
    Some 't | None

var f = Some 4
f = None
```
* Significant whitespace after newlines; no tabs allowed in significant whitespace.
```go
fun myFunction:
    if 3 > 2 then
        print("3 is greater than 2")
    else
        print("Invalid laws of mathematics, please try again in an alternate universe")


~Significant whitespace is purely optional, though recommended
fun myFunction: {
    if 3 > 2 then {
        print("3 is greater than 2")
    }else{
        print("Invalid laws of mathematics, please try again in an alternate universe")
    }
}
```
* Reference counted smart pointers by default while keeping the ability to create raw pointers
* Unique pointers used whenever possible automatically
* No more memory hassle trying to find cycles with pointers, everything is done by the compiler
* No garbage collector
```go
let intPtr = new 5
let strPtr = new "msg"

~Declaration of raw pointers is accomplished with the 'raw' modifier:
let raw myPtr = malloc(10)

~intPtr is automatically freed
~strPtr is automatically freed
free(myPtr) ~myPtr must be manually freed
```
* API designers given full reign to implement custom rules for their types, full access to the (immutable)
parse tree is provided, along with a quick list of the uses of the variable in question.
```go
~Generic types are implemented with type variables, to specify a type variable, identified by a '
fun iteratorTest: 't iter
    ~ok!
    for i in iter do print(i)

    ~assert the presence of a compile-time error generated from iterator invalidation
    assertErr(
        fun = for j in iter do print(j))
```

* Here is an example implementation of a thread that 'owns' the objects inside its function
```
type MyThread = 'f fn, Pid pid

ext MyThread
    fun run: self*
        self.pid = Threads.exec(self.fn)

    
    ~Compile time function that runs whenever MyThread is created
    pri fun handleInputs(onCreation): self
        
        let params = Ante.requestParamsOf(self.fn).unwrap()

        ~The ':' operator denotes a method where the object is mutated.
        let vars = Ante.getVarsInFn(self.fn).unwrap():remove(params)

        ~Store these vars for later use in compile-time
        ~NOTE: this is a local store where 'vars' may only be used by functions of MyThread,
        ~      for global stores, more unique names should be used
        Ante.store(vars)

        ~finally, invalidate the variables for any use whatsoever while this thread 'owns' them
        vars.iter( Ante.tmpInvalidate(_) )

    pri fun cleanup(onDeletion): self
        let vars = Ante.lookup("vars").unwrap()

        ~revalidate all the variables used
        vars.iter( Ante.revalidate(_) )
```
* Explicit yet concise currying support
```go
let increment = _ + 1

print(increment(4)) ~prints 5

let f = _ + increment(_)

f(3) |> print
~output: 7

~filter out all numbers that aren't divisible by 7
let l = List(0..100):filter(_ % 7 == 0)

```

* For more information, check out tests/language.an for all planned features.


## Installation
1. Make sure to have `llvm` version >= 3.6 installed.  To check which version you have, run `$ lli --version`.  To install llvm, install the `llvm` package on your distro's package manager, eg. for Ubuntu: `$ sudo apt-get install llvm`

2. Run `$ git clone https://github.com/jfecher/ante.git`

3. Run `$ cd ante && make && sudo make stdlib`

    - NOTE: root permissions are only needed to export the standard library.  To export it manually, execute the following command as root:

        `# mkdir -p /usr/include/ante && cp stdlib/*.an /usr/include/ante`
