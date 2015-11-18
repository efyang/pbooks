### pbooks - downloads selected books from vhf/free-programming-books

##### Crates to use
* hyper - to download the books and serve the books if need be (concurrent)
* ncurses/conrod - for selecting the books
* toml or something for storing user selection and correct checksums
* ~~twox-hash - for checking integrity of files and generation checksums~~
* clap - to specify premade manifests, package format, etc.
* zip and tar - for creating distributable packages
* clippy - good code

##### General Layout
1. Parse original free-programming-books .md to get list of pdf links and titles
2. Use that to create a UI - maybe a list of choices that are either selected of not
  * struct for each entry
    * Title
    * Link
    * selected: bool
3. Begin download process by checking the ```downloaded``` directory
  1. get list of wanted filenames
  2. get list of files in ```downloaded``` directory
  3. compare the two - if is not downloaded then download
4. Spawn hyper client(s) to download the files
  * Single or multiple process?
  * Have error list to allow for retry, much like vim-plug
5. Package downloaded files into .tar.xz and .zip files
  * All files in ```downloaded``` directory or only the ones in the manifest?
  * Maximum compression ratio (If specifiable)
6. Finished

##### Plans
* UI of some sort to pick categories?
  * Have an update function in UI -> spawn git process and then reparse
* Store choices in some files?
* Check if file exists before downloading - check hashes
* Maybe serve the books? - Useful for classes
