Lakhesis
========

Klotho spins the thread of life, Lakhesis measures it, Atropos cuts it short. [Wikipedia][1]

![Lakhesis single sandpile](/images/Lakhesis_0949847.png)

Description
-----------

`lakhesis` is a Rust implementation of the abelian sandpile model on a rectangular lattice that seeks to explore the interaction between two or more sandpiles. In this implementation of the sandpile model grains of sand fall onto a surface, imagined here as a rectangular table, which accumulate to a height of three grains per cell. Adding a fourth grain causes the pile to topple and sends the  grains to the four adjacent cells. If these adjacent sand grain piles exceed three grains, they will in turn also tumble and send sand grains to their neighbors, possibly causing an 'avalanche' of tumbling piles. Sand grains reaching the edge of the table will 'fall off' and are considered lost. The model has been used to study self-organizing criticality.

![Lakhesis multiple sandpiles](/images/Lakhesis_3325373.png)

During experimentation with the model it was noted that adding a second sand pile resulted in interesting, and recurring, patterns at the boundary between the two piles. Lakhesis has been set up in order to explore how these boundary patterns develop and how factors such as the number of additional piles and their orientation and distance from each other affect these patterns. The lattice is relatively small (1280 x 720 pixels - 720p), again envisioned as a table that sand grains can fall off once they reach the edge. Piles with three grains are transperant black in order to emphasis the patterns that cross through these stable areas. Many of these patterns have the appearance of threads intertwining and weaving into textile-like surfaces (i.e. the name of the crate - apologies for the mixed metaphors).

![Lakhesis threads](/images/LakhesisThread.GIF)

Keywords
--------

[abelian sandpile model][2], [cellular automaton][3], [self-organized criticality][4], [rust][5], [macroquad][6]

Requirements
------------

`lakhesis` was compiled using Rust 1.63.0 

Dependencies
------------

* macroquad

Usage
-----

```bash
cargo run -- release
```

There are no command line arguments.

Keyboard commands are available after execution starts and are shown below the sandpile simulation:

* [A] Add a new sandpile to the simulation. Add up to a maximum of 32 sandpiles.
* [C] Change the colors displayed with a randomly selected set of colors.
* [P] Pause the simulation. Press [P] again to resume simulation.
* [S] Save an image of the current screen as a PNG. The file is saved to the project folder or the folder containing the executable.
* [Spacebar] Step through the simulation one interval each time the [Spacebar] is pressed.
* [Up] The up arrow increases the interval between screen updates by a factor of 4 to a maximum of 65,536.
* [Down] The down arrow decreases the interval between screen updates by a factor of 4 to a minimum of 1.
* [M] Magnify a 32 by 32 pixel square by a factor of 4. Use mouse to select location.
* [V] Exports an image at the set interval 600 times.
* [CTRL|N] Start a new simulation
* [CTRL|Q] Quit the program

The `macroquad` game engine can be compiled to run on web browsers:  [https://github.com/not-fl3/macroquad]

Before compiling `lakhesis` for web usage it is suggested that the IO_SUPPORTED constant in main.rs be set to "false". This will remove the [S]napshot and [V]ideo options since attempting to save a file from within a web browser will cause a fatal error.

A `rust` local server can be used to host the webpage.  After following the directions above open a terminal in the project folder:

```sh
cargo install basic-http-server
basic-http-server .
```

Comments
--------

When the model simulation starts a white dot indicates the center of the 1280 x 720 pixel 'table'. If the model is run with only one sandpile, it will generate the standard Abelian image seen at the top of this page. Adding additional sandpiles will generate images similar to the second image above.

A control panel in the `lakhesis` window provides a map of keyboard commands and information on the model. To keep the labels short the description for the up and down arrow keys are a little misleading. When Lakhesis starts it defaults to refreshing the screen every 1024 interations (an interval of 1024 sand grains that have been dropped). The display interval can be changed by a factor of 4 down to a minumum of 1 (refreshing the screen image for each sand grain dropped) and up to a maximum of 65,536. Initially, more frequent screen updates slow down overall progress of the model, however the interesting patterns mentioned above are more apparent at these intervals. Conversely, increasing the interval between updates can 'speed' up the model to get to the point where the edges of sandpiles interact, at which point the interval can be decreased to observe the interference patterns. As the number of sand grains increases the simulation will become less responsive, especially if the interval is set at the higher end. When FPS drops to 0, the frame time variable will provide an estimate of how long it will be till the next screen update.

The video command doesn't actually create a video, rather it exports a number of frames as PNGs at the current screen update interval. The number is set to 600 by a constant (VIDEO_FRAME_COUNT) in main.rs. These 600 images could, for example, be animated with video editing software to produce a 10 second 720p video at 60fps.

License
-------

The content of this repository is licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)

[1]: https://en.wikipedia.org/wiki/Moirai
[2]: https://en.wikipedia.org/wiki/Abelian_sandpile_model
[3]: https://en.wikipedia.org/wiki/Cellular_automaton
[4]: https://en.wikipedia.org/wiki/Self-organized_criticality
[5]: https://www.rust-lang.org
[6]: https://macroquad.rs
