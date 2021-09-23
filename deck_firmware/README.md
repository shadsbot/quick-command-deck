## Deck Firmware

\* name not final

Half of the project is the physical hardware itself. This is where the firmware for the Arduino Nano lives. It doesn't have to be an Arduino Nano. The code should just be C/CPP and should be transferrable to any SBC with a little work, that's just what I'm using for this.

### Firmware?

Regardless of the debate of what is and is not firmware, this is software that lives on the microcontroller and should be pretty permanent once finished. Depending on who you ask, this isn't firmware and I can be inclined to agree. For now, that's just what I'm calling it because it's a clear distinction between this and the software being written for the other end.

### Libraries Used
- [nanopb/nanopb](https://github.com/nanopb/nanopb) - I've included the files for this because setting it up for the IDE I'm using was kind of a pain. nanopb is licensed under the [zlib license](https://github.com/nanopb/nanopb/blob/master/LICENSE.txt) (reproduced below), which allows modification and distribution of their code. It's included in this codebase as a submodule with symlinks to the used files. You'll need to check it out with `git pull --recurse-submodules`, or with `git clone --recurse-submodules` during the initial clone. 
    > Copyright (c) 2011 Petteri Aimonen <jpa at nanopb.mail.kapsi.fi>
    >
    > This software is provided 'as-is', without any express or 
    > implied warranty. In no event will the authors be held liable 
    > for any damages arising from the use of this software.
    > 
    > Permission is granted to anyone to use this software for any 
    > purpose, including commercial applications, and to alter it and 
    > redistribute it freely, subject to the following restrictions:
    > 
    > 1. The origin of this software must not be misrepresented; you 
    >    must not claim that you wrote the original software. If you use 
    >    this software in a product, an acknowledgment in the product 
    >    documentation would be appreciated but is not required.
    > 
    > 2. Altered source versions must be plainly marked as such, and 
    >    must not be misrepresented as being the original software.
    > 
    > 3. This notice may not be removed or altered from any source 
    >    distribution.

### Notes About Hardware
For the Cherry MX Brown switches, the lower of the two pins is what's connected to ground.