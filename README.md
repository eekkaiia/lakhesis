Lakhesis
========

Klotho spins the thread of life, Lakhesis measures it, Atropos cuts it short - the [Moirai][1]

![Lakhesis single sandpile](/images/Lakhesis_0949847.png)

Description
-----------

`lakhesis` is a Rust implementation of the Abelian sandpile model on a rectangular lattice that seeks to explore the interaction between two or more sandpiles. In this implementation of the sandpile model grains of sand fall onto a surface, imagined here as a square table, which accumulate to a height of three grains per cell. Adding a fourth grain causes the pile to topple and sends the  grains to the four adjacent cells. If these adjacent sand grain piles exceed three grains, they will in turn also tumble and send sand grains to their neighbors, possibly causing an 'avalanche' of tumbling piles. Sand grains reaching the edge of the table will 'fall off' and are considered lost. The model has been used to study self-organizing criticality.

![Lakhesis multiple sandpiles](/images/Lakhesis_3325373.png)

During experimentation with the model it was noted that adding a second sand pile resulted in interesting, and recurring, patterns at the boundary between the two piles. Lakhesis has been set up in order to explore how these boundary patterns develop and how factors such as the number of additional piles and their orientation and distance from each other affect these patterns. The lattice is set to 2,700 by 2,700 pixels, large enough to contain a single sandpile of approximately 20 million sand grains. Cells with three grains are set to transperant black in order to emphasis the patterns that cross through these stable areas. Many of these patterns have the appearance of threads intertwining and weaving into textile-like surfaces (i.e. the name of the crate - apologies for the mixed metaphors).

![Lakhesis threads](/images/LakhesisThread.GIF)    [Take a look at the web version][7]

Keywords
--------

[Abelian sandpile model][2], [Cellular automaton][3], [Self-organized criticality][4], [Rust][5], [Macroquad][6]

Requirements
------------

`lakhesis` was compiled using Rust 1.63.0 

Dependencies
------------

* macroquad

Usage
-----

```bash
cargo run --release
```

There are no command line arguments.

Keyboard commands are available after execution starts and are shown in an information box at the top left corner of screen.

* [A] Add a new sandpile to the simulation. Add up to a maximum of 32 sandpiles.
* [C] Change the colors displayed with a randomly selected new set of colors.
* [I] Bring up the info panel if its hidden. Hide the panel if its visible.
* [M] Magnify a 32 by 32 pixel square by a factor of 4. Use mouse to select location.
* [P] Pause the simulation. Press [P] again to resume simulation.
* [S] Save an image of the lattice as a PNG. The file is saved to the project folder or the folder containing the executable.
* [Spacebar] Step through the simulation one interval each time the [Spacebar] is pressed.
* [Up] The up arrow increases the interval between screen updates by a factor of 4 to a maximum of 16,384.
* [Down] The down arrow decreases the interval between screen updates by a factor of 4 to a minimum of 1.
* [CTRL|N] Start a new simulation - not shown on the WASM version. Use the browser's reload command.

The `macroquad` game engine can be compiled to run on web browsers - directions are here: [https://github.com/not-fl3/macroquad]

Before compiling `lakhesis` for web usage it is suggested that the IO_SUPPORTED constant in "main.rs" be set to "false". This will remove the [S]napshot and [CTRL|N] options which conflict with web browser commands.

Rust provides a local server that can be used to host the webpage on your system.  After following the `macroquad` directions to create a wasm file open a terminal in the project folder:

```sh
cargo install basic-http-server
basic-http-server
```

Comments
--------

When the simulation starts a light gray dot indicates the centerpoint of the 2,700 by 2,700 lattice so that a new sandpile can be started in the middle of the lattice. The point is just a guide - new sandpiles can be started anywhere within the lattice. If the model is run with only one sandpile, it will generate the standard Abelian image seen at the top of this page. Adding additional sandpiles will generate images similar to the second image above. Pressing [C]olor randomly changes the colors used the by model. Generating new colors doesn't impact any other aspect of the model and they can be repeatedly changed, however values of the old colors are not saved. Adding a color palette could be a future update.

A control panel in the `lakhesis` window provides a map of keyboard commands and information on the model. To keep the labels short the description for the up and down arrow keys are a little misleading. When Lakhesis starts it defaults to refreshing the screen every 1024 interations (an interval of 1024 sand grains that have been added to the model). The display interval can be changed by a factor of 4 down to a minumum of 1 (refreshing the screen for each sand grain added) and up to a maximum of 16,384. Initially, more frequent screen updates slow down overall progress of the model, however the interesting patterns mentioned above are more apparent at smaller intervals. Conversely, increasing the interval between updates can 'speed' up the model to get to the point where the edges of sandpiles interact, at which point the interval can be decreased to observe the interference patterns. As the number of sand grains becomes substantial the simulation will become less responsive as more time is needed to evaluate the model. At this point the time needed to refresh the screen is inconsequential compared to the evaluation time and the interval should be reduced to make the model more responsive to keyboard commands. When frames per second (FPS) drops to 0, the "Last frame time" and "Longest frame time" variables can help estimate how long it will be till the next screen update.

There are two undocumented features. The background color can toggled between black (default) and white by pressing the [B] key. Pressing [CTRL|V] will export a large number of PNG images at the set interval. [V] is for video, however the command doesn't actually create a video, rather it exports an image of the visible portion of the model at a constant interval. The number of images is set to 600 by a constant (VIDEO_FRAME_COUNT) in "main.rs". These 600 images could, for example, be animated with video editing software to produce a 10 second video at 60fps. WARNING - using the video command will dump 600 images in your project folder. Pressing [ESC] will cancel the command. The [S]napshot command works a little differently - it exports a PNG image of the active portion of the lattice with a 10-pixel blank boundary around the edges - even if whole image is not visible on the screen. The boundary and the 'black' triangular areas in the image are actually transparent and can be easily manipulated with image editing software.

My contact info is in the "cargo.toml" file.

License
-------

The content of this repository is licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)

[1]: https://en.wikipedia.org/wiki/Moirai
[2]: https://en.wikipedia.org/wiki/Abelian_sandpile_model
[3]: https://en.wikipedia.org/wiki/Cellular_automaton
[4]: https://en.wikipedia.org/wiki/Self-organized_criticality
[5]: https://www.rust-lang.org
[6]: https://macroquad.rs
[7]: https://eekkaiia.github.io/lakhesis
