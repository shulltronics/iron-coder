*this note is for the **spec viewer** portion of the project*

Thoughts:

* ~~Integrate 3D model view~~ I've chosen the transparent image approach, as it will be more robust; boards which don't have a 3d model can easily be added, and external contributors will be more easily able to add new boards.

* Decide on backend for the board types.. should it be some kind of relational database, or just text files?
  * For now I'm doing a file hierarchy system. This seems easy and totally practical.

* TODO -- how can I include "main" and "peripheral" boards? 
  * My vision is that a person, when starting a project, can select the dev boards that they are working with, and then have some examples/resources to get started with that set. But this is challenging because each set might have multiple configurations and use-cases. 
    * One simplifying assumption is that each "project" can only have one "main" board (i.e. the board that the firmware is running on).
  * Should the app detect if boards are "compatible" or should this be left to the user?
  * How to display projects with multiple boards?
  * Integrate related crates and examples in spec viewer.