*this note is for thinking about the **code editor** portion of the project*

### Prior Art
[Lapce](https://github.com/lapce/lapce) seems really cool.
* I wonder though: does it expose a crate that I could integrate into my [[egui integrations]]? 
* Could I make the [[spec viewer]] be a separate crate that could also be compiled to WASI format as a Lapce plugin?

**TODO**
* Read from/write to disk for save/open of projects
  * Allow for creation of new projects and editing of existing ones
* Line numbers (togglable in settings)
* Syntax Highlighting
  * Benchmarking / optimizations and supporting all colorschemes of the overall app