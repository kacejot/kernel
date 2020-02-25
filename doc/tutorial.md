# Rpi3 OS on Rust

## Why Raspberry Pi 3?
Raspberry Pi is a cheap fully functional mini-computer based on the ARM processors. Many operating systems already support this board, but the reason why I chose it for OS development is because it was designed to teach students computer science:

1. There are a variety of hardware devices assembled on one compact board making it comparable with a PC by functionality.

2. Running custom software is simple - Raspberry Pi is booted from microSD card, so you can just put your software on it without writing a bootloader.

This board is very popular and has a large community. [This forum domain](https://www.raspberrypi.org/forums/viewforum.php?f=72) will be helpful for those who want to learn Raspberry Pi bare metal development. 
Also, I’m new to ARM architecture, so this is an experience for me as well.

## Why Rust?
1. Rust is fast.<br/>
It is comparable with C and C++ by speed, because it is compiled to the target machine native code. The language is young, but many platforms are already supported. The list of supported platforms can be found [here](https://forge.rust-lang.org/release/platform-support.html). <br/>
The language has as minimal runtime as possible (panic_handler and global_allocator), detailed information about runtime can be found [here](https://doc.rust-lang.org/reference/runtime.html).

2. Rust is reliable.<br/>
Rust is designed with the purpose of early error detection. And it performs well - most memory safety errors, data races are detected during compile time. That makes compiling code with potential memory leak impossible.

3. Rust has a large knowledge base with free access. <br/>
Books, articles repos with examples and exercises, all of this can be found [here](https://www.rust-lang.org/learn).

4. Cargo. <br/>
This beast can do everything for your project management: 
    1. Project creation includes hello-world program, initialized git repo, minimal dependency file etc.
    2. One-click build.
    3. Code linter. 
    4. Code formatter.
    5. Dependency manager.

5. Rust is a cross-platform language. <br/>
That means that one code could be compiled on different architectures and systems.  <br/>
Rust compiler does not implement the Rust programming language standard. It is standard. That solves some problems with portability between compilers like C and C++ have.

6. Compared to C++, Rust has more features on bare metal. 
There is no opportunity to use standard libraries of both languages, so Rust uses libcore instead. It is a subset of libstd. It does not provide heap operations, IO and concurrency, because it can not make any assumptions about the system it is being run on. All in all, it allows you to use a lot of Rust features like iterators, strings formatting, base memory operations etc. On the other hand, C++ on bare bones has a lot of limitations. Most of them are described [here](https://wiki.osdev.org/C_PlusPlus).

## OS Kernel basics
Kernel is mainly developed in several stages:
1. Minimal kernel
2. CPU interrupts 
3. Memory management

In this article we will observe the first stage - minimal kernel. For this one we should write some kind of “hello world” program of kernel development world. As “hello world” I mean software the only purpose of which is handling input and performing output. It will echo all typed characters from the keyboard back to the terminal from which input was performed. 
### Development plan
We should choose the device from which kernel should receive keyboard input and the device to show output. As for input we could choose a USB device and connect the keyboard to it. For output HDMI could be fine.<br/>
But there is one problem - our aim is to write logic that describes kernel IO processing. Writing drivers for USB, keyboard and HDMI will take a lot of time. Instead we will use two GPIO pins. One is for input, another is for output. The data transmission will be done using an UART device. So, we will implement a simple kernel and two drivers to interact with the outside world.

### no_std binary
First of all we need to create a `no_std` binary project. As I mentioned before, we can’t use some features of libstd such as heap allocation, IO and concurrency. All the parts of libstd that use these features are also inaccessible in no_std project. libcore is used instead.

`$ cargo new kernel`

This command will create “hello world” Rust binary project. We need to remove all the code in `main.rs` add next inner attributes to our main module. `main.rs`:
```Rust
#![no_std]
#![no_main]
```
If we try to compile this code we will get next error:<br/>
```error: `#[panic_handler]` function required, but not found```

So, let us define the panic handler that will only spin the core:
```Rust
use core::panic::PanicInfo;
 
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
   loop {}
}
```
This is the only item of language runtime we will use in our kernel and it is fully controllable from the programmer side. We will modify it to do graceful panic handling process with logging of the panic info to the console later.

### kernel abstraction

The next step is to write some kernel code that is independent from board and responsible for driver loading and IO operations.<br/>
<br/>
Driver abstraction is pretty easy to write, because we just need to initialize our GPIO and UART devices, so `Driver` trait will contain `init` method. Also we want to know which driver we use, so we will add string identifier using method `name`. <br/>
The contents of `driver.rs`:
```Rust
pub trait Driver {
    fn init(&self) -> Result<(), &'static str> {
        Ok(())
    }

    fn name(&self) -> &str;
}
```

For IO operations we need do more work. As I mentioned before, `libcore` does not provide IO operations, so we need to implement the IO abstractions by ourselves... Or make a modified version of `libstd` IO abstractions. I will migrate only `Read` and `Write` traits from `std::io`. The reason why there aren't such traits in `libcore` library is their dependency on heap allocations in Error type and other minor dependencies on runtime related to OS defined operations. The part of IO is already proposed to migrate to `libcore`, you can see the details [here](https://github.com/rust-lang/rfcs/issues/2262). <br/>

Okay, so we need Read trait.<br/>
Contents of `read.rs`:
```Rust
use crate::kernel::io;

pub trait Read {
    // Error type is associated now to avoid 
    // dependencies from heap allocations, so 
    // we can choose the implementation on 
    // the implementor's side.
    type Err;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Err>;

    fn read_exact<E>(&mut self, mut buf: &mut [u8]) -> Result<(), E>
    where
        E: From<Self::Err>,
    {
        // default implementation. see full sources
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    fn chain<R: Read, E>(self, next: R) -> io::Chain<Self, R, E>
    where
        Self: Sized,
        E: From<Self::Err> + From<R::Err>,
    {
        io::Chain::new(self, next)
    }

    fn take(self, limit: u64) -> io::Take<Self>
    where
        Self: Sized,
    {
        io::Take::new(self, limit)
    }
}
```
And Write trait is implemented in the same way.<br/>
Contents of `write.rs`:
```Rust
use crate::kernel::io;
use core::fmt;

pub trait Write {
    type Err; 

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Err>;

    fn write_all<E>(&mut self, mut buf: &[u8]) -> Result<(), E>
    where
        E: From<Self::Err>
    {
        // default implementation. see full sources
    }

    fn write_fmt<E>(&mut self, fmt: fmt::Arguments) -> Result<(), E>
    where
        E: From<Self::Err>,
    {
        // default implementation. see full sources
    }
}
```
This ones are the carbon copy of `libstd` trait version, I just made associated Error type and removed methods that rely on types that use heap allocations.<br/>
Our kernel code is only missing the entry point.<br/>
`kernel.rs` contents:
```Rust
pub mod io;
pub mod driver;

pub fn init() -> ! {
    loop{}
}
```

`io` is a module that contains code with `Read` and `Write` traits. `driver` is a module with `Driver` abstraction.
I left infinite loop for beginning. I will modify this part as soon as we have drivers ready.

### board supply package

We have done with main part of kernel abstractions. It it time to write some platform-dependent code. First of all we need an entry point to our kernel image.<br/>
<br/>
In RPi 64-bit CPUs kernel is loaded at `0x80000` address, so we need to create linker script file that describes this behavior.<br/>
Contents of `link.ld`:

```Linker Script
SECTIONS
{
    . = 0x80000;

    .text :
    {
        *(.text._start) *(.text*)
    }
}
```

This one says that code should be loaded at `0x80000` address. And `.text` section is stored by that address. And the first symbol in `.text` is our kernel entry point. All the space after this symbol is a kernel code. This should be enough to link our little kernel.<br/>
<br/>
The next step is writing the entry point in Rust. Such languages as Rust and C++ use name mangling to be able to support member functions and function overloading (only C++). We need to disable this feature only for our entry point to have the same symbol compiled as described in the linker script.<br/>
Contents of `bsp.rs`:

```Rust
use crate::kernel;

#[no_mangle]
extern "C" fn _start() -> ! {
    kernel::init()
}
```

Okay, now kernel imange should be linked correctly.
The last and the most important thing we need to do is writing GPIO and UART drivers.<br/>
<br/>
Rasperry Pi 3 has 2 UART devices: mini UART and PL011 UART. PL011 UART is connected to the Bluetooth module, while the mini UART is used as the primary UART. But in fact we can use PL011 UART with GPIO 14 & 15 pins using alternative function configuration for this device. <br/>
The main driver logic is implemented by Andre Richter. You can find his writing OS tutorial [here](https://github.com/rust-embedded/rust-raspi3-OS-tutorials). I've modified UART driver to implement our kernel::io traits, so we can use it like simple console with read and write operations.