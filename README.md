Lakhesis
========

Klotho spins the thread of life, Lakhesis measures it, Atropos cuts it short. [Theoi][1] [Wikipedia][2]

![Lakhesis image](/images/Lakhesis_image.jpg)

Description
-----------

Lakhesis is a Rust implementation of the abelian sandpile model on a rectangular lattice that seeks to explore the interaction between two or more sandpiles. In this implementation of the sandpile model grains of sand fall onto a surface, imagined here as a rectangular table, which accumulate to a height of three grains per cell. Adding a fourth grain causes the pile to topple and sends the  grains to the four adjacent cells. If these adjacent sand grain piles exceed three grains, they will in turn also tumble and send sand grains to their neighbors, possibly causing an 'avalanche' of tumbling piles. Sand grains reaching the edge of the table will 'fall off' and are considered lost. The model has been used to study self-organizing criticality.

![Lakhesis thread image](/images/LakhesisThread.GIF)

During experimentation with the model it was noted that adding a second sand pile resulted in interesting, and recurring, patterns at the boundary between the two piles. Lakhesis has been set up in order to explore how these boundary patterns develop and how factors such as the number of additional piles and their orientation and distance from each other affect these patterns. The lattice is relatively small (1280 x 720 pixels - 720p), again envisioned as a table that sand grains can fall off once they reach the edge, and the model runs till 2 million sand grains have been dropped after which it restarts on a blank table from a default configuration. In the default setting Lakhesis randomly generates up to twelve grid cells within the lattice onto which sand grains are dropped. The drop points are confined to an inner rectangular area surrounded by a 250 pixel boundary. This boundary leaves room for sand piles to develop and keeps the number of lost grains to a minimum. Sand grains are added to each new point starting at a randomly generated time, while existing piles continue to grow. The colors for piles with zero, one, and two grains are randomly generated and change each time a new pile is started. Piles with three grains are black in order to emphasis the patterns that cross through these stable areas. Many of these patterns have the appearance of threads intertwining and weaving into textile-like surfaces (i.e. the name of the crate - apologies for the mixed metaphors). Some of the random aspects of the model can be altered through keyboard commands during execution (see Usage below).

Keywords
--------

[abelian sandpile model][3], [cellular automaton][4], [self-organized criticality][5]

Requirements
------------

Lakhesis was compiled using Rust 1.56 

Dependencies
------------

* winit
* winit_input_helper
* pixel
* image

Usage
-----

`cargo run --release`

There are no command line arguments.

Keyboard commands are available after execution starts and are shown on the title bar:

* [Q] Quit the program
* [N] Start a new simulation
* [P] Pause the simulation. Press [P] again to resume simulation
* [S] Save an image of the current screen as a PNG. The file is saved to the project folder or the folder containing the executable
* [Spacebar] Step through the simulation adding one sand grain each time the [Spacebar] is pressed.
* [Up] The up arrow increases the interval between screen updates by a factor of 10 to a maximum of 100,000
* [Down] The down arrow decreases the interval between screen updates by a factor of 10 to a minimum of 1
* [A] Add a new sandpile to the simulation up to a maximum of 12. Turns off random changes
* [C] Change the colors displayed with a randomly selected set of colors. Turns off random changes
* [R] Turns off random changes. Turns random changes on if random changes are off or if [A] or [C] have been pressed

Pressing the [Spacebar] automatically pauses the simulation, press [P] to resume. [A]dding a sandpile and changing [C]olors turns off most random aspects of Lakhesis. To resume random sandpiles and colors press [R]. To keep a randomly generated configuration from changing press [R] to set random to false. See the comments section about the design compromises in Lakhesis concerning randomness and user interaction.

Comments
--------

The title bar of the Lakhesis window provides a map of keyboard commands. To keep the labels short the description for the up and down arrow keys are a little misleading. When Lakhesis starts it defaults to displaying the current state of the model every 1000 interations (1000 sand grains have been dropped). The display interval can be changed by a factor of 10 down to a minumum of 1 (the result of adding each sand grain) and up to a maximum of 100,000. More frequent updates slow down overall progress of the model, however the interesting patterns mentioned above are more apparent at these intervals. Conversely, increasing the interval between updates can 'speed' up the model to get to the point where the edges of sandpiles interact, at which point the interval can be decreased to observe the interference patterns.

A light gray rectangle indicates the edges of the 'table'. Four gray points forming the vertices of a rectangle within the table are displayed to indicate the boundaries within which the randomly generated sandpiles will start. New sandpiles that start within the boundaries of an existing sandpile may be hard to see at first. If it was randomly generated a change in colors will indicate the new addition.

Compromises have been made in generating and displaying the sandpile model in Lakhesis. Many are a result of my limited experience with Rust and UIs and some are deliberate choices made between competing goals of the simulation. An earlier version used a configuration file to set an ever-growing number of parameters. Constantly reconfiguring the file slowed down exploration of sandpile dynamics and randomly generating some parameters resulted in configurations that I may not have thought to try. This goes for the colors chosen to display the model. Admittedly, many, if not most, randomly generated colors are garish or don't show the model to the best effect. On the other hand, more often than I would have expected, randomly selected colors can be quite beautiful. If the generated colors don't work for you, keep hitting the [C] key till you find one you like. The upper limit of 12 sandpiles, a 1280 x 720 lattice, and resetting the model at 2 million sand grains are arbitrary and can be changed in the source code by altering a value assigned as a constant. They were chosen because they allowed exploration of the boundary patterns before the model consumed too much CPU time to be responsive. During most runs of the model an upper limit of 2 million grains fills the lattice and allows patterns to fully develop.

There is an undocumented feature that allows a number of frames to be exported at the current screen update interval by pressing the [V] key. The number is set to 600 by default. These 600 images could, for example, be animated with video editing software to produce a 10 second 720p video at 60fps.

License
-------

The content of this repository is licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)

[1]: https://www.theoi.com/Daimon/Moirai.html
[2]: https://en.wikipedia.org/wiki/Moirai
[3]: https://en.wikipedia.org/wiki/Abelian_sandpile_model
[4]: https://en.wikipedia.org/wiki/Cellular_automaton
[5]: https://en.wikipedia.org/wiki/Self-organized_criticality
